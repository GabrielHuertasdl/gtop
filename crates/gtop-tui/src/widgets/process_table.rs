use ratatui::{
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
    Frame,
};
use unicode_width::UnicodeWidthStr;

use gtop_core::process::filter::SortColumn;
use gtop_core::process::ProcessStatus;
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
    let sort_col = &state.sort_column;

    // Header
    let header_cells: Vec<Cell> = build_columns()
        .into_iter()
        .map(|(col, label, _)| {
            let is_active = col == *sort_col;
            let suffix = if is_active {
                if state.sort_ascending { " ▲" } else { " ▼" }
            } else {
                ""
            };
            let style = if is_active {
                theme.accent_style().add_modifier(Modifier::UNDERLINED)
            } else {
                theme.header_style()
            };
            Cell::from(format!("{}{}", label, suffix)).style(style)
        })
        .collect();

    let header = Row::new(header_cells).height(1).bottom_margin(0);

    let rows: Vec<Row> = state
        .processes
        .iter()
        .enumerate()
        .map(|(i, proc)| {
            let is_cursor = i == state.cursor_index;
            let is_selected = state.selected_pids.contains(&proc.pid);

            let status_color = match proc.status {
                ProcessStatus::Running   => theme.status_run,
                ProcessStatus::Sleeping  => theme.status_sleep,
                ProcessStatus::Stopped   => theme.status_stop,
                ProcessStatus::Zombie    => theme.status_zombie,
                ProcessStatus::DiskSleep => theme.status_disk,
                _                        => theme.dim_fg,
            };

            let cpu_color = theme.cpu_color(proc.cpu_pct);
            let name_style = theme.process_name_style(proc.cpu_pct, proc.is_kernel);

            let row_style = if is_selected {
                theme.selected_style()
            } else if proc.is_kernel {
                theme.dim_style()
            } else {
                Style::default()
            };

            let cells = vec![
                Cell::from(proc.pid.to_string()),
                Cell::from(truncate(&proc.username, 9)),
                Cell::from(format!("{:>5.1}", proc.cpu_pct))
                    .style(Style::default().fg(cpu_color).add_modifier(
                        if proc.cpu_pct > 80.0 { Modifier::BOLD } else { Modifier::empty() },
                    )),
                Cell::from(format!("{:>5.1}", proc.mem_pct))
                    .style(Style::default().fg(theme.mem_color)),
                Cell::from(format!("{:>6}", format_bytes(proc.mem_bytes))),
                Cell::from(format!("{:>3}", proc.nice)),
                Cell::from(proc.status_char().to_string())
                    .style(Style::default().fg(status_color)),
                Cell::from(Span::raw(truncate(&proc.name, 35)))
                    .style(name_style),
            ];

            let row = Row::new(cells).style(row_style);
            if is_cursor && !is_selected {
                row.style(theme.highlight_style())
            } else {
                row
            }
        })
        .collect();

    let widths = col_widths(area.width);

    let title = format!(" Processes ({}) ", state.processes.len());
    let search_indicator = if state.search_active || !state.search.is_empty() {
        format!(" [/{}]", state.search)
    } else {
        String::new()
    };

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(title, theme.accent_style()),
            Span::styled(search_indicator, Style::default().fg(theme.search_active)),
        ]))
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .border_type(BorderType::Rounded);

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .row_highlight_style(theme.highlight_style());

    table_state.select(Some(state.cursor_index));
    f.render_stateful_widget(table, area, table_state);
}

// Columns: (SortColumn, label, fixed_width)
fn build_columns() -> Vec<(SortColumn, &'static str, u16)> {
    vec![
        (SortColumn::Pid,      "PID",   7u16),
        (SortColumn::User,     "USER",  9),
        (SortColumn::CpuPct,   "CPU%",  6),
        (SortColumn::MemPct,   "MEM%",  6),
        (SortColumn::MemBytes, "RSS",   7),
        (SortColumn::Nice,     "NI",    4),
        (SortColumn::Status,   "S",     2),
        (SortColumn::Name,     "NAME",  0),
    ]
}

fn col_widths(total: u16) -> Vec<Constraint> {
    // Fixed cols: 7+9+6+6+7+4+2 + borders + padding ≈ 52
    let fixed: u16 = 52;
    let remaining = total.saturating_sub(fixed).max(12);

    vec![
        Constraint::Length(7),
        Constraint::Length(9),
        Constraint::Length(6),
        Constraint::Length(6),
        Constraint::Length(7),
        Constraint::Length(4),
        Constraint::Length(2),
        Constraint::Min(remaining),
    ]
}

fn truncate(s: &str, max_cols: usize) -> String {
    let width = s.width();
    if width <= max_cols {
        return s.to_string();
    }
    let mut result = String::new();
    let mut cols = 0;
    for c in s.chars() {
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(1);
        if cols + w > max_cols.saturating_sub(1) {
            result.push('…');
            break;
        }
        result.push(c);
        cols += w;
    }
    result
}
