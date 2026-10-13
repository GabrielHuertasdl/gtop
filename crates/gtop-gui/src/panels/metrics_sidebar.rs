use egui::{Color32, Margin, RichText, Ui};

use gtop_core::state::AppState;
use gtop_core::util::{format_bytes, format_uptime};

use crate::theme::GuiTheme;

pub fn render(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    // Sidebar gets surface color (darker than main panel bg)
    egui::Frame::new()
        .fill(theme.surface)
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("sidebar_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_min_width(160.0);
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 3.0);

                    cpu_section(ui, state, theme);
                    divider(ui, theme);
                    ram_section(ui, state, theme);

                    if state.metrics.memory.swap_total > 0 {
                        divider(ui, theme);
                        swap_section(ui, state, theme);
                    }

                    if !state.metrics.networks.is_empty() {
                        divider(ui, theme);
                        net_section(ui, state, theme);
                    }

                    if !state.metrics.temperatures.is_empty() {
                        divider(ui, theme);
                        temp_section(ui, state, theme);
                    }

                    if let Some(ref bat) = state.metrics.battery {
                        divider(ui, theme);
                        bat_section(ui, bat, theme);
                    }
                });
        });
}

// ── Section helpers ───────────────────────────────────────────────────────────

fn cpu_section(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    let cpu = &state.metrics.cpu;
    let pct = cpu.global_pct;

    section_label(ui, "CPU", theme);
    ui.label(
        RichText::new(format!("{:.1}%", pct))
            .color(theme.cpu_color(pct))
            .strong()
            .size(20.0),
    );
    ui.label(
        RichText::new(format!(
            "load  {:.2}  {:.2}  {:.2}",
            cpu.load_1m, cpu.load_5m, cpu.load_15m
        ))
        .color(theme.text_dim)
        .size(11.0),
    );
    ui.label(
        RichText::new(format!("up    {}", format_uptime(cpu.uptime_secs)))
            .color(theme.text_dim)
            .size(11.0),
    );
    ui.add_space(4.0);

    // Per-core mini bars — text only, no widgets
    for (i, &core_pct) in cpu.per_core_pct.iter().enumerate() {
        let bar = mini_bar(core_pct, 10);
        let color = theme.cpu_color(core_pct);
        ui.label(
            RichText::new(format!("c{:<2} {} {:4.1}%", i, bar, core_pct))
                .color(color)
                .monospace()
                .size(10.0),
        );
    }
}

fn ram_section(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    let mem = &state.metrics.memory;
    let pct = mem.used_pct();

    section_label(ui, "RAM", theme);
    ui.label(
        RichText::new(format!("{:.1}%", pct))
            .color(theme.mem_color)
            .strong()
            .size(20.0),
    );
    ui.label(
        RichText::new(format!(
            "{} / {}",
            format_bytes(mem.used),
            format_bytes(mem.total)
        ))
        .color(theme.text_dim)
        .size(11.0),
    );
    ui.label(
        RichText::new(format!("avail {}", format_bytes(mem.available)))
            .color(theme.text_dim)
            .size(11.0),
    );
}

fn swap_section(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    let mem = &state.metrics.memory;
    let pct = if mem.swap_total > 0 {
        (mem.swap_used as f64 / mem.swap_total as f64 * 100.0) as f32
    } else {
        0.0
    };

    section_label(ui, "SWP", theme);
    ui.label(
        RichText::new(format!("{:.1}%", pct))
            .color(theme.swap_color)
            .strong()
            .size(16.0),
    );
    ui.label(
        RichText::new(format!(
            "{} / {}",
            format_bytes(mem.swap_used),
            format_bytes(mem.swap_total)
        ))
        .color(theme.text_dim)
        .size(11.0),
    );
}

fn net_section(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    section_label(ui, "NET", theme);
    for net in state.metrics.networks.iter().take(3) {
        ui.label(
            RichText::new(&net.interface)
                .color(theme.text)
                .size(11.0)
                .strong(),
        );
        ui.label(
            RichText::new(format!("↓ {}/s", format_bytes(net.rx_bytes_per_sec)))
                .color(theme.net_rx)
                .monospace()
                .size(11.0),
        );
        ui.label(
            RichText::new(format!("↑ {}/s", format_bytes(net.tx_bytes_per_sec)))
                .color(theme.net_tx)
                .monospace()
                .size(11.0),
        );
    }
}

fn temp_section(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    section_label(ui, "TEMP", theme);
    for t in state.metrics.temperatures.iter().take(4) {
        let color = theme.temp_color(t.temp_celsius);
        ui.label(
            RichText::new(format!("{:<12} {:.0}°C", truncate(&t.label, 12), t.temp_celsius))
                .color(color)
                .monospace()
                .size(11.0),
        );
    }
}

fn bat_section(ui: &mut Ui, bat: &gtop_core::metrics::BatteryMetrics, theme: &GuiTheme) {
    let color = theme.bat_color(bat.capacity_pct as u32);
    section_label(ui, "BAT", theme);
    ui.label(
        RichText::new(format!("{}%", bat.capacity_pct))
            .color(color)
            .strong()
            .size(20.0),
    );
    let status_str = format!("{:?}", bat.status).to_lowercase();
    ui.label(RichText::new(status_str).color(theme.text_dim).size(11.0));
}

// ── Layout utilities ──────────────────────────────────────────────────────────

fn section_label(ui: &mut Ui, label: &str, theme: &GuiTheme) {
    ui.label(
        RichText::new(label)
            .color(theme.text_dim)
            .size(10.0)
            .strong(),
    );
}

fn divider(ui: &mut Ui, theme: &GuiTheme) {
    ui.add_space(6.0);
    ui.painter().hline(
        ui.available_rect_before_wrap().x_range(),
        ui.cursor().top(),
        egui::Stroke::new(1.0, theme.border),
    );
    ui.add_space(6.0);
}

/// ASCII mini-bar: "████░░░░░░" style using block chars, width=total_chars.
fn mini_bar(pct: f32, width: usize) -> String {
    let filled = ((pct / 100.0).clamp(0.0, 1.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}..", &s[..max.saturating_sub(2)]) }
}
