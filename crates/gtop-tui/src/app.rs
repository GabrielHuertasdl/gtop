use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use futures_util::StreamExt;
use ratatui::{backend::CrosstermBackend, Frame, Terminal};

use gtop_core::ipc::AppEvent;
use gtop_core::metrics::MetricsCollector;
use gtop_core::process::filter::SortColumn;
use gtop_core::process::{ProcessCollector, ProcessManager};
use gtop_core::state::app_state::{ActionKind, AppState};
use gtop_core::state::{handle_event, SideEffect};

use gtop_core::state::ViewMode;

use crate::layout::compute_layout;
use crate::theme::Theme;
use crate::widgets::{
    confirm_dialog, detail_panel, header, metrics_panel, process_table, process_tree, status_bar,
};

pub struct App {
    state: AppState,
    theme: Theme,
    table_state: ratatui::widgets::TableState,
}

impl App {
    pub fn new() -> Result<Self> {
        Self::with_theme(Theme::default(), false, false)
    }

    pub fn with_theme(theme: Theme, start_tree: bool, show_kernel: bool) -> Result<Self> {
        let mut state = AppState::new(num_cpus());
        if start_tree {
            state.view_mode = ViewMode::Tree;
        }
        if show_kernel {
            state.filter.hide_kernel = false;
        }
        Ok(App {
            state,
            theme,
            table_state: ratatui::widgets::TableState::default(),
        })
    }

    pub async fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        let mut metrics_collector = MetricsCollector::new()?;
        let mut proc_collector = ProcessCollector::new();

        // Initial collection
        if let Ok(m) = metrics_collector.collect() {
            self.state.update_metrics(m);
        }
        let tree = proc_collector.collect();
        self.state.update_processes(tree);

        let mut tick = tokio::time::interval(Duration::from_millis(1000));
        let mut event_stream = crossterm::event::EventStream::new();

