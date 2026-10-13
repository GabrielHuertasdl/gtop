use ratatui::{
    layout::Alignment,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use gtop_core::process::ProcessStatus;
use gtop_core::state::AppState;
use gtop_core::util::format_uptime;

use crate::theme::Theme;

pub fn render(f: &mut Frame, area: ratatui::layout::Rect, state: &AppState, theme: &Theme) {
    let uptime = format_uptime(state.metrics.cpu.uptime_secs);
    let load = format!(
        "{:.2} {:.2} {:.2}",
        state.metrics.cpu.load_1m,
        state.metrics.cpu.load_5m,
        state.metrics.cpu.load_15m,
    );

    let proc_count = state.tree.by_pid.len();
    let running = state
        .processes
        .iter()
        .filter(|p| p.status == ProcessStatus::Running)
        .count();

    let hostname = read_hostname();
    let time_str = chrono::Local::now().format("%H:%M:%S").to_string();

    // Identity logo box: ╔══ GTOP ══╗
    let logo = if theme.ascii {
        " GTOP ".to_string()
    } else {
        "╔══ GTOP ══╗".to_string()
    };

    let sep = if theme.ascii { " -- " } else { " ── " };

    let spans = vec![
        Span::styled(logo, theme.logo_style()),
        Span::raw(" "),
        Span::styled(&hostname, Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{sep}up {uptime}{sep}"), theme.dim_style()),
        Span::styled(format!("load {load}"), theme.dim_style()),
        Span::styled(format!("{sep}"), theme.dim_style()),
        Span::styled(
            format!("{proc_count} procs ({running}R)"),
            theme.dim_style(),
        ),
    ];

    let right = Paragraph::new(time_str)
        .alignment(Alignment::Right)
        .style(theme.dim_style());

    let left = Paragraph::new(Line::from(spans));

    f.render_widget(left, area);
    f.render_widget(right, area);
}

fn read_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_default()
        .trim()
        .to_string()
}
