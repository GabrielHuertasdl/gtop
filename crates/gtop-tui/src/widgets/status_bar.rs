use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use gtop_core::state::AppState;

use crate::theme::Theme;

pub fn render(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState, theme: &Theme) {
    let line = if state.pending_action.is_some() {
        Line::from(vec![
            key("[Enter/y]", "Confirm", theme),
            sep(),
            key("[Esc/n]", "Cancel", theme),
        ])
    } else if state.search_active {
        Line::from(vec![
            Span::styled(
                format!(" /{}", state.search),
                Style::default().fg(theme.search_active),
            ),
            sep(),
            key("[Enter]", "Apply", theme),
            sep(),
            key("[Esc]", "Cancel", theme),
        ])
    } else {
        Line::from(vec![
            key("[↑↓/jk]", "Move", theme),
            sep(),
            key("[/]", "Search", theme),
            sep(),
            key("[X]", "Kill", theme),
            sep(),
            key("[t]", "Term", theme),
            sep(),
            key("[Enter]", "Detail", theme),
            sep(),
            key("[q]", "Quit", theme),
        ])
    };

    f.render_widget(Paragraph::new(line), area);
}

fn key<'a>(k: &'a str, label: &'a str, theme: &Theme) -> Span<'a> {
    Span::styled(
        format!("{} {} ", k, label),
        Style::default().fg(theme.accent),
    )
}

fn sep<'a>() -> Span<'a> {
    Span::raw("│ ")
}