        loop {
            terminal.draw(|f| self.render(f))?;

            tokio::select! {
                _ = tick.tick() => {
                    if let Ok(m) = metrics_collector.collect() {
                        self.state.update_metrics(m);
                    }
                    let tree = proc_collector.collect();
                    self.state.update_processes(tree);
                }
                maybe_event = event_stream.next() => {
                    let Some(Ok(event)) = maybe_event else { continue };
                    if let Some(effect) = self.handle_crossterm_event(event) {
                        match effect {
                            SideEffect::Quit => break,
                            SideEffect::ExecuteAction(action) => {
                                self.execute_action(action);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn render(&mut self, f: &mut Frame) {
        let layout = compute_layout(f.area(), self.state.detail_visible);

        header::render(f, layout.header, &self.state, &self.theme);
        metrics_panel::render(f, layout.metrics, &self.state, &self.theme, &layout.mode);

        match self.state.view_mode {
            gtop_core::state::ViewMode::List => {
                process_table::render(
                    f,
                    layout.main,
                    &self.state,
                    &self.theme,
                    &mut self.table_state,
                );
            }
            gtop_core::state::ViewMode::Tree => {
                process_tree::render(
                    f,
                    layout.main,
                    &self.state,
                    &self.theme,
                    &mut self.table_state,
                );
            }
        }

        if let Some(detail_area) = layout.detail {
            let proc = self.state.focused_pid
                .and_then(|pid| self.state.tree.by_pid.get(&pid));
            detail_panel::render(f, detail_area, proc, &self.theme);
        }

        status_bar::render(f, layout.statusbar, &self.state, &self.theme);

        if let Some(ref action) = self.state.pending_action.clone() {
            confirm_dialog::render(f, f.area(), action, &self.theme);
        }
    }

    fn handle_crossterm_event(&mut self, event: Event) -> Option<SideEffect> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.handle_key(key)
            }
            Event::Mouse(mouse) => {
                self.handle_mouse(mouse);
                None
            }
            Event::Resize(_, _) => None,
            _ => None,
        }
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> Option<SideEffect> {
        // Handle search input mode
        if self.state.search_active {
            match key.code {
                KeyCode::Esc => {
                    return dispatch(&mut self.state, AppEvent::ClearSearch);
                }
                KeyCode::Enter => {
                    self.state.search_active = false;
                    return None;
                }
                KeyCode::Backspace => {
                    self.state.search.pop();
                    let q = self.state.search.clone();
                    return dispatch(&mut self.state, AppEvent::SetSearch(q));
                }
                KeyCode::Char(c) => {
                    self.state.search.push(c);
                    let q = self.state.search.clone();
                    return dispatch(&mut self.state, AppEvent::SetSearch(q));
                }
                _ => {}
            }
            return None;
        }

        // Handle confirmation mode
        if self.state.pending_action.is_some() {
            match key.code {
                KeyCode::Enter | KeyCode::Char('y') => {
                    return dispatch(&mut self.state, AppEvent::Confirm);
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => {
                    return dispatch(&mut self.state, AppEvent::Cancel);
                }
                _ => {}
            }
            return None;
        }

        // Normal mode
        let event = match (key.code, key.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                AppEvent::Quit
            }
            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => AppEvent::CursorUp,
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => AppEvent::CursorDown,
            (KeyCode::PageUp, _) | (KeyCode::Char('u'), KeyModifiers::CONTROL) => AppEvent::PageUp,
            (KeyCode::PageDown, _) | (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                AppEvent::PageDown
            }
            (KeyCode::Char('g'), _) | (KeyCode::Home, _) => AppEvent::CursorTop,
            (KeyCode::Char('G'), _) | (KeyCode::End, _) => AppEvent::CursorBottom,
            (KeyCode::Char(' '), _) => AppEvent::ToggleSelect,
            (KeyCode::Char('a'), _) => AppEvent::SelectAll,
            (KeyCode::Char('A'), _) => AppEvent::DeselectAll,
            (KeyCode::Enter, _) | (KeyCode::Char('d'), _) => AppEvent::ToggleDetail,
            (KeyCode::Char('X'), _) => AppEvent::RequestKill,
            (KeyCode::Char('t'), _) => AppEvent::RequestTerminate,
            (KeyCode::Char('p'), _) => AppEvent::RequestPause,
            (KeyCode::Char('r'), _) => AppEvent::RequestResume,
            (KeyCode::Char('/'), _) => {
                self.state.search_active = true;
                return None;
            }
            (KeyCode::Char('T'), _) => AppEvent::ToggleViewMode,
            (KeyCode::Char('i'), _) => AppEvent::ToggleKernelThreads,
            (KeyCode::Char('I'), _) => AppEvent::ToggleSortDirection,
            (KeyCode::Char('1'), _) => AppEvent::SetSort(SortColumn::Pid),
            (KeyCode::Char('2'), _) => AppEvent::SetSort(SortColumn::User),
            (KeyCode::Char('3'), _) => AppEvent::SetSort(SortColumn::CpuPct),
            (KeyCode::Char('4'), _) => AppEvent::SetSort(SortColumn::MemPct),
            (KeyCode::Char('5'), _) => AppEvent::SetSort(SortColumn::MemBytes),
            (KeyCode::Char('6'), _) => AppEvent::SetSort(SortColumn::Nice),
            (KeyCode::Char('7'), _) => AppEvent::SetSort(SortColumn::Status),
            (KeyCode::Char('8'), _) => AppEvent::SetSort(SortColumn::Name),
            (KeyCode::F(5), _) => AppEvent::Refresh,
            _ => return None,
        };

        dispatch(&mut self.state, event)
    }

    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                dispatch(&mut self.state, AppEvent::CursorUp);
            }
            MouseEventKind::ScrollDown => {
                dispatch(&mut self.state, AppEvent::CursorDown);
            }
            _ => {}
        }
    }

    fn execute_action(&mut self, action: gtop_core::state::app_state::PendingAction) {
        use nix::sys::signal::Signal;

        for (&pid, name) in action.pids.iter().zip(action.names.iter()) {
            // PID reuse guard
            if !ProcessManager::verify_process(pid, name) {
                tracing::warn!("Process {} (PID {}) no longer exists or was replaced", name, pid);
                continue;
            }

            let result = match &action.kind {
                ActionKind::Kill => ProcessManager::kill(pid),
                ActionKind::Terminate => ProcessManager::terminate(pid),
                ActionKind::Pause => ProcessManager::pause(pid),
                ActionKind::Resume => ProcessManager::resume(pid),
                ActionKind::Renice(v) => ProcessManager::renice(pid, *v),
                ActionKind::Signal(sig) => ProcessManager::send_signal(pid, *sig),
            };

            if let Err(e) = result {
                tracing::error!("Action {:?} on PID {} failed: {}", action.kind.label(), pid, e);
            }
        }

        // Clear selection after batch action
        self.state.selected_pids.clear();
    }
}

fn dispatch(state: &mut AppState, event: AppEvent) -> Option<SideEffect> {
    handle_event(state, event)
}

fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}
