extern crate alloc;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::AppState;
use crate::animation::AnimationState;
use crate::sprites;

// Yggdrasil Nexus colors
const GREEN: Color = Color::Rgb(45, 216, 129);   // Verdandi Green
const GOLD: Color = Color::Rgb(244, 201, 93);    // Sif's Gold
const BLUE: Color = Color::Rgb(74, 158, 255);    // Urd's Blue
const DIM: Color = Color::Rgb(107, 114, 128);    // Muted gray
const WHITE: Color = Color::Rgb(224, 224, 224);   // Light text

/// Main render function
pub fn draw(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    if state.show_history {
        draw_history_view(frame, state, area);
    } else {
        draw_roll_view(frame, state, area);
    }
}

fn draw_roll_view(frame: &mut Frame, state: &AppState, area: Rect) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GREEN))
        .title(Span::styled(
            format!(" ⚡ {} ", state.current_die.name().to_uppercase()),
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Split: die sprite (top) + die selector + recent rolls (bottom)
    let chunks = Layout::vertical([
        Constraint::Min(3),        // Die sprite
        Constraint::Length(1),     // Die selector
        Constraint::Length(1),     // Recent rolls
    ])
    .split(inner);

    // Die sprite display
    draw_die_sprite(frame, state, chunks[0]);

    // Die type selector
    draw_die_selector(frame, state, chunks[1]);

    // Recent rolls
    draw_recent_rolls(frame, state, chunks[2]);
}

fn draw_die_sprite(frame: &mut Frame, state: &AppState, area: Rect) {
    let die_name = state.current_die.name();

    let (lines, style) = match &state.animation {
        AnimationState::Idle => {
            match state.last_result {
                Some(val) => {
                    let face = sprites::die_face(die_name, val);
                    let lines: Vec<Line> = face
                        .iter()
                        .map(|s| Line::from(s.to_string()))
                        .collect();
                    (lines, Style::default().fg(WHITE))
                }
                None => {
                    // Show die shape with "?" prompt
                    let frames = sprites::tumble_frames(die_name);
                    let lines: Vec<Line> = frames[0]
                        .iter()
                        .map(|s| Line::from(s.replace("??", "? ")))
                        .collect();
                    (lines, Style::default().fg(DIM))
                }
            }
        }
        AnimationState::Rolling { frame: anim_frame, display_value } => {
            let frames = sprites::tumble_frames(die_name);
            let frame_idx = (*anim_frame as usize) % frames.len();
            let val_str = format!("{:<2}", display_value);
            let lines: Vec<Line> = frames[frame_idx]
                .iter()
                .map(|s| Line::from(s.replace("??", &val_str)))
                .collect();
            (lines, Style::default().fg(BLUE))
        }
        AnimationState::Landed { result, flash_frames } => {
            let face = sprites::die_face(die_name, *result);
            let lines: Vec<Line> = face
                .iter()
                .map(|s| Line::from(s.to_string()))
                .collect();
            let style = if *flash_frames > 0 && *flash_frames % 2 == 0 {
                Style::default().fg(Color::Black).bg(GREEN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
            };
            (lines, style)
        }
    };

    // Center the sprite vertically in available space
    let sprite_height = lines.len() as u16;
    let y_offset = area.height.saturating_sub(sprite_height) / 2;
    let centered = Rect::new(area.x, area.y + y_offset, area.width, sprite_height.min(area.height));

    let paragraph = Paragraph::new(lines)
        .style(style)
        .alignment(Alignment::Center);
    frame.render_widget(paragraph, centered);
}

fn draw_die_selector(frame: &mut Frame, state: &AppState, area: Rect) {
    let spans: Vec<Span> = state
        .die_types
        .iter()
        .enumerate()
        .map(|(i, die)| {
            if i == state.die_index {
                Span::styled(
                    format!("▸{}", die.name()),
                    Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(
                    format!(" {}", die.name()),
                    Style::default().fg(DIM),
                )
            }
        })
        .collect();

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).alignment(Alignment::Center);
    frame.render_widget(paragraph, area);
}

fn draw_recent_rolls(frame: &mut Frame, state: &AppState, area: Rect) {
    let recent: Vec<String> = state
        .history
        .recent(6)
        .map(|r| format!("{}", r.result))
        .collect();

    if recent.is_empty() {
        let p = Paragraph::new("Press SPACE to roll")
            .style(Style::default().fg(DIM))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    } else {
        let text = format!("Last: {}", recent.join(" "));
        let p = Paragraph::new(text)
            .style(Style::default().fg(DIM))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }
}

fn draw_history_view(frame: &mut Frame, state: &AppState, area: Rect) {
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(GREEN))
        .title(Span::styled(
            " 📊 HISTORY ",
            Style::default().fg(GOLD).add_modifier(Modifier::BOLD),
        ));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let mut lines: Vec<Line> = Vec::new();

    // Stats for current die
    if let Some((min, max, avg, count)) = state.history.stats_for(state.current_die) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{}: ", state.current_die.name()),
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("{}r min:{} max:{} avg:{}", count, min, max, avg),
                Style::default().fg(WHITE),
            ),
        ]));
    }

    lines.push(Line::from(""));

    // Recent rolls
    for record in state.history.recent(inner.height.saturating_sub(2) as usize) {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{:>4} ", record.die.name()),
                Style::default().fg(BLUE),
            ),
            Span::styled(
                format!("→ {}", record.result),
                Style::default().fg(WHITE),
            ),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}
