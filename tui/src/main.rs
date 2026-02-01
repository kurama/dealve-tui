mod app;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{env, io::{stdout, Stdout}};

use app::App;

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

async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, api_key: Option<String>) -> Result<()> {
    let mut app = App::new(api_key);

    app.load_deals().await;

    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        if app.should_quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if app.show_platform_dropdown {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('f') => app.toggle_dropdown(),
                            KeyCode::Down | KeyCode::Char('j') => app.dropdown_next(),
                            KeyCode::Up | KeyCode::Char('k') => app.dropdown_previous(),
                            KeyCode::Enter => {
                                app.dropdown_select().await;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                            KeyCode::Char('f') => app.toggle_dropdown(),
                            KeyCode::Enter => app.open_selected_deal(),
                            KeyCode::Char('r') => {
                                app.load_deals().await;
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
