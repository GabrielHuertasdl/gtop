use std::sync::{Arc, RwLock};
use std::time::Duration;

use gtop_core::metrics::MetricsCollector;
use gtop_core::process::ProcessCollector;
use gtop_core::state::AppState;

pub type SharedState = Arc<RwLock<AppState>>;

pub fn spawn_collector(state: SharedState) {
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");

        rt.block_on(async move {
            let Ok(mut metrics_collector) = MetricsCollector::new() else {
                tracing::error!("Failed to initialize metrics collector");
                return;
            };
            let mut proc_collector = ProcessCollector::new();
            let mut interval = tokio::time::interval(Duration::from_millis(1000));

            loop {
                interval.tick().await;

                let metrics = metrics_collector.collect().ok();
                let tree = proc_collector.collect();

                if let Ok(mut st) = state.write() {
                    if let Some(m) = metrics {
                        st.update_metrics(m);
                    }
                    st.update_processes(tree);
                }
            }
        });
    });
}

pub fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}
