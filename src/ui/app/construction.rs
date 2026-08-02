//! VerveApp construction: the `new()` constructor and the manual update flow.
//! All app-wide wiring (panels, git, share server, subscriptions) happens here.

use gpui::{img, *};
use gpui::prelude::FluentBuilder as _;
use gpui_component::{ActiveTheme, Sizable as _, WindowExt as _, select::{SelectEvent, SelectState}};
use gpui_component::resizable::ResizableState;
use crate::git::GitState;
use crate::share::server::{self, ShareServer};
use crate::state::{AppEvent, AppState};
use crate::ui::console_panel::{ConsoleEvent, ConsolePanel};
use crate::ui::mock_console_panel::{MockConsoleEvent, MockConsolePanel};
use crate::ui::project_tree_panel::ProjectTreePanel;
use crate::ui::hosts_panel::HostsPanel;
use crate::ui::json_panel::JsonPanel;
use crate::ui::proxy_panel::ProxyPanel;
use crate::ui::request_panel::RequestPanel;
use crate::ui::response_panel::ResponsePanel;
use crate::ui::project_manage_panel::ProjectManagePanel;
use crate::share::models::ShareScope;
use crate::ui::share_panel::{ShareEvent, SharePanel};
use super::{SideView, VerveApp};

impl VerveApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = AppState::global(cx);
        let tree = cx.new(|cx| ProjectTreePanel::new(state.clone(), window, cx));
        let request = cx.new(|cx| RequestPanel::new(state.clone(), window, cx));
        let response = cx.new(|cx| ResponsePanel::new(state.clone(), window, cx));
        let console = cx.new(|cx| ConsolePanel::new(state.clone(), cx));
        let mock_console = cx.new(|cx| MockConsolePanel::new(state.clone(), cx));

        // Environment switcher.
        let env_names: Vec<String> = state
            .read(cx)
            .active_project()
            .map(|p| {
                let mut v = vec!["No environment".to_string()];
                v.extend(p.environments.iter().map(|e| e.name.clone()));
                v
            })
            .unwrap_or_default();
        let env_select = cx.new(|cx| {
            SelectState::new(
                env_names,
                Some(gpui_component::IndexPath::default()),
                window,
                cx,
            )
        });

        // Project switcher: one entry per project in the workspace.
        let project_names: Vec<String> = state
            .read(cx)
            .data
            .projects
            .iter()
            .map(|p| p.name.clone())
            .collect();
        let project_select = cx.new(|cx| {
            SelectState::new(
                project_names.clone(),
                Some(gpui_component::IndexPath::default()),
                window,
                cx,
            )
        });

        // Resizable panel groups. Sizes are restored on render from the saved
        // layout via the panel `.size()` builder; `on_resize` persists changes.
        let main_resize = cx.new(|_| ResizableState::default());
        let center_resize = cx.new(|_| ResizableState::default());

        // Git version-control state. Initialised at the workspace data dir
        // (`~/.verve`); loads persisted auth/remote/auto-push config first.
        let data_dir =
            crate::state::persistence::data_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let git = GitState::init(data_dir.clone(), cx);
        {
            // Ensure the git repo exists (pure-local init, no remote/token
            // needed) and the .gitignore excludes cross-branch files, so every
            // workspace gets its own branch from the start.
            if let Ok(repo) = crate::git::ops::ensure_repo(&data_dir) {
                crate::state::persistence::ensure_gitignore();
                // Checkout the active workspace's branch (create if missing).
                let ws_idx = crate::state::persistence::load_workspaces_index();
                if let Some(branch) = ws_idx.active_meta().map(|m| m.branch.clone()) {
                    let no_auth = crate::git::ops::GitAuth::default();
                    if let Err(e) = crate::git::ops::create_or_checkout(&repo, &branch, &no_auth) {
                        log::warn!("启动时切换到 workspace 分支 {branch} 失败: {e}");
                    }
                }
            }
            let cfg = crate::state::persistence::load_git_config();
            let sync_minutes = cfg
                .sync_interval_minutes
                .unwrap_or(crate::state::persistence::DEFAULT_SYNC_INTERVAL_MINUTES);
            let has_remote = cfg.remote.is_some();
            git.update(cx, |g, cx| {
                g.load_config(
                    cfg.auto_commit,
                    cfg.auto_push,
                    cfg.remote.clone(),
                    cfg.username.clone(),
                    cfg.token.clone(),
                );
                // Trigger an initial refresh now that config is loaded.
                g.refresh_async(cx);
                // On startup, if a remote is configured, pull latest changes.
                if has_remote {
                    g.pull_async(cx);
                }
                // Auto-sync on the configured interval: commit dirty changes +
                // push/pull so work is versioned without manual intervention.
                g.start_auto_sync(
                    cx,
                    std::time::Duration::from_secs((sync_minutes as u64) * 60),
                );
            });
        }
        // Reload workspace data from the now-correct branch.
        state.update(cx, |s, _cx| {
            s.reload_from_disk(crate::state::persistence::load_workspaces_index().active);
        });
        let project_manage =
            cx.new(|cx| ProjectManagePanel::new(git.clone(), state.clone(), window, cx));
        let share = cx.new(|cx| SharePanel::new(state.clone(), window, cx));
        let proxy = cx.new(|cx| ProxyPanel::new(window, cx));
        let hosts = cx.new(|cx| HostsPanel::new(state.clone(), window, cx));
        let json = cx.new(|cx| JsonPanel::new(window, cx));

        // Document-sharing: load configs and start the local HTTP server if any
        // shares already exist. The server resolves projects from disk
        // (`workspace.json`) at request time so edits show up live without a
        // restart — the app persists to disk on a debounce, so a fresh read
        // always reflects current edits.
        let share_configs = server::config_store(crate::share::persist::load_shares());

        // Build initial mock rules from active project.
        let initial_mock: Vec<crate::mock::RuleEntry> = state
            .read(cx)
            .active_project()
            .map(|p| {
                std::sync::Arc::try_unwrap(crate::mock::rule_map(p))
                    .unwrap_or_else(|arc| (*arc).clone())
            })
            .unwrap_or_default();
        let mock_shared = crate::mock::shared_rules(initial_mock);
        let mock_shared_for_state = mock_shared.clone();
        state.update(cx, move |s, _| {
            s.mock_rules = Some(mock_shared_for_state);
        });

        let share_server = Some(server::start_desktop(
            crate::share::server::DEFAULT_PORT,
            share_configs.clone(),
            |project_id| {
                let data = crate::state::persistence::load_or_default();
                data.projects.iter().find(|p| p.id == project_id).cloned()
            },
            Some(mock_shared),
        ));

        let mut app = Self {
            state: state.clone(),
            tree,
            request,
            response,
            console,
            mock_console,
            git,
            project_manage,
            share,
            proxy,
            hosts,
            json,
            share_server,
            share_configs,
            share_host: "127.0.0.1".to_string(),
            share_port: crate::share::server::DEFAULT_PORT,
            env_select,
            project_select,
            show_console: false,
            active_view: SideView::Api,
            home_view: crate::state::persistence::load_layout()
                .and_then(|l| l.home_view.as_deref().map(SideView::parse))
                .unwrap_or(SideView::Api),
            pending_switcher_refresh: false,
            sidebar_collapsed: false,
            rail_collapsed: false,
            hidden_rails: crate::state::persistence::load_hidden_rails(),
            theme_popover_open: false,
            lang_popover_open: false,
            export_popover_open: false,
            project_popover_open: false,
            workspace_popover_open: false,
            env_popover_open: false,
            pending_new_project: false,
            pending_new_workspace: false,
            pending_workspace_name_input: None,
            pending_new_env: false,
            pending_dialog: None,
            pending_share: None,
            applied_sync_interval: crate::state::persistence::DEFAULT_SYNC_INTERVAL_MINUTES,
            main_resize: main_resize.clone(),
            center_resize: center_resize.clone(),
            saved_layout: crate::state::persistence::load_layout(),
            update_info: None,
            update_checking: false,
            update_check_result: None,
            rail_order: {
                let saved = crate::state::persistence::load_rail_order();
                let default: Vec<String> =
                    SideView::ALL.iter().map(|v| v.name().to_string()).collect();
                match saved {
                    None => default,
                    Some(mut order) => {
                        let existing: std::collections::HashSet<_> =
                            order.iter().cloned().collect();
                        // Append new default items missing from saved order
                        for name in &default {
                            if !existing.contains(name) {
                                order.push(name.clone());
                            }
                        }
                        // Remove items no longer in defaults
                        let default_set: std::collections::HashSet<_> =
                            default.into_iter().collect();
                        order.retain(|n| default_set.contains(n));
                        order
                    }
                }
            },
            dragging_rail: None,
            rail_drop_target: None,
            _subs: Vec::new(),
        };

        // Persist on workspace edits (debounced by coalescing).
        let sub_edited = cx.subscribe(&state, |this, _src, ev: &AppEvent, cx| {
            match ev {
                // Structural changes persist immediately and may have renamed a
                // project / changed environments, so the switchers must rebuild.
                AppEvent::WorkspaceChanged => {
                    this.state.update(cx, |s, cx| s.persist(cx));
                    this.pending_switcher_refresh = true;
                    cx.notify();
                }
                // A workspace switch reloaded the data; refresh everything.
                AppEvent::WorkspaceSwitched => {
                    this.pending_switcher_refresh = true;
                    cx.notify();
                }
                // Field edits use a debounced save (scheduled in notify_edited).
                AppEvent::RequestEdited => {}
                // A persist just landed on disk — kick a git auto-commit (and
                // optional auto-push) so changes are versioned without the user
                // having to think about it. The Persisted event already coalesces
                // with the 1s debounce, so we don't double-commit.
                AppEvent::Persisted => {
                    let git = this.git.clone();
                    let _ = git.update(cx, |g, cx| {
                        if g.initialized && g.auto_commit && !g.is_busy() {
                            log::info!("Persisted → 自动同步触发 (auto_commit=true)");
                            g.sync_async(None, cx);
                        } else if g.is_busy() {
                            log::info!("Persisted → 已有同步进行中，跳过自动同步");
                        } else if !g.initialized {
                            log::info!("Persisted → git 未初始化，跳过自动同步");
                        }
                    });
                }
                // Share a single API: resolve the request's name, then defer
                // the dialog open to render (where a Window is available).
                AppEvent::ShareRequest(id) => {
                    let (target_name, found) = this
                        .state
                        .read(cx)
                        .active_project()
                        .and_then(|p| p.find_request(id).map(|(_, r)| r.name.clone()))
                        .map(|n| (Some(n), true))
                        .unwrap_or((None, false));
                    if found {
                        this.pending_share = Some((
                            crate::share::models::ShareScope::Request,
                            Some(id.clone()),
                            target_name,
                        ));
                        cx.notify();
                    }
                }
                _ => {}
            }
        });

        // Env switcher: update the active environment when the user picks one.
        let env_select_handle = app.env_select.clone();
        let sub_env = cx.subscribe(
            &app.env_select,
            move |this, _src, _ev: &SelectEvent<Vec<String>>, cx| {
                let chosen = env_select_handle.read(cx).selected_value().cloned();
                this.state.update(cx, |s, cx| {
                    if let Some(project) = s.active_project_mut() {
                        // "No environment" (index 0) maps to None; otherwise match by name.
                        let new_id = match chosen.as_deref() {
                            Some(name) if name != "No environment" => project
                                .environments
                                .iter()
                                .find(|e| e.name == name)
                                .map(|e| e.id.clone()),
                            _ => None,
                        };
                        project.active_environment = new_id;
                        cx.emit(AppEvent::EnvironmentChanged);
                    }
                });
            },
        );

        // Project switcher: change the active project when the user picks one.
        let project_select_handle = app.project_select.clone();
        let sub_project = cx.subscribe(
            &app.project_select,
            move |this, _src, _ev: &SelectEvent<Vec<String>>, cx| {
                let chosen = project_select_handle.read(cx).selected_value().cloned();
                this.state.update(cx, |s, cx| {
                    if let Some(name) = chosen {
                        if let Some(idx) = s.data.projects.iter().position(|p| p.name == name) {
                            if s.active_project != idx {
                                s.active_project = idx;
                                s.data.active_project_id =
                                    s.data.projects.get(idx).map(|p| p.id.clone());
                                s.selected_request = None;
                                s.selected_folder = None;
                                s.open_request_ids.clear();
                                s.active_tab_id = None;
                                cx.emit(AppEvent::WorkspaceChanged);
                                cx.emit(AppEvent::SelectionChanged);
                            }
                        }
                    }
                });
            },
        );

        // Re-render when git state changes (busy flag, dirty count, branches)
        // so the title-bar sync button tooltip stays current.
        let sub_git = cx.subscribe(&app.git, |this, _src, _ev: &crate::git::GitEvent, cx| {
            cx.notify();
            let _ = this;
        });

        // The management panel emits import/export requests upward (it can't
        // call VerveApp's file-prompt methods directly).
        let sub_manage = cx.subscribe(
            &app.project_manage,
            |this, _src, ev: &crate::ui::project_manage_panel::ManageEvent, cx| match ev {
                crate::ui::project_manage_panel::ManageEvent::Import => {
                    this.import_collection(cx);
                }
                crate::ui::project_manage_panel::ManageEvent::Export(fmt) => {
                    this.export_project(*fmt, cx);
                }
                crate::ui::project_manage_panel::ManageEvent::GenerateMocks => {
                    this.generate_mocks(cx);
                }
            },
        );

        // The share panel emits NewShare / Open / Copy / Delete requests upward.
        let sub_share = cx.subscribe(&app.share, |this, _src, ev: &ShareEvent, cx| {
            match ev {
                ShareEvent::NewShare => {
                    // Opening the dialog needs a Window; defer to render.
                    this.pending_share = Some((ShareScope::Project, None, None));
                    cx.notify();
                }
                ShareEvent::Open(id) => {
                    let url = this.build_share_url(id);
                    cx.open_url(&url);
                }
                ShareEvent::Copy(id) => {
                    let url = this.build_share_url(id);
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(url));
                }
                ShareEvent::Delete(id) => {
                    this.delete_share(id.clone(), cx);
                }
            }
        });

        // The console (history) panel emits OpenRequest when the user clicks a row.
        let console_handle = app.console.clone();
        let sub_console = cx.subscribe(
            &console_handle,
            |this, _src, ev: &ConsoleEvent, cx| match ev {
                ConsoleEvent::OpenRequest {
                    request_id: Some(req_id),
                    project_id,
                } => {
                    // Switch project if the history entry belongs to a different project.
                    let current_pid = this
                        .state
                        .read(cx)
                        .active_project()
                        .map(|p| p.id.clone())
                        .unwrap_or_default();
                    if !project_id.is_empty() && &current_pid != project_id {
                        if let Some(idx) = this
                            .state
                            .read(cx)
                            .data
                            .projects
                            .iter()
                            .position(|p| &p.id == project_id)
                        {
                            this.state.update(cx, |s, cx| {
                                s.active_project = idx;
                                s.selected_request = None;
                                s.selected_folder = None;
                                s.open_request_ids.clear();
                                s.active_tab_id = None;
                                cx.emit(AppEvent::WorkspaceChanged);
                            });
                        }
                    }

                    // Switch to API view, select the request, ask the tree to reveal it.
                    let exists = this
                        .state
                        .read(cx)
                        .active_project()
                        .map(|p| p.find_request(req_id).is_some())
                        .unwrap_or(false);
                    if exists {
                        this.active_view = SideView::Api;
                        let req_id_emit = req_id.clone();
                        this.state.update(cx, |s, cx| {
                            s.open_or_focus_tab(&req_id_emit, cx);
                            cx.emit(AppEvent::LocateActive);
                        });
                        // Collapse the bottom dock so the request/response is visible.
                        this.show_console = false;
                    }
                    cx.notify();
                }
                ConsoleEvent::OpenRequest {
                    request_id: None, ..
                } => {
                    // Legacy entries without a request id: nothing to navigate to.
                }
            },
        );

        // Subscribe to mock console events (GenerateAll).
        let mock_console_handle = app.mock_console.clone();
        let sub_mock = cx.subscribe(
            &mock_console_handle,
            |this, _src, ev: &MockConsoleEvent, cx| match ev {
                MockConsoleEvent::GenerateAll => {
                    this.generate_mocks(cx);
                }
            },
        );

        // Subscribe to tree events (share request).
        let tree_handle = app.tree.clone();
        let sub_tree = cx.subscribe(
            &tree_handle,
            |this, _src, ev: &crate::ui::project_tree_panel::TreeEvent, _cx| match ev {
                crate::ui::project_tree_panel::TreeEvent::ShareRequest(id, name) => {
                    this.pending_share = Some((
                        crate::share::models::ShareScope::Request,
                        Some(id.clone()),
                        Some(name.clone()),
                    ));
                }
            },
        );

        app._subs = vec![
            sub_edited,
            sub_env,
            sub_project,
            sub_git,
            sub_manage,
            sub_share,
            sub_console,
            sub_mock,
            sub_tree,
        ];

        // Silent startup update check: query GitHub for the latest release
        // after a short delay. On success, store the result so the title-bar
        // button can show a red dot — no notification is shown (to avoid
        // bothering the user on every launch).
        app.startup_update_check(cx);

        app
    }
}

