use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io::Stdout;
use std::time::Instant;
use tachyonfx::fx::EvolveSymbolSet;
use tachyonfx::pattern::RadialPattern;
use tachyonfx::{fx, Effect, EffectTimer, Interpolation, Motion};

use crate::config::Config;
use crate::view::styles::{
    ACCENT_GREEN, ASCII_LOGO, BG_DARK, ERROR_RED, PURPLE_ACCENT, PURPLE_LIGHT, PURPLE_PRIMARY,
    SHORTCUT_KEY, TEXT_PRIMARY, TEXT_SECONDARY,
};

const SPINNER_FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

#[derive(Debug, Clone, PartialEq)]
pub enum OnboardingStep {
    Welcome,
    Instructions,
    ApiKeyEntry,
    Validating,
    Success,
    Failed { error: String },
}

pub struct OnboardingState {
    pub step: OnboardingStep,
    pub api_key_input: String,
    pub key_hidden: bool,
    pub spinner_frame: usize,
    pub should_quit: bool,
}

impl OnboardingState {
    pub fn new() -> Self {
        Self {
            step: OnboardingStep::Welcome,
            api_key_input: String::new(),
            key_hidden: true,
            spinner_frame: 0,
            should_quit: false,
        }
    }

    pub fn tick_spinner(&mut self) {
        self.spinner_frame = (self.spinner_frame + 1) % SPINNER_FRAMES.len();
    }

    pub fn spinner_char(&self) -> char {
        SPINNER_FRAMES[self.spinner_frame]
    }

    pub fn progress_dots(&self) -> [bool; 4] {
        match self.step {
            OnboardingStep::Welcome => [true, false, false, false],
            OnboardingStep::Instructions => [true, true, false, false],
            OnboardingStep::ApiKeyEntry => [true, true, true, false],
            OnboardingStep::Validating
            | OnboardingStep::Success
            | OnboardingStep::Failed { .. } => [true, true, true, true],
        }
    }

    pub fn displayed_key(&self) -> String {
        if self.api_key_input.is_empty() {
            String::new()
        } else if self.key_hidden {
            "*".repeat(self.api_key_input.len())
        } else {
            self.api_key_input.clone()
        }
    }
}

pub fn render(frame: &mut Frame, state: &OnboardingState) {
    let area = frame.area();

    // Clear background
    let bg = Block::default().style(Style::default().bg(BG_DARK));
    frame.render_widget(bg, area);

    match &state.step {
        OnboardingStep::Welcome => render_welcome(frame, area),
        OnboardingStep::Instructions => render_instructions(frame, area),
        OnboardingStep::ApiKeyEntry => render_api_key_entry(frame, state, area),
        OnboardingStep::Validating => render_validating(frame, state, area),
        OnboardingStep::Success => render_success(frame, area),
        OnboardingStep::Failed { error } => render_failed(frame, area, error),
    }

    // Render progress dots at bottom
    render_progress_dots(frame, state, area);
}

fn render_welcome(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top padding
            Constraint::Length(6), // Logo
            Constraint::Length(2), // Gap
            Constraint::Length(1), // Title
            Constraint::Length(2), // Gap
            Constraint::Length(2), // Subtitle
            Constraint::Length(3), // Gap
            Constraint::Length(6), // Info box
            Constraint::Min(0),    // Remaining space
        ])
        .split(area);

    // Logo
    let logo_text: Vec<Line> = ASCII_LOGO
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(PURPLE_PRIMARY))))
        .collect();
    let logo = Paragraph::new(logo_text).alignment(Alignment::Center);
    frame.render_widget(logo, chunks[1]);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("Welcome to ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(
            "Dealve",
            Style::default()
                .fg(PURPLE_LIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" - Game Deal Finder", Style::default().fg(TEXT_SECONDARY)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[3]);

    // Subtitle
    let subtitle = Paragraph::new(vec![
        Line::from(Span::styled(
            "Browse the best game deals from IsThereAnyDeal.com",
            Style::default().fg(TEXT_SECONDARY),
        )),
        Line::from(Span::styled(
            "across Steam, GOG, Humble, Epic, and more stores.",
            Style::default().fg(TEXT_SECONDARY),
        )),
    ])
    .alignment(Alignment::Center);
    frame.render_widget(subtitle, chunks[5]);

    // Info box
    let box_width = 54;
    let box_x = area.width.saturating_sub(box_width) / 2;
    let info_area = Rect::new(box_x, chunks[7].y, box_width, 4);

    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PURPLE_ACCENT));

    let info_text = Paragraph::new(vec![
        Line::from(Span::styled(
            "To get started, you'll need an IsThereAnyDeal",
            Style::default().fg(TEXT_PRIMARY),
        )),
        Line::from(Span::styled(
            "API key. Don't worry - it's free!",
            Style::default().fg(TEXT_PRIMARY),
        )),
    ])
    .alignment(Alignment::Center)
    .block(info_block);
    frame.render_widget(info_text, info_area);

    // Action hint at bottom
    render_action_hints(frame, area, &[("Enter", "Continue"), ("Esc", "Quit")]);
}

