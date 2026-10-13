mod app;
mod bridge;
mod panels;
mod theme;

use std::sync::{Arc, RwLock};

use anyhow::Result;
use clap::Parser;
use eframe::NativeOptions;

use gtop_core::state::AppState;

use crate::bridge::{num_cpus, spawn_collector};

#[derive(Parser, Debug)]
#[command(name = "gtop-gui", about = "Real-time process manager GUI", version)]
struct Args {
    /// Initial window width
    #[arg(long, default_value_t = 1200)]
    width: u32,

    /// Initial window height
    #[arg(long, default_value_t = 800)]
    height: u32,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();

    // Shared state between background collector and GUI
    let state = Arc::new(RwLock::new(AppState::new(num_cpus())));

    // Spawn background data collection thread
    spawn_collector(Arc::clone(&state));

    // Launch GUI
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Gtop - Process Manager")
            .with_inner_size([args.width as f32, args.height as f32])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "gtop-gui",
        native_options,
        Box::new(move |cc| {
            Ok(Box::new(app::GuiApp::new(cc, Arc::clone(&state))))
        }),
    )
    .map_err(|e| anyhow::anyhow!("GUI error: {}", e))?;

    Ok(())
}
