mod app;
mod config;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io::{stdout, Stdout}};

use app::{App, Popup};

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

async fn load_deals_with_spinner(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<()> {
    app.set_loading(true);

    let api_key_clone = env::var("ITAD_API_KEY").ok();
    let platform_filter = app.platform_filter;
    let region_code = app.region.code().to_string();
    let load_task = tokio::spawn(async move {
        let client = dealve_api::ItadClient::new(api_key_clone);
        let shop_id = platform_filter.shop_id();
        client.get_deals(&region_code, 50, shop_id).await
    });

    // Animate spinner while loading
    while !load_task.is_finished() {
        terminal.draw(|frame| ui::render(frame, app))?;
        app.tick_spinner();
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    }

    // Get the result
    match load_task.await {
        Ok(Ok(deals)) => {
            app.deals = deals;
            app.list_state.select(Some(0));
            app.error = None;
        }
        Ok(Err(e)) => {
            app.error = Some(e.to_string());
        }
        Err(_) => {
            app.error = Some("Task failed".to_string());
        }
    }
    app.set_loading(false);

    Ok(())
}

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, api_key: Option<String>) -> Result<()> {
    let mut app = App::new(api_key);

    // Initial load with animated spinner
    load_deals_with_spinner(terminal, &mut app).await?;

    // Track when selection changed to debounce game info loading
    let mut last_selection_change = std::time::Instant::now();
    let mut pending_game_info_load = false;
    let mut pending_deals_load = false;

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        if app.should_quit {
            break;
        }

        // Check if we need to load deals (with animated spinner)
        if pending_deals_load {
            pending_deals_load = false;
            load_deals_with_spinner(terminal, &mut app).await?;
            last_selection_change = std::time::Instant::now();
            pending_game_info_load = true;
            continue; // Redraw immediately after loading
        }

        // Check if we should load game info (after 200ms of no selection change)
        if pending_game_info_load && last_selection_change.elapsed() >= std::time::Duration::from_millis(200) {
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
                                if needs_reload {
                                    pending_deals_load = true;
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
                                    pending_deals_load = true;
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
                            KeyCode::Esc => app.toggle_menu(),
                            KeyCode::Down | KeyCode::Char('j') => app.menu_next(),
                            KeyCode::Up | KeyCode::Char('k') => app.menu_previous(),
                            KeyCode::Enter => {
                                app.menu_select().await;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc => app.toggle_menu(),
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
                            KeyCode::Enter => app.open_selected_deal(),
                            KeyCode::Char('r') => {
                                app.set_loading(true);
                                pending_deals_load = true;
                            }
                            KeyCode::Char('s') => {
                                app.cycle_sort_order();
                                last_selection_change = std::time::Instant::now();
                                pending_game_info_load = true;
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