fn render_instructions(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Top padding
            Constraint::Length(1),  // Title
            Constraint::Length(2),  // Gap
            Constraint::Length(14), // Instructions box
            Constraint::Length(2),  // Gap
            Constraint::Length(1),  // Tip
            Constraint::Min(0),     // Remaining
        ])
        .split(area);

    // Title with btop-style brackets
    let title = Paragraph::new(Line::from(vec![
        Span::styled("┐", Style::default().fg(PURPLE_ACCENT)),
        Span::styled(
            "Getting Your API Key",
            Style::default()
                .fg(PURPLE_LIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("┌", Style::default().fg(PURPLE_ACCENT)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[1]);

    // Instructions box
    let box_width = 60;
    let box_x = area.width.saturating_sub(box_width) / 2;
    let instructions_area = Rect::new(box_x, chunks[3].y, box_width, 12);

    let instructions_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PURPLE_ACCENT));

    let step_style = Style::default()
        .fg(PURPLE_PRIMARY)
        .add_modifier(Modifier::BOLD);
    let text_style = Style::default().fg(TEXT_PRIMARY);

    let instructions = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  1. ", step_style),
            Span::styled("Go to ", text_style),
            Span::styled(
                "isthereanydeal.com",
                Style::default()
                    .fg(PURPLE_LIGHT)
                    .add_modifier(Modifier::UNDERLINED),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  2. ", step_style),
            Span::styled("Create an account or log in", text_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  3. ", step_style),
            Span::styled("Click your profile → Apps", text_style),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  4. ", step_style),
            Span::styled("Create a new app and copy the API key", text_style),
        ]),
        Line::from(""),
    ])
    .block(instructions_block);
    frame.render_widget(instructions, instructions_area);

    // Tip
    let tip = Paragraph::new(Line::from(vec![
        Span::styled("Tip: Press ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled("[o]", Style::default().fg(SHORTCUT_KEY)),
        Span::styled(
            " to open the website in your browser",
            Style::default().fg(TEXT_SECONDARY),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(tip, chunks[5]);

    render_action_hints(
        frame,
        area,
        &[
            ("o", "Open website"),
            ("Enter", "Continue"),
            ("Esc", "Back"),
        ],
    );
}

fn render_api_key_entry(frame: &mut Frame, state: &OnboardingState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Top padding
            Constraint::Length(1),  // Title
            Constraint::Length(2),  // Gap
            Constraint::Length(12), // Input box
            Constraint::Min(0),     // Remaining
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("┐", Style::default().fg(PURPLE_ACCENT)),
        Span::styled(
            "Enter Your API Key",
            Style::default()
                .fg(PURPLE_LIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("┌", Style::default().fg(PURPLE_ACCENT)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[1]);

    // Input box
    let box_width = 60;
    let box_x = area.width.saturating_sub(box_width) / 2;
    let input_area = Rect::new(box_x, chunks[3].y, box_width, 10);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(PURPLE_ACCENT));

    // Build input field content
    let displayed = state.displayed_key();
    let cursor = if displayed.len() < 50 { "▋" } else { "" };

    let input_content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Paste your IsThereAnyDeal API key below:",
            Style::default().fg(TEXT_PRIMARY),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("┌", Style::default().fg(TEXT_SECONDARY)),
            Span::styled("─".repeat(46), Style::default().fg(TEXT_SECONDARY)),
            Span::styled("┐", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("│ ", Style::default().fg(TEXT_SECONDARY)),
            Span::styled(
                format!("{:<44}", format!("{}{}", displayed, cursor)),
                Style::default().fg(PURPLE_LIGHT),
            ),
            Span::styled(" │", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(vec![
            Span::styled("└", Style::default().fg(TEXT_SECONDARY)),
            Span::styled("─".repeat(46), Style::default().fg(TEXT_SECONDARY)),
            Span::styled("┘", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Tip: Use Ctrl+V or Ctrl+Shift+V to paste",
            Style::default().fg(TEXT_SECONDARY),
        )),
    ])
    .alignment(Alignment::Center)
    .block(input_block);
    frame.render_widget(input_content, input_area);

    let toggle_label = if state.key_hidden {
        "Show key"
    } else {
        "Hide key"
    };
    render_action_hints(
        frame,
        area,
        &[("Enter", "Validate"), ("t", toggle_label), ("Esc", "Back")],
    );
}

fn render_validating(frame: &mut Frame, state: &OnboardingState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top padding
            Constraint::Length(1), // Title
            Constraint::Length(5), // Gap
            Constraint::Length(1), // Spinner
            Constraint::Min(0),    // Remaining
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("┐", Style::default().fg(PURPLE_ACCENT)),
        Span::styled(
            "Validating...",
            Style::default()
                .fg(PURPLE_LIGHT)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("┌", Style::default().fg(PURPLE_ACCENT)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[1]);

    // Spinner
    let spinner = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{} ", state.spinner_char()),
            Style::default().fg(PURPLE_PRIMARY),
        ),
        Span::styled(
            "Connecting to IsThereAnyDeal...",
            Style::default().fg(TEXT_SECONDARY),
        ),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(spinner, chunks[3]);
}

fn render_success(frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top padding
            Constraint::Length(1), // Title
            Constraint::Length(2), // Gap
            Constraint::Length(8), // Success box
            Constraint::Min(0),    // Remaining
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("┐", Style::default().fg(ACCENT_GREEN)),
        Span::styled(
            "Setup Complete!",
            Style::default()
                .fg(ACCENT_GREEN)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("┌", Style::default().fg(ACCENT_GREEN)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[1]);

    // Success box
    let box_width = 54;
    let box_x = area.width.saturating_sub(box_width) / 2;
    let success_area = Rect::new(box_x, chunks[3].y, box_width, 6);

    let success_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT_GREEN));

    let success_text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "✓ API Key Valid!",
            Style::default()
                .fg(ACCENT_GREEN)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Your key has been saved. You're all set!",
            Style::default().fg(TEXT_PRIMARY),
        )),
    ])
    .alignment(Alignment::Center)
    .block(success_block);
    frame.render_widget(success_text, success_area);

    render_action_hints(frame, area, &[("Enter", "Start Dealve")]);
}

fn render_failed(frame: &mut Frame, area: Rect, error: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Top padding
            Constraint::Length(1),  // Title
            Constraint::Length(2),  // Gap
            Constraint::Length(10), // Error box
            Constraint::Min(0),     // Remaining
        ])
        .split(area);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("┐", Style::default().fg(ERROR_RED)),
        Span::styled(
            "Validation Failed",
            Style::default().fg(ERROR_RED).add_modifier(Modifier::BOLD),
        ),
        Span::styled("┌", Style::default().fg(ERROR_RED)),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(title, chunks[1]);

    // Error box
    let box_width = 54;
    let box_x = area.width.saturating_sub(box_width) / 2;
    let error_area = Rect::new(box_x, chunks[3].y, box_width, 8);

    let error_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ERROR_RED));

    let error_text = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "✗ Invalid API Key",
            Style::default().fg(ERROR_RED).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(error, Style::default().fg(TEXT_SECONDARY))),
        Line::from(""),
        Line::from(Span::styled(
            "Please check your key and try again.",
            Style::default().fg(TEXT_PRIMARY),
        )),
    ])
    .alignment(Alignment::Center)
    .block(error_block);
    frame.render_widget(error_text, error_area);

    render_action_hints(
        frame,
        area,
        &[("Enter", "Try again"), ("o", "Open ITAD"), ("Esc", "Quit")],
    );
}

