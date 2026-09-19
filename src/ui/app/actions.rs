//! Keybinding action handlers. Wired via `on_action` in the root Render.
//!
//! Also rail-view switching (⌘/Ctrl + 1..=5).

use gpui::{img, *};
use super::{CloseFile, SaveWorkspace, SelectRailSlot, SideView, VerveApp};

impl VerveApp {
    /// cmd-s — save. Persists the workspace to disk immediately (bypassing the
    /// debounce), so changes survive a crash or restart.
    pub(super) fn on_save_workspace(
        &mut self,
        _: &SaveWorkspace,
        _w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.state.update(cx, |s, cx| s.persist(cx));
        cx.notify();
    }

    /// ⌘/Ctrl + 1..=5 — switch to one of the fixed shortcut views. The
    /// bindings carry a "!BlockEditor" context so deeper-focused bindings win
    /// while a block editor is focused. Hidden rails are skipped.
    pub(super) fn on_select_rail_slot(
        &mut self,
        a: &SelectRailSlot,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(&view) = SideView::SHORTCUT_VIEWS.get(a.0) else {
            return;
        };
        if self.hidden_rails.contains(view.name()) {
            return;
        }
        self.activate_view(view, w, cx);
    }

    /// cmd-w — close the active file/tab: Api → close the active request tab;
    /// Ssh → close the active terminal session. (The tab-overflow popover also
    /// dispatches `CloseFile`, but it has its own `on_overflow_close_tab`
    /// handler on the popover's focus target, so this root handler only fires
    /// when focus is in the main tree.)
    pub(super) fn on_close_file(
        &mut self,
        _: &CloseFile,
        _w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match self.active_view {
            SideView::Api => {
                let _ = self.state.update(cx, |s, cx| {
                    if let Some(id) = s.active_tab_id.clone() {
                        s.close_tab(&id, cx);
                    }
                });
            }
            SideView::Ssh => {
                let _ = self.ssh.update(cx, |p, cx| {
                    // Read the index into a local first to avoid a borrow of
                    // `p` across the `&mut p` close call.
                    let idx = p.active_tab;
                    p.close_session(idx, cx);
                });
            }
            _ => {}
        }
        cx.notify();
    }
}
