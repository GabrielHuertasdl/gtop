use egui::{Align, Color32, Layout, Margin, RichText, Ui};

use gtop_core::state::AppState;
use gtop_core::util::{format_bytes, format_speed, format_uptime};

use crate::theme::GuiTheme;

pub fn render(ui: &mut Ui, state: &AppState, theme: &GuiTheme) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(8.0, 4.0);

        // CPU card
        let cpu = state.metrics.cpu.global_pct;
        metric_card(
            ui,
            theme,
            "CPU",
            &format!("{:.1}%", cpu),
            &format!(
                "load {:.2}  up {}",
                state.metrics.cpu.load_1m,
                format_uptime(state.metrics.cpu.uptime_secs)
            ),
            cpu / 100.0,
            theme.cpu_color(cpu),
        );

        // RAM card
        let mem = &state.metrics.memory;
        metric_card(
            ui,
            theme,
            "RAM",
            &format!("{}/{}", format_bytes(mem.used), format_bytes(mem.total)),
            &format!("{:.1}% used  avail {}", mem.used_pct(), format_bytes(mem.available)),
            mem.used_pct() / 100.0,
            theme.mem_color,
        );

        // Swap card (only if swap exists)
        if mem.swap_total > 0 {
            metric_card(
                ui,
                theme,
                "SWP",
                &format!("{}/{}", format_bytes(mem.swap_used), format_bytes(mem.swap_total)),
                &format!("{:.1}% used", mem.swap_pct()),
                mem.swap_pct() / 100.0,
                theme.swap_color,
            );
        }

        // Network card
        if let Some(net) = state.metrics.networks.first() {
            net_card(ui, theme, &net.interface, net.rx_bytes_per_sec, net.tx_bytes_per_sec);
        }

        // Temperature card
        if let Some(temp) = state
            .metrics
            .temperatures
            .iter()
            .find(|t| t.label.to_lowercase().contains("cpu") || t.device.to_lowercase().contains("cpu"))
        {
            temp_card(ui, theme, &temp.label, temp.temp_celsius);
        }

        // Battery card
        if let Some(bat) = &state.metrics.battery {
            bat_card(ui, theme, bat.capacity_pct, &bat.status);
        }

        // GPU card
        if let Some(gpu) = &state.metrics.gpu {
            if let Some(pct) = gpu.utilization_pct {
                metric_card(
                    ui,
                    theme,
                    "GPU",
                    &format!("{:.0}%", pct),
                    &gpu.freq_mhz.map(|f| format!("{}MHz", f)).unwrap_or_default(),
                    pct / 100.0,
                    Color32::from_rgb(168, 85, 247),
                );
            }
        }
    });
}

fn card_frame(theme: &GuiTheme) -> egui::Frame {
    egui::Frame::new()
        .fill(theme.surface)
        .corner_radius(8.0)
        .inner_margin(Margin::symmetric(12, 8))
        .stroke(egui::Stroke::new(1.0, theme.border))
}

fn metric_card(
    ui: &mut Ui,
    theme: &GuiTheme,
    title: &str,
    value: &str,
    subtitle: &str,
    pct: f32,
    color: Color32,
) {
    card_frame(theme).show(ui, |ui| {
        ui.set_min_width(155.0);
        ui.set_max_width(240.0);

        // Title row: label on left, value on right
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(title)
                    .color(theme.text_dim)
                    .size(11.0),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(value)
                        .color(theme.text_strong)
                        .strong()
                        .size(14.0),
                );
            });
        });

        ui.add_space(4.0);

        // Progress bar
        ui.add(
            egui::ProgressBar::new(pct.clamp(0.0, 1.0))
                .fill(color)
                .desired_height(5.0),
        );

        ui.add_space(3.0);
        ui.label(RichText::new(subtitle).color(theme.text_dim).size(10.0));
    });
}

fn net_card(ui: &mut Ui, theme: &GuiTheme, iface: &str, rx: u64, tx: u64) {
    card_frame(theme).show(ui, |ui| {
        ui.set_min_width(140.0);
        ui.set_max_width(200.0);

        ui.label(
            RichText::new(format!("NET  {}", iface))
                .color(theme.text_dim)
                .size(11.0),
        );

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("↓ {}", format_speed(rx)))
                    .color(theme.net_rx)
                    .strong()
                    .size(12.0),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("↑ {}", format_speed(tx)))
                    .color(theme.net_tx)
                    .strong()
                    .size(12.0),
            );
        });

        let total = rx.saturating_add(tx);
        let rx_pct = if total > 0 { rx as f32 / total as f32 } else { 0.0 };
        ui.add_space(3.0);
        ui.label(RichText::new(format!("rx {:.0}%", rx_pct * 100.0)).color(theme.text_dim).size(10.0));
    });
}

fn temp_card(ui: &mut Ui, theme: &GuiTheme, label: &str, celsius: f32) {
    let color = theme.temp_color(celsius);
    card_frame(theme).show(ui, |ui| {
        ui.set_min_width(90.0);
        ui.set_max_width(130.0);

        ui.label(RichText::new("TEMP").color(theme.text_dim).size(11.0));
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("{:.0}°C", celsius))
                .color(color)
                .strong()
                .size(16.0),
        );
        ui.add_space(2.0);
        ui.label(RichText::new(label).color(theme.text_dim).size(10.0));
    });
}

fn bat_card(ui: &mut Ui, theme: &GuiTheme, pct: u8, status: &gtop_core::metrics::BatteryStatus) {
    let color = theme.bat_color(pct as u32);
    let arrow = match status {
        gtop_core::metrics::BatteryStatus::Charging => " ↑",
        gtop_core::metrics::BatteryStatus::Discharging => " ↓",
        _ => "",
    };
    card_frame(theme).show(ui, |ui| {
        ui.set_min_width(90.0);
        ui.set_max_width(130.0);

        ui.horizontal(|ui| {
            ui.label(RichText::new("BAT").color(theme.text_dim).size(11.0));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{}%{}", pct, arrow))
                        .color(color)
                        .strong()
                        .size(14.0),
                );
            });
        });
        ui.add_space(4.0);
        ui.add(
            egui::ProgressBar::new(pct as f32 / 100.0)
                .fill(color)
                .desired_height(5.0),
        );
    });
}
