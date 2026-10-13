use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use gtop_core::state::AppState;
use gtop_core::util::{format_bytes, format_speed};

use crate::layout::LayoutMode;
use crate::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme, mode: &LayoutMode) {
    match mode {
        LayoutMode::Compact => render_compact(f, area, state, theme),
        LayoutMode::Full => render_full(f, area, state, theme),
    }
}

// ─── Compact (single line) ────────────────────────────────────────────────────

fn render_compact(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let cpu = state.metrics.cpu.global_pct;
    let mem = &state.metrics.memory;
    let spark = state.history.cpu_sparkline(20, theme.ascii);

    let bat_str = state
        .metrics
        .battery
        .as_ref()
        .map(|b| {
            let arrow = match b.status {
                gtop_core::metrics::BatteryStatus::Charging => "↑",
                gtop_core::metrics::BatteryStatus::Discharging => "↓",
                _ => "",
            };
            format!("  BAT:{}%{}", b.capacity_pct, arrow)
        })
        .unwrap_or_default();

    let temp_str = cpu_temp_str(state);

    let text = Line::from(vec![
        Span::styled(format!("CPU:{:.1}%", cpu), Style::default().fg(theme.cpu_color(cpu))),
        Span::raw(" "),
        Span::styled(spark, Style::default().fg(theme.cpu_color(cpu))),
        Span::raw("  "),
        Span::styled(
            format!("RAM:{}/{}", format_bytes(mem.used), format_bytes(mem.total)),
            Style::default().fg(theme.mem_color),
        ),
        Span::styled(temp_str, Style::default().fg(Color::Yellow)),
        Span::styled(bat_str, Style::default().fg(Color::Green)),
    ]);

    f.render_widget(Paragraph::new(text), area);
}

// ─── Full ──────────────────────────────────────────────────────────────────────

fn render_full(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Split: left (core grid) | right (system overview)
    let n_cores = state.metrics.cpu.per_core_pct.len().max(1);
    let core_rows = ((n_cores + 1) / 2) as u16;
    let min_core_height = core_rows.min(inner.height);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(inner);

    render_core_grid(f, cols[0], state, theme, min_core_height);
    render_system_overview(f, cols[1], state, theme);
}

// ─── Core grid ────────────────────────────────────────────────────────────────

fn render_core_grid(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme, max_rows: u16) {
    let cores = &state.metrics.cpu.per_core_pct;
    let n = cores.len();
    if n == 0 {
        return;
    }

    let half = (n + 1) / 2;
    let col_w = (area.width / 2) as usize;

    let mut lines: Vec<Line> = Vec::new();

    for i in 0..half {
        if lines.len() >= max_rows as usize || lines.len() >= area.height as usize {
            break;
        }

        let left_pct = *cores.get(i).unwrap_or(&0.0);
        let right_pct = *cores.get(i + half).unwrap_or(&0.0);

        let mut spans = build_core_bar(i, left_pct, col_w, state, theme);
        if i + half < n {
            spans.extend(build_core_bar(i + half, right_pct, col_w, state, theme));
        }
        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), area);
}

fn build_core_bar<'a>(
    idx: usize,
    pct: f32,
    col_w: usize,
    state: &AppState,
    theme: &Theme,
) -> Vec<Span<'a>> {
    // label: "C0 " = 3 chars, pct: " 62%" = 4 chars, freq: " 3.1G" = 5 chars = 12 fixed
    let label = format!("C{:<2} ", idx);
    let pct_str = format!("{:3.0}%", pct);
    let freq_str = if let Some(&freq) = state.metrics.cpu.freq_mhz.get(idx) {
        format!(" {:.1}G", freq as f32 / 1000.0)
    } else {
        String::new()
    };

    let fixed = label.len() + pct_str.len() + freq_str.len() + 1; // +1 padding
    let bar_w = col_w.saturating_sub(fixed).clamp(4, 20);
    let filled = ((pct / 100.0) * bar_w as f32).round() as usize;
    let empty = bar_w.saturating_sub(filled);

    let color = theme.cpu_color(pct);
    let filled_str: String = std::iter::repeat(theme.block_full()).take(filled).collect();
    let empty_str: String = std::iter::repeat(theme.block_empty()).take(empty).collect();

    vec![
        Span::styled(label, theme.dim_style()),
        Span::styled(filled_str, Style::default().fg(color)),
        Span::styled(empty_str, Style::default().fg(theme.dim_fg)),
        Span::styled(format!(" {}", pct_str), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(freq_str, theme.dim_style()),
    ]
}

// ─── System overview (right column) ───────────────────────────────────────────

