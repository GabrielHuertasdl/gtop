use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use gtop_core::state::app_state::{ActionKind, PendingAction};

use crate::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, action: &PendingAction, theme: &Theme) {
    // Center the dialog
    let dialog_width = area.width.min(60).max(40);
    let dialog_height = 8u16;
    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear background
    f.render_widget(Clear, dialog_area);

    let is_danger = action.kind.is_destructive();
    let border_color = if is_danger { Color::Red } else { Color::Yellow };

    let block = Block::default()
        .title(" ⚠  Confirm Action ")
        .title_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .border_type(BorderType::Rounded);

    let inner = block.inner(dialog_area);
    f.render_widget(block, dialog_area);

    let action_label = action.kind.label();
    let targets_str = if action.pids.len() == 1 {
        format!("{} (PID {})", action.names[0], action.pids[0])
    } else {
        format!("{} processes", action.pids.len())
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("Action: "),
            Span::styled(&action_label, Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::raw("Target: "),
            Span::styled(&targets_str, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  [Enter/y]  Confirm     [Esc/n]  Cancel  ",
                Style::default().fg(Color::White).bg(if is_danger { Color::DarkGray } else { Color::DarkGray }),
            ),
        ])
        .alignment(Alignment::Center),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}
