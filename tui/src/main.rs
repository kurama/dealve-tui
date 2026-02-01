mod app;
mod config;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use dealve_core::models::{Deal, Platform};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io::{stdout, Stdout}, time::Instant};
use tokio::task::JoinHandle;

use app::{App, Popup};

type DealsLoadTask = JoinHandle<dealve_core::Result<Vec<Deal>>>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = env::var("ITAD_API_KEY").ok();
    if api_key.is_none() {
        eprintln!("Error: ITAD_API_KEY not set.");
        eprintln!("Create a .env file with:");
        eprintln!("ITAD_API_KEY=your_key_here");
        return Ok(());
    }

    let mut terminal = setup_terminal()?;
    let result = run(&mut terminal, api_key).await;
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
fn spawn_deals_load(platform_filter: Platform, region_code: String, offset: usize, page_size: usize) -> DealsLoadTask {
    let api_key = env::var("ITAD_API_KEY").ok();
    tokio::spawn(async move {
        let client = dealve_api::ItadClient::new(api_key);
        let shop_id = platform_filter.shop_id();
        client.get_deals(&region_code, page_size, offset, shop_id).await
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
        app.platform_filter,
        app.region.code().to_string(),
        0,
        app.deals_page_size,
    ));

    // Task for loading more deals (pagination)
    let mut load_more_task: Option<DealsLoadTask> = None;

    // Track when selection changed to debounce game info loading
    let mut last_selection_change = Instant::now();
    let mut pending_game_info_load = false;

    loop {
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        if app.should_quit {
            break;
        }

        // Check if initial/refresh load task completed
        if check_load_task(&mut app, &mut load_task, false).await {
            last_selection_change = std::time::Instant::now();
            pending_game_info_load = true;
        }

        // Check if load-more task completed
        check_load_task(&mut app, &mut load_more_task, true).await;

        // Check if we should load more deals (infinite scroll)
        if app.should_load_more() && load_more_task.is_none() && load_task.is_none() {
            app.loading_more = true;
            load_more_task = Some(spawn_deals_load(
                app.platform_filter,
                app.region.code().to_string(),
                app.deals_offset,
                app.deals_page_size,
            ));
        }

        // Tick spinner if loading
        if app.loading || app.loading_more {
            app.tick_spinner();
        }

        // Check if we should load game info (after debounce delay)
        if pending_game_info_load && !app.loading && last_selection_change.elapsed() >= std::time::Duration::from_millis(app.game_info_delay_ms) {
            pending_game_info_load = false;
            app.load_game_info_for_selected().await;
        }

        if event::poll(std::time::Duration::from_millis(50))? {
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
                                        app.platform_filter,
                                        app.region.code().to_string(),
                                        0,
                                        app.deals_page_size,
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
                                            app.platform_filter,
                                            app.region.code().to_string(),
                                            0,
                                            app.deals_page_size,
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
                                        app.platform_filter,
                                        app.region.code().to_string(),
                                        0,
                                        app.deals_page_size,
                                    ));
                                }
                            }
                            KeyCode::Char('s') => {
                                app.cycle_sort_order();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
                            }
                            KeyCode::Char('c') => {
                                // Clear filter if one is active
                                if !app.filter_text.is_empty() {
                                    app.clear_filter();
                                    last_selection_change = std::time::Instant::now();
                                    pending_game_info_load = true;
                                }
                            }
                            KeyCode::Char('m') => {
                                // Load more deals (same as infinite scroll)
                                if app.has_more_deals && load_task.is_none() && load_more_task.is_none() {
                                    app.loading_more = true;
                                    load_more_task = Some(spawn_deals_load(
                                        app.platform_filter,
                                        app.region.code().to_string(),
                                        app.deals_offset,
                                        app.deals_page_size,
                                    ));
                                }
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
