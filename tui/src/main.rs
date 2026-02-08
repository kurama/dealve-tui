mod config;
mod events;
mod message;
mod model;
mod onboarding;
mod tasks;
mod update;
mod view;

use anyhow::Result;
use crossterm::{
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{backend::CrosstermBackend, layout::Rect, prelude::Color, Terminal};
use std::io::{stdout, Stdout};
use tachyonfx::fx::EvolveSymbolSet;
use tachyonfx::pattern::RadialPattern;
use tachyonfx::{fx, Effect, EffectTimer, Interpolation, Motion};

use model::Model;
use tasks::TaskManager;
use update::UpdateResult;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let api_key = config::Config::load_api_key();
    let mut terminal = setup_terminal()?;

    let result = if api_key.is_none() {
        match onboarding::run_onboarding(&mut terminal).await {
            Ok(Some(key)) => run(&mut terminal, Some(key)).await,
            Ok(None) => Ok(()),
            Err(e) => Err(e),
        }
    } else {
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

async fn run(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    api_key: Option<String>,
) -> Result<()> {
    let mut model = Model::new(api_key);
    let mut task_mgr = TaskManager::new();

    // Start initial load
    tasks::start_load(&mut model, &mut task_mgr);

    // Tachyonfx effects for animations
    let mut effects: Vec<(Effect, Rect)> = Vec::new();
    let mut last_frame_time = std::time::Instant::now();

    // Initial evolve_into effect for app startup
    let term_size = terminal.size()?;
    let full_screen = Rect::new(0, 0, term_size.width, term_size.height);

    let style = ratatui::style::Style::default()
        .fg(Color::Rgb(20, 15, 30))
        .bg(Color::Rgb(10, 8, 15));

    let timer = EffectTimer::from_ms(1200, Interpolation::CubicOut);
    effects.push((
        fx::evolve_into((EvolveSymbolSet::Shaded, style), timer)
            .with_pattern(RadialPattern::center().with_transition_width(15.0)),
        full_screen,
    ));

    loop {
        // ── 1. View ─────────────────────────────────────────────────────
        let elapsed = last_frame_time.elapsed();
        last_frame_time = std::time::Instant::now();

        terminal.draw(|frame| {
            view::view(frame, &mut model);

            for (effect, area) in effects.iter_mut() {
                effect.process(elapsed.into(), frame.buffer_mut(), *area);
            }
        })?;

        effects.retain(|(effect, _)| !effect.done());

        if model.should_quit {
            break;
        }

        // ── 2. Check async tasks ────────────────────────────────────────
        let task_messages = tasks::check_tasks(&mut model, &mut task_mgr).await;
        for msg in task_messages {
            let result = update::update(&mut model, msg);
            handle_result(&mut model, &mut task_mgr, &mut effects, terminal, result)?;
        }

        // ── 3. Game info loading (debounced) ────────────────────────────
        if task_mgr.pending_game_info_load
            && !model.loading.deals
            && effects.is_empty()
            && task_mgr.last_selection_change.elapsed()
                >= std::time::Duration::from_millis(model.game_info_delay_ms)
        {
            task_mgr.pending_game_info_load = false;
            tasks::load_game_info_if_needed(&mut model).await;
        }

        // ── 4. Handle event ─────────────────────────────────────────────
        let poll_duration = if !effects.is_empty() {
            std::time::Duration::from_millis(16)
        } else {
            std::time::Duration::from_millis(50)
        };

        if let Some(msg) = events::handle_event(&model, poll_duration)? {
            let result = update::update(&mut model, msg);
            handle_result(&mut model, &mut task_mgr, &mut effects, terminal, result)?;
        }
    }

    Ok(())
}

fn handle_result(
    model: &mut Model,
    task_mgr: &mut TaskManager,
    effects: &mut Vec<(Effect, Rect)>,
    terminal: &Terminal<CrosstermBackend<Stdout>>,
    result: UpdateResult,
) -> Result<()> {
    // Handle selection changed → reset debounce timer
    if result.selection_changed {
        task_mgr.last_selection_change = std::time::Instant::now();
        task_mgr.pending_game_info_load = true;
    }

    // Handle reload request → spawn new deals load
    if result.needs_reload {
        tasks::start_load(model, task_mgr);

        // Trigger sweep-in effect for deals list
        let term_size = terminal.size()?;
        let deals_inner = Rect::new(
            1,
            1,
            term_size.width / 2 - 2,
            term_size.height.saturating_sub(2),
        );
        effects.push((
            fx::sweep_in(
                Motion::UpToDown,
                15,
                3,
                Color::Rgb(20, 15, 30),
                (600, Interpolation::QuadOut),
            ),
            deals_inner,
        ));
    }

    // Handle chained messages
    if let Some(chained_msg) = result.msg {
        let chained_result = update::update(model, chained_msg);
        handle_result(model, task_mgr, effects, terminal, chained_result)?;
    }

    Ok(())
}
