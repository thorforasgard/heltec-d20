use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::prelude::*;

use heltec_d20_shared::dice::RngSource;
use heltec_d20_shared::animation::AnimationState;
use heltec_d20_shared::ui;
use heltec_d20_shared::AppState;

/// Desktop RNG using rand crate
struct DesktopRng;

impl RngSource for DesktopRng {
    fn random_u32(&mut self) -> u32 {
        rand::random::<u32>()
    }
}

fn main() -> Result<()> {
    // Terminal setup
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let result = run(&mut terminal);

    // Teardown
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut state = AppState::new();
    let mut rng = DesktopRng;
    let mut last_tick = Instant::now();
    let tick_rate = Duration::from_millis(50); // 20fps during animation

    loop {
        // Render
        terminal.draw(|frame| {
            // Simulate 128x64 OLED size (21x8) or use full terminal
            ui::draw(frame, &state);
        })?;

        // Handle input (non-blocking)
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),

                        KeyCode::Char(' ') | KeyCode::Enter => {
                            // Roll!
                            if state.animation.is_idle() {
                                state.animation = AnimationState::start_roll();
                            }
                        }

                        KeyCode::Tab => {
                            state.cycle_die();
                        }

                        KeyCode::Char('h') => {
                            state.toggle_history();
                        }

                        // Vim-style die selection
                        KeyCode::Left | KeyCode::Char('k') => {
                            if state.die_index > 0 {
                                state.die_index -= 1;
                                state.current_die = state.die_types[state.die_index];
                            }
                        }
                        KeyCode::Right | KeyCode::Char('j') => {
                            if state.die_index < state.die_types.len() - 1 {
                                state.die_index += 1;
                                state.current_die = state.die_types[state.die_index];
                            }
                        }

                        _ => {}
                    }
                }
            }
        }

        // Tick animation
        if last_tick.elapsed() >= tick_rate {
            let was_animating = !state.animation.is_idle();
            state.animation.tick(&mut rng, state.current_die);

            // Check if animation just finished
            if was_animating {
                if let Some(result) = state.animation.final_result() {
                    state.last_result = Some(result);
                    state.history.push(state.current_die, result);
                    state.animation = AnimationState::Idle;
                }
            }

            last_tick = Instant::now();
        }
    }
}
