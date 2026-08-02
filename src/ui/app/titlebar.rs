//! All view title bars: the API workbench bar (workspace/project/env
//! switchers, sync, import/export, update, language), the shared rail
//! toggle / update / export / language pickers, and the per-view bars
//! (JSON).

use gpui::{img, *};
use gpui::prelude::FluentBuilder as _;
use gpui_component::{ActiveTheme, Icon, IconName, Sizable as _, Selectable as _, Disableable as _,
    button::{Button, ButtonVariants as _}, h_flex, v_flex, popover::Popover, WindowExt};
use crate::assets::{BRACES, BRACES_JSON, DOCS, EXPORT,
    HISTORY, IMPORT, REFRESH_CW, SAVE,
    SAVE_AS, SERVER, SHARE};
use crate::share::models::ShareScope;
use crate::state::{AppEvent, AppState};
use super::widgets::{menu_item, menu_separator, vicon};
use super::{PendingDialog, SideView, VerveApp};

impl VerveApp {
    pub(super) fn render_title_bar(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        match self.active_view {
            SideView::JsonFormat => self.render_json_title_bar(cx).into_any_element(),
            _ => self.render_api_title_bar(cx).into_any_element(),
        }
    }

    pub(super) fn render_api_title_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let st = self.state.read(cx);
        let project_name = st
            .active_project()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "Verve".to_string());
        // Active environment name + the list of (id, name) for the env popover.
        let active_env_name = st
            .active_project()
            .and_then(|p| {
                p.active_environment
                    .as_ref()
                    .and_then(|id| p.environments.iter().find(|e| &e.id == id))
                    .map(|e| e.name.clone())
            })
            .unwrap_or_else(|| "No environment".to_string());
        let active_project_envs: Vec<(String, String)> = st
            .active_project()
            .map(|p| {
                p.environments
                    .iter()
                    .map(|e| (e.id.clone(), e.name.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let active_env_id = st
            .active_project()
            .and_then(|p| p.active_environment.clone());
        let _ = theme.clone();

        h_flex()
            .h(px(40.))
            // On macOS the (transparent) titlebar hosts the traffic-light
            // buttons on the left; pad the content past them.
            .when(cfg!(target_os = "macos"), |this| this.pl(px(78.)))
            .when(!cfg!(target_os = "macos"), |this| this.pl_3())
            .pr_3()
            .gap_2()
            .items_center()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.muted)
            // Far-left activity rail collapse/expand toggle.
            .child(self.render_rail_toggle(cx))
            // Sidebar (tree) collapse/expand toggle.
            .child(
                Button::new("sidebar-toggle")
                    .ghost()
                    .small()
                    .icon(if self.sidebar_collapsed {
                        IconName::PanelLeftOpen
                    } else {
                        IconName::PanelLeft
                    })
                    .selected(!self.sidebar_collapsed)
                    .tooltip(if self.sidebar_collapsed {
                        "展开接口列表"
                    } else {
                        "收起接口列表"
                    })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.sidebar_collapsed = !this.sidebar_collapsed;
                        cx.notify();
                    })),
            )
            // Workspace switcher — lists all workspaces, allows switching,
            // creating, and deleting (except the default).
            .child(
                div()
                    .w(px(150.))
                    .child({
                        let ws_idx = crate::state::persistence::load_workspaces_index();
                        let ws_name = ws_idx
                            .active_meta()
                            .map(|w| w.name.clone())
                            .unwrap_or_else(|| "Default".to_string());
                        let workspaces: Vec<(String, String, bool)> = ws_idx
                            .workspaces
                            .iter()
                            .map(|w| (w.id.clone(), w.name.clone(), w.is_default))
                            .collect();
                        let ent = cx.entity();
                        Popover::new("workspace-popover")
                            .anchor(gpui::Anchor::BottomLeft)
                            .open(self.workspace_popover_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.workspace_popover_open = *open;
                                cx.notify();
                            }))
                            .trigger(
                                Button::new("workspace-trigger")
                                    .ghost()
                                    .small()
                                    .icon(IconName::LayoutDashboard)
                                    .label(ws_name)
                                    .icon(IconName::ChevronDown)
                                    .w_full(),
                            )
                            .p(px(4.))
                            .child(
                                v_flex()
                                    .w(px(200.))
                                    .gap(px(1.))
                                    .children(workspaces.iter().map(|(id, name, is_default)| {
                                        let id = id.clone();
                                        let name = name.clone();
                                        let can_delete = !*is_default;
                                        let ent = ent.clone();
                                        let id_del = id.clone();
                                        let ent_del = ent.clone();
                                        h_flex()
                                            .id(format!("ws-{id}"))
                                            .w_full()
                                            .px(px(8.))
                                            .py(px(5.))
                                            .gap(px(6.))
                                            .items_center()
                                            .rounded(px(4.))
                                            .cursor_pointer()
                                            .hover(|d| d.bg(theme.muted))
                                            .child(div().flex_1().text_size(px(13.)).child(name.clone()))
                                            .when(can_delete, |d| {
                                                d.child(
                                                    div()
                                                        .id(format!("ws-del-{id_del}"))
                                                        .cursor_pointer()
                                                        .text_color(theme.muted_foreground)
                                                        .hover(|h| h.text_color(theme.danger))
                                                        .child(IconName::Delete)
                                                        .on_click(move |_, _w, cx: &mut App| {
                                                            let _ = ent_del.update(cx, |this, cx| {
                                                                this.workspace_popover_open = false;
                                                                this.delete_workspace(id_del.clone(), cx);
                                                            });
                                                        }),
                                                )
                                            })
                                            .on_click(move |_, _w, cx: &mut App| {
                                                let _ = ent.update(cx, |this, cx| {
                                                    this.workspace_popover_open = false;
                                                    this.switch_workspace(id.clone(), cx);
                                                });
                                            })
                                    }))
                                    .child(div().h(px(1.)).w_full().bg(theme.border))
                                    .child(
                                        div()
                                            .id("ws-new")
                                            .w_full()
                                            .px(px(8.))
                                            .py(px(5.))
                                            .text_size(px(13.))
                                            .rounded(px(4.))
                                            .cursor_pointer()
                                            .text_color(theme.primary)
                                            .hover(|d| d.bg(theme.muted))
                                            .child("+ 新建工作空间")
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.workspace_popover_open = false;
                                                this.open_new_workspace(window, cx);
                                            })),
                                    ),
                            )
                    }),
            )
            // Project switcher — a popover listing the active workspace's
            // projects plus a "新建项目" entry.
            .child(
                div()
                    .w(px(180.))
                    .child(
                        Popover::new("project-popover")
                            .anchor(gpui::Anchor::BottomLeft)
                            .open(self.project_popover_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.project_popover_open = *open;
                                cx.notify();
                            }))
                            .trigger(
                                Button::new("project-trigger")
                                    .ghost()
                                    .small()
                                    .label(project_name.clone())
                                    .icon(IconName::ChevronDown)
                                    .w_full(),
                            )
                            .p(px(4.))
                            .child({
                                // Build the project list + management menu.
                                let projects: Vec<(String, usize)> = st
                                    .data
                                    .projects
                                    .iter()
                                    .enumerate()
                                    .map(|(i, p)| (p.name.clone(), i))
                                    .collect();
                                let active_idx = st.active_project;
                                let ent = cx.entity();
                                v_flex()
                                    .w(px(200.))
                                    .gap(px(1.))
                                    .children(projects.iter().map(|(name, idx)| {
                                        let name = name.clone();
                                        let idx = *idx;
                                        let is_active = idx == active_idx;
                                        let ent = ent.clone();
                                        let ent_set = ent.clone();
                                        let ent_del = ent.clone();
                                        menu_item(
                                            format!("proj-{}", idx),
                                            name,
                                            is_active,
                                            None,
                                            // Hover-revealed delete → confirm dialog.
                                            Some(Box::new(move |cx| {
                                                let _ = ent_del.update(cx, |this, cx| {
                                                    this.project_popover_open = false;
                                                    this.pending_dialog =
                                                        Some(PendingDialog::DeleteProject(idx));
                                                    cx.notify();
                                                });
                                            })),
                                            // Hover-revealed gear → open project settings.
                                            Some(Box::new(move |cx| {
                                                let _ = ent_set.update(cx, |this, cx| {
                                                    this.project_popover_open = false;
                                                    this.pending_dialog =
                                                        Some(PendingDialog::ProjectSettings(idx));
                                                    cx.notify();
                                                });
                                            })),
                                            move |_, cx| {
                                                let _ = ent.update(cx, |this, cx| {
                                                    this.project_popover_open = false;
                                                    this.state.update(cx, |s, cx| {
                                                        if s.active_project != idx {
                                                            s.active_project = idx;
                                                            s.data.active_project_id = s.data.projects.get(idx).map(|p| p.id.clone());
                                                            s.selected_request = None;
                                                            s.selected_folder = None;
                                                            s.open_request_ids.clear();
                                                            s.active_tab_id = None;
                                                            cx.emit(AppEvent::WorkspaceChanged);
                                                            cx.emit(AppEvent::SelectionChanged);
                                                        }
                                                    });
                                                    cx.notify();
                                                });
                                            },
                                            &theme,
                                        )
                                    }))
                                    // Separator + the single "+ 新建" entry.
                                    .child(menu_separator(&theme))
                                    .child({
                                        let ent = ent.clone();
                                        menu_item(
                                            "proj-new".to_string(),
                                            "新建项目".to_string(),
                                            false,
                                            Some(IconName::Plus),
                                            None,
                                            None,
                                            move |_, cx| {
                                                let _ = ent.update(cx, |this, cx| {
                                                    this.project_popover_open = false;
                                                    this.pending_new_project = true;
                                                    cx.notify();
                                                });
                                            },
                                            &theme,
                                        )
                                    })
                            }),
                    ),
            )
            // Share the whole project's docs (scope = Project).
            .child(
                Button::new("share-project")
                    .ghost()
                    .small()
                    .icon(vicon(SHARE))
                    .tooltip("分享API文档")
                    .on_click(cx.listener(
                        |this, _ev, window, cx| {
                            this.open_share_dialog(
                                ShareScope::Project,
                                None,
                                None,
                                window,
                                cx,
                            );
                        },
                    )),
            )
            .child(div().flex_1())
            // Environment switcher — a popover listing environments plus
            // management entries (环境管理/Cookie/全局参数/全局变量/新建).
            .child(
                div()
                    .w(px(160.))
                    .child(
                        Popover::new("env-popover")
                            .anchor(gpui::Anchor::BottomLeft)
                            .open(self.env_popover_open)
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.env_popover_open = *open;
                                cx.notify();
                            }))
                            .trigger(
                                Button::new("env-trigger")
                                    .ghost()
                                    .small()
                                    .label(active_env_name.clone())
                                    .icon(IconName::ChevronDown)
                                    .w_full(),
                            )
                            .p(px(4.))
                            .child({
                                // Build env list + a single "+ 新建" entry. Each
                                // environment row reveals a settings (gear) button
                                // on hover that opens environment management.
                                let envs: Vec<(String, String)> = active_project_envs
                                    .iter()
                                    .map(|(id, name)| (name.clone(), id.clone()))
                                    .collect();
                                let active_env = active_env_id.clone();
                                let ent = cx.entity();
                                v_flex()
                                    .w(px(220.))
                                    .gap(px(1.))
                                    .child(menu_item(
                                        "env-none".to_string(),
                                        "No environment".to_string(),
                                        active_env.is_none(),
                                        None,
                                        None,
                                        None,
                                        {
                                            let ent = ent.clone();
                                            move |_, cx| {
                                                let _ = ent.update(cx, |this, cx| {
                                                    this.env_popover_open = false;
                                                    this.state.update(cx, |s, cx| {
                                                        if let Some(project) =
                                                            s.active_project_mut()
                                                        {
                                                            project.active_environment = None;
                                                            cx.emit(AppEvent::EnvironmentChanged);
                                                        }
                                                    });
                                                    cx.notify();
                                                });
                                            }
                                        },
                                        &theme,
                                    ))
                                    .children(envs.iter().map(|(name, id)| {
                                        let name = name.clone();
                                        let id = id.clone();
                                        let id_del = id.clone();
                                        let is_active = Some(&id) == active_env.as_ref();
                                        let ent = ent.clone();
                                        let ent_set = ent.clone();
                                        let ent_del = ent.clone();
                                        menu_item(
                                            format!("env-{}", id),
                                            name,
                                            is_active,
                                            None,
                                            // Hover-revealed delete → confirm dialog.
                                            Some(Box::new(move |cx| {
                                                let _ = ent_del.update(cx, |this, cx| {
                                                    this.env_popover_open = false;
                                                    this.pending_dialog =
                                                        Some(PendingDialog::DeleteEnv(id_del.clone()));
                                                    cx.notify();
                                                });
                                            })),
                                            // Hover-revealed gear → open env management.
                                            Some(Box::new(move |cx| {
                                                let _ = ent_set.update(cx, |this, cx| {
                                                    this.env_popover_open = false;
                                                    this.pending_dialog =
                                                        Some(PendingDialog::Environments);
                                                    cx.notify();
                                                });
                                            })),
                                            move |_, cx| {
                                                let _ = ent.update(cx, |this, cx| {
                                                    this.env_popover_open = false;
                                                    this.state.update(cx, |s, cx| {
                                                        if let Some(project) =
                                                            s.active_project_mut()
                                                        {
                                                            project.active_environment =
                                                                Some(id.clone());
                                                            cx.emit(AppEvent::EnvironmentChanged);
                                                        }
                                                    });
                                                    cx.notify();
                                                });
                                            },
                                            &theme,
                                        )
                                    }))
                                    // Separator + the single "+ 新建" entry.
                                    .child(menu_separator(&theme))
                                    .child({
                                        let ent = ent.clone();
                                        menu_item(
                                            "env-new".to_string(),
                                            "新建环境".to_string(),
                                            false,
                                            Some(IconName::Plus),
                                            None,
                                            None,
                                            move |_, cx| {
                                                let _ = ent.update(cx, |this, cx| {
                                                    this.env_popover_open = false;
                                                    this.pending_new_env = true;
                                                    cx.notify();
                                                });
                                            },
                                            &theme,
                                        )
                                    })
                            }),
                    ),
            )
            // Sync (save to disk) button — persists the workspace, then runs
            // a git sync (commit dirty → pull/push). Shows a label while busy
            // and a transient success/failure chip after, so the user always
            // gets feedback (the original "no response" complaint).
            .child({
                let g = self.git.read(cx);
                let theme = cx.theme().clone();
                let busy_label = g.busy.clone();
                let busy = g.is_busy();
                let branch = g.status.branch.clone().unwrap_or_else(|| "—".to_string());
                let dirty = g.status.dirty;
                let tooltip = if g.initialized {
                    format!("Git 同步 · {} · {} 待提交", branch, dirty)
                } else {
                    "同步/保存到本地".to_string()
                };
                let last_result = g.last_result.clone();
                let git_for_click = self.git.clone();
                let git_for_dismiss = self.git.clone();
                let muted_fg = theme.muted_foreground;
                let fg = theme.foreground;
                let accent = theme.accent;
                let danger = theme.danger;
                h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        Button::new("sync")
                            .ghost()
                            .small()
                            .icon(IconName::Replace)
                            .selected(busy)
                            .tooltip(tooltip)
                            // Show "同步中" while a git op is in flight, so the
                            // click is visibly acknowledged.
                            .when_some(busy_label, |btn, label| {
                                btn.label(label).disabled(true)
                            })
                            .on_click(cx.listener(move |this, _, _, cx| {
                                log::info!("sync 按钮：点击 — 先持久化 workspace.json");
                                // Persist first so the latest workspace.json is on disk,
                                // then commit + sync.
                                this.state.update(cx, |s, cx| s.persist(cx));
                                let _ = git_for_click.update(cx, |g, cx| {
                                    log::info!("sync 按钮：调用 sync_async（若已有操作进行中会被 is_busy 挡掉，由 Persisted 自动同步接管）");
                                    g.sync_async(None, cx)
                                });
                            })),
                    )
                    // Transient result chip: green ✓ on success, red message on
                    // failure. Dismissable via the × control.
                    .when_some(last_result, |col, (msg, ok)| {
                        col.child(
                            div()
                                .text_size(px(11.))
                                .px(px(6.))
                                .py(px(2.))
                                .rounded(px(4.))
                                .max_w(px(260.))
                                .text_ellipsis()
                                .whitespace_nowrap()
                                .overflow_hidden()
                                .when(ok, |d| d.bg(accent.opacity(0.2)).text_color(fg))
                                .when(!ok, |d| d.bg(danger.opacity(0.2)).text_color(danger))
                                .child(if ok { format!("✓ {msg}") } else { format!("⚠ {msg}") }),
                        )
                    })
                    // A small dismiss (×) control to clear the chip.
                    .when_some(self.git.read(cx).last_result.clone(), |col, _| {
                        col.child(
                            div()
                                .id("sync-dismiss")
                                .cursor_pointer()
                                .text_size(px(11.))
                                .text_color(muted_fg)
                                .hover(|d| d.text_color(fg))
                                .child("×")
                                .on_click(move |_, _w, cx: &mut App| {
                                    let _ = git_for_dismiss.update(cx, |g, cx| {
                                        g.last_result = None;
                                        cx.emit(crate::git::GitEvent::Updated);
                                        cx.notify();
                                    });
                                }),
                        )
                    })
            })
            .child(
                Button::new("import")
                    .ghost()
                    .small()
                    .icon(vicon(IMPORT))
                    .tooltip("导入集合 (Postman/OpenAPI JSON)")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.import_collection(cx);
                    })),
            )
            .child(self.render_export_picker(cx))
            // Update check button: shows a spinner while checking, a download
            // icon when an update is available, or a refresh icon otherwise.
            .child(self.render_update_button(cx))
            // Language switcher (top-right).
            .child(self.render_lang_picker(cx))
    }

    pub(super) fn render_rail_toggle(&self, cx: &Context<Self>) -> impl IntoElement {
        Button::new("rail-toggle")
            .ghost()
            .small()
            .icon(if self.rail_collapsed {
                IconName::PanelLeftOpen
            } else {
                IconName::PanelLeftClose
            })
            .selected(!self.rail_collapsed)
            .tooltip(if self.rail_collapsed {
                "展开左侧导航栏"
            } else {
                "收起左侧导航栏"
            })
            .on_click(cx.listener(|this, _, _, cx| {
                this.rail_collapsed = !this.rail_collapsed;
                cx.notify();
            }))
    }

    pub(super) fn render_update_button(&self, cx: &Context<Self>) -> impl IntoElement {
        let checking = self.update_checking;
        let has_update = self.update_info.is_some();
        let update_info = self.update_info.clone();
        let icon: Icon = if checking {
            IconName::LoaderCircle.into()
        } else if has_update {
            IconName::ExternalLink.into()
        } else {
            vicon(REFRESH_CW)
        };
        let tooltip = if checking {
            "正在检查更新…".to_string()
        } else if let Some(info) = &update_info {
            format!("发现新版本 v{}，点击前往下载", info.version)
        } else {
            "检查更新".to_string()
        };
        Button::new("update-check")
            .ghost()
            .small()
            .icon(icon)
            .tooltip(tooltip)
            .when(checking, |b| b.disabled(true))
            .on_click(cx.listener(move |this, _ev, window, cx| {
                if this.update_info.is_some() {
                    this.open_update_download(window, cx);
                } else {
                    this.check_updates_manual(window, cx);
                }
            }))
    }

    pub(super) fn render_export_picker(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let open = self.export_popover_open;
        Popover::new("export-picker")
            .anchor(gpui::Anchor::BottomRight)
            .open(open)
            .on_open_change(cx.listener(|this, open, _, cx| {
                this.export_popover_open = *open;
                cx.notify();
            }))
            .trigger(
                Button::new("export-trigger")
                    .ghost()
                    .small()
                    .icon(vicon(EXPORT))
                    .tooltip("导出项目"),
            )
            .p(px(4.))
            .child(v_flex().w(px(200.)).gap(px(2.)).children(
                crate::export::Format::ALL.iter().map(|&f| {
                    let ent = cx.entity();
                    div()
                        .id(format!("export-fmt-{}", f.label()))
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .cursor_pointer()
                        .hover(|s| s.bg(theme.muted))
                        .child(div().text_sm().text_color(theme.foreground).child(format!(
                            "{} (.{})",
                            f.label(),
                            f.extension()
                        )))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.export_popover_open = false;
                            this.export_project(f, cx);
                        }))
                }),
            ))
    }

    pub(super) fn render_lang_picker(&self, cx: &Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let open = self.lang_popover_open;
        let cur = rust_i18n::locale().to_string();
        let lang_label = if cur.starts_with("en") { "EN" } else { "中" };
        Popover::new("lang-picker")
            .anchor(gpui::Anchor::BottomRight)
            .open(open)
            .on_open_change(cx.listener(|this, open, _, cx| {
                this.lang_popover_open = *open;
                cx.notify();
            }))
            .trigger(
                Button::new("rail-lang")
                    .ghost()
                    .small()
                    .label(lang_label)
                    .icon(IconName::Globe)
                    .tooltip("语言 / Language"),
            )
            .p(px(4.))
            .child(
                v_flex()
                    .w(px(140.))
                    .gap(px(2.))
                    .child(
                        div()
                            .id("lang-opt-zh")
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.muted))
                            .when(cur.starts_with("zh") || cur == "zh-CN", |d| {
                                d.bg(theme.accent.opacity(0.5))
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.foreground)
                                    .child("简体中文"),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                rust_i18n::set_locale("zh-CN");
                                let mut l =
                                    crate::state::persistence::load_layout().unwrap_or_default();
                                l.locale = Some("zh-CN".into());
                                let _ = crate::state::persistence::save_layout(&l);
                                this.lang_popover_open = false;
                                window.refresh();
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .id("lang-opt-en")
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(theme.muted))
                            .when(cur == "en" || cur.starts_with("en"), |d| {
                                d.bg(theme.accent.opacity(0.5))
                            })
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(theme.foreground)
                                    .child("English"),
                            )
                            .on_click(cx.listener(|this, _, window, cx| {
                                rust_i18n::set_locale("en");
                                let mut l =
                                    crate::state::persistence::load_layout().unwrap_or_default();
                                l.locale = Some("en".into());
                                let _ = crate::state::persistence::save_layout(&l);
                                this.lang_popover_open = false;
                                window.refresh();
                                cx.notify();
                            })),
                    ),
            )
    }

    /// The title bar shown while the SSH manager is active. SSH hosts live in
    /// a global store (`~/.verve/ssh_hosts.json`), not in any workspace, so
    /// the API bar's workspace/project/environment controls don't apply here.
    /// This bar carries the SSH context instead — title, saved-host count,
    /// and the primary "new host" action — plus the app-level update/language
    /// controls. Height, padding, and the leading rail toggle match
    /// `render_api_title_bar` exactly, so nothing shifts on view switch.

    /// cmd-o — open a Markdown file in the markdown editor view.

    pub(super) fn render_json_title_bar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();

        h_flex()
            .h(px(40.))
            .when(cfg!(target_os = "macos"), |this| this.pl(px(78.)))
            .when(!cfg!(target_os = "macos"), |this| this.pl_3())
            .pr_3()
            .gap_2()
            .items_center()
            .border_b_1()
            .border_color(theme.border)
            .bg(theme.muted)
            .child(self.render_rail_toggle(cx))
            // Icon + title
            .child(
                h_flex()
                    .gap(px(6.))
                    .items_center()
                    .child(div().text_color(theme.primary).child(vicon(BRACES_JSON)))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(rust_i18n::t!("json.title").to_string()),
                    ),
            )
            .child(div().flex_1())
            // Toolbar buttons that trigger actions on the json panel
            .child({
                let json = self.json.clone();
                Button::new("json-title-format")
                    .small()
                    .ghost()
                    .icon(IconName::Check)
                    .label(rust_i18n::t!("json.format_btn").to_string())
                    .on_click(cx.listener(move |_, _, _, cx| {
                        let _ = json.update(cx, |panel, cx| panel.format_from_title(cx));
                    }))
            })
            .child({
                let json = self.json.clone();
                let active = json.read(cx).is_compact_active();
                Button::new("json-title-compact")
                    .small()
                    .ghost()
                    .label(rust_i18n::t!("json.compact_btn").to_string())
                    .when(active, |b| b.selected(true))
                    .on_click(cx.listener(move |_, _, _, cx| {
                        let _ = json.update(cx, |panel, cx| panel.toggle_compact(cx));
                    }))
            })
            .child({
                let json = self.json.clone();
                Button::new("json-title-expand")
                    .small()
                    .ghost()
                    .label(rust_i18n::t!("json.expand_all").to_string())
                    .on_click(cx.listener(move |_, _, _, cx| {
                        let _ = json.update(cx, |panel, cx| panel.expand_all(cx));
                    }))
            })
            .child({
                let json = self.json.clone();
                Button::new("json-title-collapse")
                    .small()
                    .ghost()
                    .label(rust_i18n::t!("json.collapse_all").to_string())
                    .on_click(cx.listener(move |_, _, _, cx| {
                        let _ = json.update(cx, |panel, cx| panel.collapse_all(cx));
                    }))
            })
            .child({
                let json = self.json.clone();
                Button::new("json-title-copy")
                    .small()
                    .ghost()
                    .icon(IconName::Copy)
                    .label(rust_i18n::t!("json.copy_btn").to_string())
                    .on_click(cx.listener(move |_, _, _, cx| {
                        let _ = json.update(cx, |panel, cx| panel.copy_result(cx));
                    }))
            })
            .child(self.render_update_button(cx))
            .child(self.render_lang_picker(cx))
    }

}
