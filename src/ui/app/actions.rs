//! Keybinding action handlers. Wired via `on_action` in the root Render.

use gpui::{img, *};
use super::{SaveWorkspace, VerveApp};

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
}
