use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutMode {
    Compact,   // < 100 cols
    Full,      // >= 100 cols
}

pub struct AppLayout {
    pub header: Rect,
    pub metrics: Rect,
    pub main: Rect,         // process table
    pub detail: Option<Rect>, // right-side detail panel
    pub statusbar: Rect,
    pub mode: LayoutMode,
}

pub fn compute_layout(area: Rect, detail_visible: bool) -> AppLayout {
    let mode = if area.width < 100 {
        LayoutMode::Compact
    } else {
        LayoutMode::Full
    };

    // Compact: 1 line of text. Full: block border (2) + 4 content rows = 6.
    let metrics_height = if mode == LayoutMode::Compact { 2 } else { 6 };

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),             // header
            Constraint::Length(metrics_height), // metrics
            Constraint::Min(5),                // main content
            Constraint::Length(1),             // statusbar
        ])
        .split(area);

    let header = vertical[0];
    let metrics = vertical[1];
    let content_area = vertical[2];
    let statusbar = vertical[3];

    let (main, detail) = if detail_visible && area.width >= 80 {
        let detail_width = (area.width / 3).max(30).min(60);
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(40),
                Constraint::Length(detail_width),
            ])
            .split(content_area);
        (horizontal[0], Some(horizontal[1]))
    } else {
        (content_area, None)
    };

    AppLayout {
        header,
        metrics,
        main,
        detail,
        statusbar,
        mode,
    }
}