fn render_progress_dots(frame: &mut Frame, state: &OnboardingState, area: Rect) {
    let dots = state.progress_dots();
    let dot_line = Line::from(vec![
        Span::styled("[Step ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(
            match state.step {
                OnboardingStep::Welcome => "1",
                OnboardingStep::Instructions => "2",
                OnboardingStep::ApiKeyEntry => "3",
                _ => "4",
            },
            Style::default().fg(PURPLE_LIGHT),
        ),
        Span::styled(" of 4] ", Style::default().fg(TEXT_SECONDARY)),
        Span::styled(
            if dots[0] { "●" } else { "○" },
            Style::default().fg(if dots[0] {
                PURPLE_PRIMARY
            } else {
                TEXT_SECONDARY
            }),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            if dots[1] { "●" } else { "○" },
            Style::default().fg(if dots[1] {
                PURPLE_PRIMARY
            } else {
                TEXT_SECONDARY
            }),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            if dots[2] { "●" } else { "○" },
            Style::default().fg(if dots[2] {
                PURPLE_PRIMARY
            } else {
                TEXT_SECONDARY
            }),
        ),
        Span::styled(" ", Style::default()),
        Span::styled(
            if dots[3] { "●" } else { "○" },
            Style::default().fg(if dots[3] {
                PURPLE_PRIMARY
            } else {
                TEXT_SECONDARY
            }),
        ),
    ]);

    let y = area.height.saturating_sub(4);
    let dots_area = Rect::new(0, y, area.width, 1);
    let dots_widget = Paragraph::new(dot_line).alignment(Alignment::Center);
    frame.render_widget(dots_widget, dots_area);
}

