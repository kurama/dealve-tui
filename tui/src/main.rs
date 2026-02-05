mod app;
mod config;
mod onboarding;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use dealve_core::models::{Deal, Platform, PriceHistoryPoint};
use ratatui::{backend::CrosstermBackend, prelude::Color, layout::Rect, Terminal};
use std::{io::{stdout, Stdout}, time::Instant};
use tachyonfx::{fx, Effect, EffectTimer, Interpolation, Motion};
use tachyonfx::fx::EvolveSymbolSet;
use tachyonfx::pattern::RadialPattern;
use tokio::task::JoinHandle;

use app::{App, Popup};

type DealsLoadTask = JoinHandle<dealve_core::Result<Vec<Deal>>>;
type PriceHistoryTask = JoinHandle<(String, dealve_core::Result<Vec<PriceHistoryPoint>>)>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // Try to load API key from env or config
    let api_key = config::Config::load_api_key();

    let mut terminal = setup_terminal()?;

    let result = if api_key.is_none() {
        // No API key found - run onboarding
        match onboarding::run_onboarding(&mut terminal).await {
            Ok(Some(key)) => {
                // User completed onboarding, start the app
                run(&mut terminal, Some(key)).await
            }
            Ok(None) => {
                // User quit during onboarding
                Ok(())
            }
            Err(e) => Err(e),
        }
    } else {
        // API key found - start app directly
        run(&mut terminal, api_key).await
    };

    restore_terminal()?;
    result
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

/// Spawn a background task to load deals (non-blocking)
fn spawn_deals_load(api_key: Option<String>, platform_filter: Platform, region_code: String, offset: usize, page_size: usize, sort: String) -> DealsLoadTask {
    tokio::spawn(async move {
        let client = dealve_api::ItadClient::new(api_key);
        let shop_id = platform_filter.shop_id();
        client.get_deals(&region_code, page_size, offset, shop_id, Some(&sort)).await
    })
}

