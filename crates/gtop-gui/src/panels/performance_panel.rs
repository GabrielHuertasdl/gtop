use egui::{Color32, RichText, Ui};

use gtop_core::state::AppState;
use gtop_core::util::{format_bytes, format_uptime};

use crate::theme::GuiTheme;

#[derive(PartialEq, Clone, Copy, Default)]
pub enum PerfView { #[default] Cpu, Memory, Network }

pub fn render(ui: &mut Ui, state: &AppState, theme: &GuiTheme, view: &mut PerfView) {
    ui.horizontal_top(|ui| {
        // ── Left nav ─────────────────────────────────────────────────────────
        egui::Frame::new()
            .fill(theme.surface)
            .inner_margin(egui::Margin::symmetric(0, 0))
            .show(ui, |ui| {
                ui.set_min_width(140.0);
                ui.set_max_width(140.0);
                ui.set_min_height(ui.available_height());

                nav_item(ui, theme, "CPU", &format!("{:.0}%", state.metrics.cpu.global_pct),
                    theme.cpu_color(state.metrics.cpu.global_pct),
                    *view == PerfView::Cpu, || *view = PerfView::Cpu);

                nav_item(ui, theme, "Memory",
                    &format!("{:.0}%", state.metrics.memory.used_pct()),
                    theme.mem_color,
                    *view == PerfView::Memory, || *view = PerfView::Memory);

                if !state.metrics.networks.is_empty() {
                    nav_item(ui, theme, "Network", "↓↑",
                        theme.net_rx,
                        *view == PerfView::Network, || *view = PerfView::Network);
                }
            });

        ui.separator();

        // ── Detail pane ───────────────────────────────────────────────────────
        ui.vertical(|ui| {
            ui.add_space(16.0);
            match view {
                PerfView::Cpu    => cpu_detail(ui, state, theme),
                PerfView::Memory => mem_detail(ui, state, theme),
                PerfView::Network => net_detail(ui, state, theme),
            }
        });
    });
}

// ── Nav item ─────────────────────────────────────────────────────────────────

fn nav_item(
    ui: &mut Ui,
    theme: &GuiTheme,
    label: &str,
    value: &str,
    value_color: Color32,
    active: bool,
    mut on_click: impl FnMut(),
) {
    let bg = if active { theme.surface2 } else { theme.surface };
    let left_bar = if active { theme.accent } else { Color32::TRANSPARENT };

    let (resp, painter) = ui.allocate_painter(
        egui::vec2(140.0, 64.0),
        egui::Sense::click(),
    );

    painter.rect_filled(resp.rect, egui::CornerRadius::ZERO, bg);
    // Left accent bar when active
    let bar_rect = egui::Rect::from_min_size(
        resp.rect.min,
        egui::vec2(3.0, resp.rect.height()),
    );
    painter.rect_filled(bar_rect, egui::CornerRadius::ZERO, left_bar);

    let text_pos = resp.rect.min + egui::vec2(14.0, 10.0);
    painter.text(text_pos, egui::Align2::LEFT_TOP, label,
        egui::FontId::proportional(12.0), theme.text_dim);
    painter.text(text_pos + egui::vec2(0.0, 20.0), egui::Align2::LEFT_TOP, value,
        egui::FontId::proportional(18.0), value_color);

    if resp.clicked() { on_click(); }
    if resp.hovered() && !active {
        painter.rect_filled(resp.rect, egui::CornerRadius::ZERO,
            Color32::from_white_alpha(8));
    }
}

// ── CPU detail ────────────────────────────────────────────────────────────────

fn cpu_detail(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    let cpu = &state.metrics.cpu;
    let pct = cpu.global_pct;

    ui.label(RichText::new("CPU").color(theme.text_dim).size(13.0));
    ui.add_space(4.0);
    ui.label(
        RichText::new(format!("{:.1}%", pct))
            .color(theme.cpu_color(pct))
            .size(40.0)
            .strong(),
    );
    ui.label(
        RichText::new("Utilization")
            .color(theme.text_dim)
            .size(11.0),
    );

    ui.add_space(20.0);
    stat_row(ui, theme, "Speed",     &format!("{:.2} GHz", cpu.freq_mhz.iter().copied().max().unwrap_or(0) as f64 / 1000.0));
    stat_row(ui, theme, "Processes", &state.processes.len().to_string());
    stat_row(ui, theme, "Load 1m",   &format!("{:.2}", cpu.load_1m));
    stat_row(ui, theme, "Load 5m",   &format!("{:.2}", cpu.load_5m));
    stat_row(ui, theme, "Load 15m",  &format!("{:.2}", cpu.load_15m));
    stat_row(ui, theme, "Uptime",    &format_uptime(cpu.uptime_secs));
    stat_row(ui, theme, "Cores",     &cpu.per_core_pct.len().to_string());

    ui.add_space(20.0);
    ui.label(RichText::new("Per core").color(theme.text_dim).size(11.0));
    ui.add_space(4.0);

    let cols = 4usize;
    let n = cpu.per_core_pct.len();
    for row in 0..n.div_ceil(cols) {
        ui.horizontal(|ui| {
            for col in 0..cols {
                let i = row * cols + col;
                if i >= n { break; }
                let p = cpu.per_core_pct[i];
                ui.label(
                    RichText::new(format!("C{:<2} {:.0}%", i, p))
                        .color(theme.cpu_color(p))
                        .monospace()
                        .size(11.0),
                );
                ui.add_space(12.0);
            }
        });
    }
}

// ── Memory detail ─────────────────────────────────────────────────────────────

fn mem_detail(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    let mem = &state.metrics.memory;
    let pct = mem.used_pct();

    ui.label(RichText::new("Memory").color(theme.text_dim).size(13.0));
    ui.add_space(4.0);
    ui.label(
        RichText::new(format!("{:.1}%", pct))
            .color(theme.mem_color)
            .size(40.0)
            .strong(),
    );
    ui.label(RichText::new("In use").color(theme.text_dim).size(11.0));

    ui.add_space(20.0);
    stat_row(ui, theme, "Total",     &format_bytes(mem.total));
    stat_row(ui, theme, "In use",    &format_bytes(mem.used));
    stat_row(ui, theme, "Available", &format_bytes(mem.available));
    stat_row(ui, theme, "Cached",    &format_bytes(mem.cached));
    if mem.swap_total > 0 {
        ui.add_space(12.0);
        ui.label(RichText::new("Swap").color(theme.text_dim).size(11.0));
        ui.add_space(4.0);
        stat_row(ui, theme, "Total", &format_bytes(mem.swap_total));
        stat_row(ui, theme, "Used",  &format_bytes(mem.swap_used));
    }
}

// ── Network detail ────────────────────────────────────────────────────────────

fn net_detail(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    ui.label(RichText::new("Network").color(theme.text_dim).size(13.0));
    ui.add_space(12.0);

    for net in &state.metrics.networks {
        ui.label(RichText::new(&net.interface).color(theme.text).strong().size(13.0));
        stat_row(ui, theme, "Download", &format!("{}/s", format_bytes(net.rx_bytes_per_sec)));
        stat_row(ui, theme, "Upload",   &format!("{}/s", format_bytes(net.tx_bytes_per_sec)));
        stat_row(ui, theme, "Total RX", &format_bytes(net.rx_total));
        stat_row(ui, theme, "Total TX", &format_bytes(net.tx_total));
        ui.add_space(12.0);
    }
}

// ── Shared helpers ────────────────────────────────────────────────────────────

fn stat_row(ui: &mut Ui, theme: &GuiTheme, key: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{:<14}", key)).color(theme.text_dim).size(12.0));
        ui.label(RichText::new(value).color(theme.text).size(12.0).strong());
    });
}
