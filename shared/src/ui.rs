use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::AppState;
use crate::animation::AnimationState;

// Yggdrasil Nexus colors (mapped to closest ANSI for monochrome fallback)
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

    // Split: big number area (top) + die selector + recent rolls (bottom)
    let chunks = Layout::vertical([
        Constraint::Min(1),        // Big number
        Constraint::Length(1),     // Die selector
        Constraint::Length(1),     // Recent rolls
    ])
    .split(inner);

    // Big number display
    draw_big_number(frame, state, chunks[0]);

    // Die type selector
    draw_die_selector(frame, state, chunks[1]);

    // Recent rolls
    draw_recent_rolls(frame, state, chunks[2]);
}

fn draw_big_number(frame: &mut Frame, state: &AppState, area: Rect) {
    let (text, style) = match &state.animation {
        AnimationState::Idle => {
            match state.last_result {
                Some(val) => (
                    format!("{}", val),
                    Style::default().fg(WHITE).add_modifier(Modifier::BOLD),
                ),
                None => (
                    "ROLL!".to_string(),
                    Style::default().fg(DIM),
                ),
            }
        }
        AnimationState::Rolling { display_value, .. } => (
            format!("{}", display_value),
            Style::default().fg(BLUE),
        ),
        AnimationState::Landed { result, flash_frames } => {
            let style = if *flash_frames > 0 && *flash_frames % 2 == 0 {
                Style::default().fg(Color::Black).bg(GREEN).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(GREEN).add_modifier(Modifier::BOLD)
            };
            (format!("{}", result), style)
        }
    };

    let paragraph = Paragraph::new(text)
        .style(style)
        .alignment(Alignment::Center);

    // Center vertically
    let y_offset = area.height.saturating_sub(1) / 2;
    let centered = Rect::new(area.x, area.y + y_offset, area.width, 1);
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