impl VerveApp {
    pub(super) fn startup_update_check(&mut self, cx: &mut Context<Self>) {
        let client = cx.http_client();
        cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(3))
                .await;
            let result = crate::updater::run_check(client).await;
            let _ = this.update(cx, |this, cx| {
                if let crate::updater::UpdateCheckResult::UpdateAvailable(info) = &result {
                    this.update_info = Some(info.clone());
                    this.update_check_result = Some(result);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn check_updates_manual(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if self.update_checking {
            return;
        }
        self.update_checking = true;
        cx.notify();

        let client = cx.http_client();
        cx.spawn(async move |this, cx| {
            let result = crate::updater::run_check(client).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.update_checking = false;
                this.update_check_result = Some(result.clone());
                match &result {
                    crate::updater::UpdateCheckResult::UpdateAvailable(info) => {
                        this.update_info = Some(info.clone());
                        let msg =
                            format!("发现新版本 v{}！点击标题栏下载按钮前往下载。", info.version);
                        window.push_notification(
                            gpui_component::notification::Notification::new()
                                .title("发现新版本")
                                .message(msg)
                                .autohide(true),
                            cx,
                        );
                    }
                    crate::updater::UpdateCheckResult::UpToDate => {
                        window.push_notification(
                            gpui_component::notification::Notification::new()
                                .title("检查更新")
                                .message("当前已是最新版本。")
                                .autohide(true),
                            cx,
                        );
                    }
                    crate::updater::UpdateCheckResult::Error(e) => {
                        window.push_notification(
                            gpui_component::notification::Notification::new()
                                .title("检查更新失败")
                                .message(e.clone())
                                .autohide(true),
                            cx,
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn open_update_download(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(info) = &self.update_info {
            // Prefer the direct download URL; fall back to the release page.
            let url = info.download_url.as_deref().unwrap_or(&info.release_url);
            cx.open_url(url);
        }
    }
}
