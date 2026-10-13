use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame,
};

use gtop_core::state::AppState;
use gtop_core::util::format_bytes;

use crate::theme::Theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    state: &AppState,
    theme: &Theme,
    table_state: &mut TableState,
) {
    let flat = state.tree.flatten_dfs();

    let rows: Vec<Row> = flat
        .iter()
        .enumerate()
        .filter_map(|(i, (pid, depth))| {
            let proc = state.tree.by_pid.get(pid)?;

            if state.filter.hide_kernel && proc.is_kernel {
                return None;
            }

            let is_cursor = state
                .processes
                .get(state.cursor_index)
                .map(|p| p.pid == *pid)
                .unwrap_or(false);
            let is_selected = state.selected_pids.contains(pid);

            let indent = "  ".repeat(*depth);
            let prefix = if proc.children.is_empty() {
                format!("{}└─ ", indent)
            } else {
                format!("{}├─ ", indent)
            };

            let row_style = if is_selected {
                theme.selected_style()
            } else if proc.is_kernel {
                theme.dim_style()
            } else {
                Style::default()
            };

            let cells = vec![
                Cell::from(proc.pid.to_string()),
                Cell::from(format!("{}{}", prefix, proc.name)),
                Cell::from(format!("{:>5.1}", proc.cpu_pct))
                    .style(Style::default().fg(theme.cpu_color(proc.cpu_pct))),
                Cell::from(format!("{:>6}", format_bytes(proc.mem_bytes))),
            ];

            let row = Row::new(cells).style(row_style);
            Some(if is_cursor && !is_selected {
                row.style(theme.highlight_style())
            } else {
                row
            })
        })
        .collect();

    let block = Block::default()
        .title(format!(" Process Tree ({}) ", flat.len()))
        .title_style(theme.accent_style())
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .border_type(BorderType::Rounded);

    let widths = [
        Constraint::Length(7),
        Constraint::Min(20),
        Constraint::Length(6),
        Constraint::Length(7),
    ];

    let header = Row::new(["PID", "NAME", "CPU%", "RSS"])
        .style(theme.header_style())
        .height(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme.highlight_style());

    table_state.select(Some(state.cursor_index));
    f.render_stateful_widget(table, area, table_state);
}
