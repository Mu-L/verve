//! Document-sharing actions: open the share-config dialog (local mode),
//! build share URLs, and delete a share config.

use gpui::{img, *};
use gpui::prelude::FluentBuilder as _;
use gpui_component::{ActiveTheme, Sizable as _, WindowExt as _, button::{Button, ButtonVariants as _}, h_flex, v_flex};
use crate::share::models::ShareScope;
use crate::ui::share_dialog;
use super::widgets::show_share_result_dialog;
use super::VerveApp;

impl VerveApp {
    pub(super) fn open_share_dialog(
        &mut self,
        scope: ShareScope,
        target_id: Option<String>,
        target_name: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let state = self.state.clone();
        let configs_store = self.share_configs.clone();
        let share_host = self.share_host.clone();
        let share_port = self.share_port;
        // The confirm handler needs `this` to start/refresh the server + panel,
        // but `this` isn't movable into `open_dialog`. We close over the pieces
        // we need (the config store) and do the panel refresh via cx.notify.
        share_dialog::open_dialog(
            state.clone(),
            scope,
            target_id,
            target_name,
            move |cfg, window, cx: &mut App| {
                log::info!("创建分享: id={} title={}", cfg.id, cfg.display_title());

                // Local mode: persist + serve from embedded server.
                let all = crate::share::persist::upsert_share(cfg.clone());
                if let Ok(mut guard) = configs_store.write() {
                    *guard = all;
                }

                let url = format!("http://{share_host}:{share_port}/s/{}", cfg.id);
                show_share_result_dialog(window, cx, cfg, url);
            },
            window,
            cx,
        );
    }

    pub(super) fn build_share_url(&self, id: &str) -> String {
        format!("http://{}:{}/s/{}", self.share_host, self.share_port, id)
    }

    pub(super) fn delete_share(&mut self, id: String, cx: &mut Context<Self>) {
        let all = crate::share::persist::remove_share(&id);
        log::info!("删除分享 {id}，剩余 {} 个", all.len());
        if let Ok(mut guard) = self.share_configs.write() {
            *guard = all;
        }
        // Refresh the share panel so the row disappears.
        self.share.update(cx, |s, cx| s.reload(cx));
        cx.notify();
    }
}