fn render_action_hints(frame: &mut Frame, area: Rect, hints: &[(&str, &str)]) {
    let spans: Vec<Span> = hints
        .iter()
        .enumerate()
        .flat_map(|(i, (key, action))| {
            let mut s = vec![
                Span::styled(format!("[{}]", key), Style::default().fg(SHORTCUT_KEY)),
                Span::styled(format!(" {}", action), Style::default().fg(TEXT_SECONDARY)),
            ];
            if i < hints.len() - 1 {
                s.push(Span::styled("  ", Style::default()));
            }
            s
        })
        .collect();

    let y = area.height.saturating_sub(2);
    let hints_area = Rect::new(0, y, area.width, 1);
    let hints_widget = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    frame.render_widget(hints_widget, hints_area);
}

/// Run the onboarding flow
/// Returns Some(api_key) on success, None if user quit
pub async fn run_onboarding(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<Option<String>> {
    let mut state = OnboardingState::new();
    let mut effects: Vec<(Effect, Rect)> = Vec::new();
    let mut last_frame_time = Instant::now();

    // Initial evolve_into animation
    let term_size = terminal.size()?;
    let full_screen = Rect::new(0, 0, term_size.width, term_size.height);

    let style = ratatui::style::Style::default()
        .fg(BG_DARK)
        .bg(Color::Rgb(10, 8, 15));

    let timer = EffectTimer::from_ms(1000, Interpolation::CubicOut);
    effects.push((
        fx::evolve_into((EvolveSymbolSet::Shaded, style), timer)
            .with_pattern(RadialPattern::center().with_transition_width(12.0)),
        full_screen,
    ));

    loop {
        let elapsed = last_frame_time.elapsed();
        last_frame_time = Instant::now();

        terminal.draw(|frame| {
            render(frame, &state);

            // Apply effects
            for (effect, area) in effects.iter_mut() {
                effect.process(elapsed.into(), frame.buffer_mut(), *area);
            }
        })?;

        // Remove completed effects
        effects.retain(|(effect, _)| !effect.done());

        if state.should_quit {
            return Ok(None);
        }

        // Handle validation step
        if state.step == OnboardingStep::Validating {
            state.tick_spinner();

            // Only validate once effects are done (so animation plays)
            if effects.is_empty() {
                // Perform validation
                match dealve_api::ItadClient::validate_api_key(&state.api_key_input).await {
                    Ok(()) => {
                        // Save to config
                        let mut config = Config::load();
                        if let Err(e) = config.set_api_key(state.api_key_input.clone()) {
                            state.step = OnboardingStep::Failed {
                                error: format!("Failed to save config: {}", e),
                            };
                        } else {
                            state.step = OnboardingStep::Success;
                            // Add success animation
                            let term_size = terminal.size()?;
                            effects.push((
                                fx::sweep_in(
                                    Motion::UpToDown,
                                    10,
                                    2,
                                    BG_DARK,
                                    (400, Interpolation::QuadOut),
                                ),
                                Rect::new(0, 0, term_size.width, term_size.height),
                            ));
                        }
                    }
                    Err(e) => {
                        state.step = OnboardingStep::Failed {
                            error: e.to_string(),
                        };
                    }
                }
            }
        }

        // Poll for events
        let poll_duration = if !effects.is_empty() || state.step == OnboardingStep::Validating {
            std::time::Duration::from_millis(16)
        } else {
            std::time::Duration::from_millis(50)
        };

        if event::poll(poll_duration)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match &state.step {
                        OnboardingStep::Welcome => match key.code {
                            KeyCode::Enter => {
                                state.step = OnboardingStep::Instructions;
                                add_transition_effect(&mut effects, terminal)?;
                            }
                            KeyCode::Esc => {
                                state.should_quit = true;
                            }
                            _ => {}
                        },
                        OnboardingStep::Instructions => match key.code {
                            KeyCode::Enter => {
                                state.step = OnboardingStep::ApiKeyEntry;
                                add_transition_effect(&mut effects, terminal)?;
                            }
                            KeyCode::Char('o') => {
                                let _ = webbrowser::open("https://isthereanydeal.com/apps/");
                            }
                            KeyCode::Esc => {
                                state.step = OnboardingStep::Welcome;
                                add_transition_effect(&mut effects, terminal)?;
                            }
                            _ => {}
                        },
                        OnboardingStep::ApiKeyEntry => match key.code {
                            KeyCode::Enter => {
                                if !state.api_key_input.is_empty() {
                                    state.step = OnboardingStep::Validating;
                                }
                            }
                            KeyCode::Char('t') => {
                                state.key_hidden = !state.key_hidden;
                            }
                            KeyCode::Backspace => {
                                state.api_key_input.pop();
                            }
                            KeyCode::Char(c) => {
                                // Allow alphanumeric and dashes (UUID format)
                                if c.is_alphanumeric() || c == '-' {
                                    state.api_key_input.push(c);
                                }
                            }
                            KeyCode::Esc => {
                                state.step = OnboardingStep::Instructions;
                                add_transition_effect(&mut effects, terminal)?;
                            }
                            _ => {}
                        },
                        OnboardingStep::Validating => {
                            // No input during validation
                        }
                        OnboardingStep::Success => {
                            if key.code == KeyCode::Enter {
                                return Ok(Some(state.api_key_input));
                            }
                        }
                        OnboardingStep::Failed { .. } => match key.code {
                            KeyCode::Enter => {
                                state.step = OnboardingStep::ApiKeyEntry;
                                add_transition_effect(&mut effects, terminal)?;
                            }
                            KeyCode::Char('o') => {
                                let _ = webbrowser::open("https://isthereanydeal.com/apps/");
                            }
                            KeyCode::Esc => {
                                state.should_quit = true;
                            }
                            _ => {}
                        },
                    }
                }
            }
        }
    }
}

fn add_transition_effect(
    effects: &mut Vec<(Effect, Rect)>,
    terminal: &Terminal<CrosstermBackend<Stdout>>,
) -> Result<()> {
    let term_size = terminal.size()?;
    effects.push((
        fx::sweep_in(
            Motion::LeftToRight,
            8,
            2,
            BG_DARK,
            (250, Interpolation::QuadOut),
        ),
        Rect::new(0, 0, term_size.width, term_size.height),
    ));
    Ok(())
}
