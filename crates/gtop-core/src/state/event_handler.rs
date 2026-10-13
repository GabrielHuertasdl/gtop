use crate::ipc::AppEvent;
use crate::process::filter::SortColumn;
use crate::state::app_state::{ActionKind, AppState, PendingAction, ViewMode};

#[derive(Debug)]
pub enum SideEffect {
    ExecuteAction(PendingAction),
    Quit,
}

pub fn handle_event(state: &mut AppState, event: AppEvent) -> Option<SideEffect> {
    match event {
        AppEvent::CursorUp => {
            state.move_cursor(-1);
        }
        AppEvent::CursorDown => {
            state.move_cursor(1);
        }
        AppEvent::PageUp => {
            state.move_cursor(-20);
        }
        AppEvent::PageDown => {
            state.move_cursor(20);
        }
        AppEvent::CursorTop => {
            state.cursor_index = 0;
            state.focused_pid = state.processes.first().map(|p| p.pid);
        }
        AppEvent::CursorBottom => {
            let len = state.processes.len();
            if len > 0 {
                state.cursor_index = len - 1;
                state.focused_pid = state.processes.last().map(|p| p.pid);
            }
        }
        AppEvent::SetCursor(idx) => {
            if idx < state.processes.len() {
                state.cursor_index = idx;
                state.focused_pid = state.processes.get(idx).map(|p| p.pid);
            }
        }

        AppEvent::ToggleSelect => {
            if let Some(pid) = state.focused_pid {
                if state.selected_pids.contains(&pid) {
                    state.selected_pids.remove(&pid);
                } else {
                    state.selected_pids.insert(pid);
                }
                state.move_cursor(1);
            }
        }
        AppEvent::SelectAll => {
            state.selected_pids = state.processes.iter().map(|p| p.pid).collect();
        }
        AppEvent::DeselectAll => {
            state.selected_pids.clear();
        }
        AppEvent::OpenDetail | AppEvent::ToggleDetail => {
            state.detail_visible = !state.detail_visible;
        }
        AppEvent::CloseDetail => {
            state.detail_visible = false;
        }

        AppEvent::RequestKill => {
            return build_pending(state, ActionKind::Kill);
        }
        AppEvent::RequestTerminate => {
            return build_pending(state, ActionKind::Terminate);
        }
        AppEvent::RequestPause => {
            return build_pending(state, ActionKind::Pause);
        }
        AppEvent::RequestResume => {
            return build_pending(state, ActionKind::Resume);
        }
        AppEvent::RequestRenice => {
            state.renice_input.clear();
            return build_pending(state, ActionKind::Renice(0));
        }
        AppEvent::RequestSignal(sig) => {
            return build_pending(state, ActionKind::Signal(sig));
        }

        AppEvent::SetReniceValue(v) => {
            if let Some(ref mut pa) = state.pending_action {
                pa.kind = ActionKind::Renice(v);
            }
        }
        AppEvent::Confirm => {
            let action = state.pending_action.take()?;
            return Some(SideEffect::ExecuteAction(action));
        }
        AppEvent::Cancel => {
            state.pending_action = None;
            state.renice_input.clear();
        }

        AppEvent::ToggleViewMode => {
            state.view_mode = match state.view_mode {
                ViewMode::List => ViewMode::Tree,
                ViewMode::Tree => ViewMode::List,
            };
        }
        AppEvent::ToggleKernelThreads => {
            state.filter.hide_kernel = !state.filter.hide_kernel;
            state.recompute_processes();
        }
        AppEvent::SetSort(col) => {
            if state.sort_column == col {
                state.sort_ascending = !state.sort_ascending;
            } else {
                state.sort_column = col;
                state.sort_ascending = false;
            }
            state.recompute_processes();
        }
        AppEvent::ToggleSortDirection => {
            state.sort_ascending = !state.sort_ascending;
            state.recompute_processes();
        }

        AppEvent::SetSearch(q) => {
            state.search = q;
            state.recompute_processes();
        }
        AppEvent::ClearSearch => {
            state.search.clear();
            state.search_active = false;
            state.recompute_processes();
        }
        AppEvent::SetFilter(f) => {
            state.filter = f;
            state.recompute_processes();
        }
        AppEvent::ClearFilter => {
            state.filter = Default::default();
            state.filter.hide_kernel = true;
            state.recompute_processes();
        }

        AppEvent::Quit => {
            return Some(SideEffect::Quit);
        }
        AppEvent::Refresh => {}
    }

    None
}

fn build_pending(state: &mut AppState, kind: ActionKind) -> Option<SideEffect> {
    let targets = state.effective_targets();
    if targets.is_empty() {
        return None;
    }
    let pids: Vec<_> = targets.iter().map(|p| p.pid).collect();
    let names: Vec<_> = targets.iter().map(|p| p.name.clone()).collect();
    state.pending_action = Some(PendingAction { kind, pids, names });
    None
}