/// Check if load task is finished and handle result
/// Returns true if task completed (for initial load)
async fn check_load_task(app: &mut App, load_task: &mut Option<DealsLoadTask>, is_loading_more: bool) -> bool {
    if let Some(task) = load_task.as_mut() {
        if task.is_finished() {
            // Task finished, get result
            let task = load_task.take().unwrap();
            let page_size = app.deals_page_size;
            match task.await {
                Ok(Ok(new_deals)) => {
                    // Check if we got fewer deals than requested (no more available)
                    if new_deals.len() < page_size {
                        app.has_more_deals = false;
                    }

                    if is_loading_more {
                        // Append to existing deals
                        app.deals.extend(new_deals);
                        app.deals_offset += page_size;
                    } else {
                        // Replace deals (initial load or filter change)
                        app.deals = new_deals;
                        app.deals_offset = page_size;
                        app.list_state.select(Some(0));
                        app.table_state.select(Some(0));
                    }
                    app.error = None;
                }
                Ok(Err(e)) => {
                    app.error = Some(e.to_string());
                }
                Err(_) => {
                    app.error = Some("Task failed".to_string());
                }
            }
            if is_loading_more {
                app.loading_more = false;
            } else {
                app.set_loading(false);
            }
            return true; // Task completed
        }
    }
    false
}

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, api_key: Option<String>) -> Result<()> {
    let mut app = App::new(api_key);

    // Start initial load (non-blocking)
    app.set_loading(true);
    let mut load_task: Option<DealsLoadTask> = Some(spawn_deals_load(
        app.api_key.clone(),
        app.platform_filter,
        app.region.code().to_string(),
        0,
        app.deals_page_size,
        app.sort_state.api_param(),
    ));

    // Task for loading more deals (pagination)
    let mut load_more_task: Option<DealsLoadTask> = None;

    // Track when selection changed to debounce game info loading
    let mut last_selection_change = Instant::now();
    let mut pending_game_info_load = false;

    // Task for loading price history
    let mut price_history_task: Option<PriceHistoryTask> = None;

    // Tachyonfx effects for animations
    let mut effects: Vec<(Effect, Rect)> = Vec::new();
    let mut last_frame_time = Instant::now();

    // Initial evolve_into effect for app startup
    let term_size = terminal.size()?;
    let full_screen = Rect::new(0, 0, term_size.width, term_size.height);

    let style = ratatui::style::Style::default()
        .fg(Color::Rgb(20, 15, 30))   // BG_DARK
        .bg(Color::Rgb(10, 8, 15));   // darker bg

    let timer = EffectTimer::from_ms(1200, Interpolation::CubicOut);

    effects.push((
        fx::evolve_into((EvolveSymbolSet::Shaded, style), timer)
            .with_pattern(RadialPattern::center().with_transition_width(15.0)),
        full_screen,
    ));

    loop {
        let elapsed = last_frame_time.elapsed();
        last_frame_time = Instant::now();

        terminal.draw(|frame| {
            ui::render(frame, &mut app);

            // Apply all active effects
            for (effect, area) in effects.iter_mut() {
                effect.process(elapsed.into(), frame.buffer_mut(), *area);
            }
        })?;

        // Remove completed effects
        effects.retain(|(effect, _)| !effect.done());

        if app.should_quit {
            break;
        }

        // Check if initial/refresh load task completed
        if check_load_task(&mut app, &mut load_task, false).await {
            last_selection_change = std::time::Instant::now();
            pending_game_info_load = true;

            // Trigger sweep-in effect for deals list inner area (excluding status bar)
            let term_size = terminal.size()?;
            // Exclude top border (1), bottom border (1), and status line area
            let deals_inner = Rect::new(1, 1, term_size.width / 2 - 2, term_size.height.saturating_sub(2));
            effects.push((
                fx::sweep_in(
                    Motion::UpToDown,
                    15,  // gradient length for smoother wave
                    3,   // randomness for wave-like effect
                    Color::Rgb(20, 15, 30),  // BG_DARK color
                    (600, Interpolation::QuadOut),  // 600ms with ease-out
                ),
                deals_inner,
            ));
        }

        // Check if load-more task completed
        check_load_task(&mut app, &mut load_more_task, true).await;

        // Check if we should load more deals (infinite scroll)
        if app.should_load_more() && load_more_task.is_none() && load_task.is_none() {
            app.loading_more = true;
            load_more_task = Some(spawn_deals_load(
                app.api_key.clone(),
                app.platform_filter,
                app.region.code().to_string(),
                app.deals_offset,
                app.deals_page_size,
                app.sort_state.api_param(),
            ));
        }

        // Tick spinner if loading
        if app.loading || app.loading_more {
            app.tick_spinner();
        }

        // Check if we should load game info (after debounce delay)
        // Don't load during animations to avoid blocking the render loop
        if pending_game_info_load && !app.loading && effects.is_empty() && last_selection_change.elapsed() >= std::time::Duration::from_millis(app.game_info_delay_ms) {
            pending_game_info_load = false;
            app.load_game_info_for_selected().await;
        }

        // Check if price history task completed
        if let Some(task) = price_history_task.as_mut() {
            if task.is_finished() {
                let task = price_history_task.take().unwrap();
                if let Ok((game_id, result)) = task.await {
                    match result {
                        Ok(history) => app.finish_loading_price_history(game_id, history),
                        Err(_) => app.finish_loading_price_history(game_id, vec![]),
                    }
                }
            }
        }

        // Check if we should load price history (after game info is loaded)
        if price_history_task.is_none() && !app.loading && effects.is_empty() {
            if let Some(game_id) = app.needs_price_history_load() {
                app.start_loading_price_history(game_id.clone());
                let api_key = app.api_key.clone();
                let region_code = app.region.code().to_string();
                price_history_task = Some(tokio::spawn(async move {
                    let client = dealve_api::ItadClient::new(api_key);
                    let result = client.get_price_history(&game_id, &region_code).await;
                    (game_id, result)
                }));
            }
        }

        // Use shorter poll time during animations for smoother rendering (~60 FPS)
        let poll_duration = if !effects.is_empty() {
            std::time::Duration::from_millis(16)
        } else {
            std::time::Duration::from_millis(50)
        };
        if event::poll(poll_duration)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.popup == Popup::Platform {
                        match key.code {
                            KeyCode::Esc => app.close_popup(),
                            KeyCode::Down | KeyCode::Char('j') => app.platform_popup_next(),
                            KeyCode::Up | KeyCode::Char('k') => app.platform_popup_prev(),
                            KeyCode::Enter => {
                                let needs_reload = app.platform_popup_select();
                                if needs_reload && load_task.is_none() {
                                    app.reset_pagination();
                                    app.set_loading(true);
                                    load_task = Some(spawn_deals_load(
                                        app.api_key.clone(),
                                        app.platform_filter,
                                        app.region.code().to_string(),
                                        0,
                                        app.deals_page_size,
                                        app.sort_state.api_param(),
                                    ));
                                }
                            }
                            _ => {}
                        }
                    } else if app.popup == Popup::Options {
                        match key.code {
                            KeyCode::Esc => app.close_popup(),
                            KeyCode::Tab | KeyCode::Right => app.options_next_tab(),
                            KeyCode::BackTab | KeyCode::Left => app.options_prev_tab(),
                            KeyCode::Down | KeyCode::Char('j') => app.options_next_item(),
                            KeyCode::Up | KeyCode::Char('k') => app.options_prev_item(),
                            KeyCode::Enter | KeyCode::Char(' ') => {
                                let needs_reload = app.options_toggle_item();
                                if needs_reload {
                                    app.close_popup();
                                    if load_task.is_none() {
                                        app.reset_pagination();
                                        app.set_loading(true);
                                        load_task = Some(spawn_deals_load(
                                            app.api_key.clone(),
                                            app.platform_filter,
                                            app.region.code().to_string(),
                                            0,
                                            app.deals_page_size,
                                            app.sort_state.api_param(),
                                        ));
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else if app.popup == Popup::Keybinds {
                        if key.code == KeyCode::Esc {
                            app.close_popup();
                        }
                    } else if app.popup == Popup::PriceFilter {
                        match key.code {
                            KeyCode::Esc => app.close_popup(),
                            KeyCode::Tab => app.price_filter_switch_field(),
                            KeyCode::Enter => {
                                app.price_filter_apply();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Backspace => app.price_filter_pop(),
                            KeyCode::Char('c') => {
                                app.price_filter_clear();
                                app.close_popup();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Char(c) => app.price_filter_push(c),
                            _ => {}
                        }
                    } else if app.show_menu {
                        match key.code {
                            KeyCode::Esc => {
                                app.toggle_menu();
                            }
                            KeyCode::Char('q') => app.quit(),
                            KeyCode::Down | KeyCode::Char('j') => app.menu_next(),
                            KeyCode::Up | KeyCode::Char('k') => app.menu_previous(),
                            KeyCode::Enter => {
                                app.menu_select().await;
                            }
                            _ => {}
                        }
                    } else if app.filter_active {
                        // Filter input mode
                        match key.code {
                            KeyCode::Esc => {
                                app.cancel_filter();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Enter => {
                                app.confirm_filter();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Backspace => {
                                app.filter_pop();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Char(c) => {
                                app.filter_push(c);
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.toggle_menu();
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.next();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                app.previous();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Char('p') => {
                                app.open_platform_popup();
                            }
                            KeyCode::Char('f') => {
                                app.start_filter();
                            }
                            KeyCode::Enter => app.open_selected_deal(),
                            KeyCode::Char('r') => {
                                if load_task.is_none() {
                                    app.reset_pagination();
                                    app.set_loading(true);
                                    load_task = Some(spawn_deals_load(
                                        app.api_key.clone(),
                                        app.platform_filter,
                                        app.region.code().to_string(),
                                        0,
                                        app.deals_page_size,
                                        app.sort_state.api_param(),
                                    ));
                                }
                            }
                            KeyCode::Char('s') => {
                                let needs_reload = app.toggle_sort_direction();
                                if needs_reload && load_task.is_none() {
                                    app.reset_pagination();
                                    app.set_loading(true);
                                    load_task = Some(spawn_deals_load(
                                        app.api_key.clone(),
                                        app.platform_filter,
                                        app.region.code().to_string(),
                                        0,
                                        app.deals_page_size,
                                        app.sort_state.api_param(),
                                    ));
                                }
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Left => {
                                let needs_reload = app.prev_sort_criteria();
                                if needs_reload && load_task.is_none() {
                                    app.reset_pagination();
                                    app.set_loading(true);
                                    load_task = Some(spawn_deals_load(
                                        app.api_key.clone(),
                                        app.platform_filter,
                                        app.region.code().to_string(),
                                        0,
                                        app.deals_page_size,
                                        app.sort_state.api_param(),
                                    ));
                                }
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Right => {
                                let needs_reload = app.next_sort_criteria();
                                if needs_reload && load_task.is_none() {
                                    app.reset_pagination();
                                    app.set_loading(true);
                                    load_task = Some(spawn_deals_load(
                                        app.api_key.clone(),
                                        app.platform_filter,
                                        app.region.code().to_string(),
                                        0,
                                        app.deals_page_size,
                                        app.sort_state.api_param(),
                                    ));
                                }
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Char('c') => {
                                // Clear filters if any are active
                                if !app.filter_text.is_empty() || app.price_filter.is_active() {
                                    app.clear_filter();
                                    app.price_filter_clear();
                                    last_selection_change = std::time::Instant::now();
                                    pending_game_info_load = true;
                                }
                            }
                            KeyCode::Char('$') => {
                                app.open_price_filter_popup();
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