fn render_system_overview(f: &mut Frame, area: Rect, state: &AppState, theme: &Theme) {
    let mem = &state.metrics.memory;
    let bar_w = (area.width.saturating_sub(16)) as usize;

    // CPU sparkline line
    let spark = state.history.cpu_sparkline(bar_w.min(40), theme.ascii);
    let cpu_pct = state.metrics.cpu.global_pct;
    let cpu_line = Line::from(vec![
        Span::styled("CPU ", theme.header_style()),
        Span::styled(spark, Style::default().fg(theme.cpu_color(cpu_pct))),
        Span::styled(
            format!(" {:5.1}%", cpu_pct),
            Style::default().fg(theme.cpu_color(cpu_pct)).add_modifier(Modifier::BOLD),
        ),
    ]);

    // Memory bar line
    let mem_pct = mem.used_pct();
    let mem_filled = ((mem_pct / 100.0) * bar_w.min(40) as f32) as usize;
    let mem_bar = build_bar(mem_filled, bar_w.min(40), theme);
    let mem_line = Line::from(vec![
        Span::styled("RAM ", theme.header_style()),
        Span::styled(mem_bar, Style::default().fg(theme.mem_color)),
        Span::styled(
            format!(" {}/{}", format_bytes(mem.used), format_bytes(mem.total)),
            Style::default().fg(theme.mem_color),
        ),
    ]);

    // Swap line
    let swap_line = if mem.swap_total > 0 {
        let swap_pct = mem.swap_pct();
        let filled = ((swap_pct / 100.0) * bar_w.min(40) as f32) as usize;
        let bar = build_bar(filled, bar_w.min(40), theme);
        Line::from(vec![
            Span::styled("SWP ", theme.header_style()),
            Span::styled(bar, Style::default().fg(theme.swap_color)),
            Span::styled(
                format!(" {}/{}", format_bytes(mem.swap_used), format_bytes(mem.swap_total)),
                Style::default().fg(theme.swap_color),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("SWP ", theme.header_style()),
            Span::styled("none", theme.dim_style()),
        ])
    };

    // Net + temp line
    let net_line = build_net_line(state, theme);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    if rows.len() >= 1 { f.render_widget(Paragraph::new(cpu_line), rows[0]); }
    if rows.len() >= 2 { f.render_widget(Paragraph::new(mem_line), rows[1]); }
    if rows.len() >= 3 { f.render_widget(Paragraph::new(swap_line), rows[2]); }
    if rows.len() >= 4 { f.render_widget(Paragraph::new(net_line), rows[3]); }
}

fn build_bar(filled: usize, total: usize, theme: &Theme) -> String {
    let empty = total.saturating_sub(filled);
    let f: String = std::iter::repeat(theme.block_full()).take(filled).collect();
    let e: String = std::iter::repeat(theme.block_empty()).take(empty).collect();
    format!("{}{}", f, e)
}

fn build_net_line<'a>(state: &AppState, theme: &Theme) -> Line<'a> {
    let temp_str = cpu_temp_str(state);
    let bat_str = state.metrics.battery.as_ref().map(|b| {
        let a = match b.status {
            gtop_core::metrics::BatteryStatus::Charging => "↑",
            gtop_core::metrics::BatteryStatus::Discharging => "↓",
            _ => "=",
        };
        format!("  BAT:{}%{}", b.capacity_pct, a)
    }).unwrap_or_default();

    if let Some(net) = state.metrics.networks.first() {
        Line::from(vec![
            Span::styled(format!("↓{}", format_speed(net.rx_bytes_per_sec)), Style::default().fg(theme.net_rx)),
            Span::raw("  "),
            Span::styled(format!("↑{}", format_speed(net.tx_bytes_per_sec)), Style::default().fg(theme.net_tx)),
            Span::styled(format!("  {}", net.interface), theme.dim_style()),
            Span::styled(temp_str, Style::default().fg(Color::Yellow)),
            Span::styled(bat_str, Style::default().fg(Color::Green)),
        ])
    } else {
        Line::from(vec![
            Span::styled("no net", theme.dim_style()),
            Span::styled(temp_str, Style::default().fg(Color::Yellow)),
            Span::styled(bat_str, Style::default().fg(Color::Green)),
        ])
    }
}

fn cpu_temp_str(state: &AppState) -> String {
    state.metrics.temperatures
        .iter()
        .find(|t| t.label.to_lowercase().contains("cpu") || t.device.to_lowercase().contains("cpu"))
        .map(|t| format!("  {}:{:.0}°C", t.label, t.temp_celsius))
        .unwrap_or_default()
}
