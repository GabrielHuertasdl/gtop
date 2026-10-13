use ratatui::{
    layout::Rect,
    style::Color,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame,
};

use gtop_core::process::ProcessInfo;
use gtop_core::util::format_bytes;

use crate::theme::Theme;

pub fn render(f: &mut Frame, area: Rect, proc: Option<&ProcessInfo>, theme: &Theme) {
    let block = Block::default()
        .title(" Process Detail ")
        .title_style(theme.accent_style())
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let Some(p) = proc else {
        f.render_widget(
            Paragraph::new("No process selected").style(theme.dim_style()),
            inner,
        );
        return;
    };

    let lines: Vec<Line> = vec![
        row("PID", p.pid.to_string(), theme),
        row("PPID", p.ppid.to_string(), theme),
        row("Name", p.name.clone(), theme),
        row("User", p.username.clone(), theme),
        row("Status", p.status_str().to_string(), theme),
        row("Priority", p.priority.to_string(), theme),
        row("Nice", p.nice.to_string(), theme),
        row("Threads", p.threads.to_string(), theme),
        row("CPU%", format!("{:.2}", p.cpu_pct), theme),
        row("Mem", format_bytes(p.mem_bytes), theme),
        row("Mem%", format!("{:.2}", p.mem_pct), theme),
        row("I/O Read", format_bytes(p.io_read_bytes), theme),
        row("I/O Write", format_bytes(p.io_write_bytes), theme),
        Line::from(""),
        Line::from(Span::styled("Command:", theme.header_style())),
        Line::from(Span::raw(p.cmdline.clone())),
    ];

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}

fn row(key: &str, value: String, theme: &Theme) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("{:<12} ", key),
            ratatui::style::Style::default().fg(Color::Cyan),
        ),
        Span::raw(value),
    ])
}
