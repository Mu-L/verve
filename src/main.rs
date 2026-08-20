//! Verve — an offline-first API co-development platform built with gpui-component.

// Roadmap features and helpers produce dead-code warnings until wired up.
#![allow(dead_code)]
// The GPUI element-builder style produces some complex closures.
#![allow(clippy::type_complexity)]
// Some serde data structs are fine to derive-impl via macros.
#![allow(clippy::derivable_impls)]

use gpui::{img, *};
use gpui_component::button::*;
use gpui_component::*;

use verve::assets::VerveAssets;
use verve::state::persistence;
use verve::ui::VerveApp;
use verve::{mock, state, ui};

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        };
    }
    let _ = env_logger::Builder::from_default_env()
        .format_timestamp_secs()
        .try_init();

    let app = gpui_platform::application().with_assets(VerveAssets::new());

    app.run(move |cx| {
        gpui_component::init(cx);

        ui::themes::load_builtin_themes(cx);
        gpui_component::Theme::change(gpui_component::ThemeMode::Dark, None, cx);
        gpui_component::Theme::global_mut(cx).list.active_highlight = false;

        if let Ok(http_client) =
            reqwest_client::ReqwestClient::user_agent(concat!("verve/", env!("CARGO_PKG_VERSION")))
        {
            cx.set_http_client(std::sync::Arc::new(http_client));
        }

        // Apply locale early.
        if let Some(locale) = persistence::load_layout().and_then(|l| l.locale) {
            rust_i18n::set_locale(&locale);
        } else {
            rust_i18n::set_locale("zh-CN");
        }

        // Decide whether to show bootstrap or main app.
        let is_first_run = persistence::is_first_run();

        // The main window must be opened *before* the first-run welcome
        // dialog: Windows/Linux stack each newly opened window in front of
        // the previously opened one, so the old order (welcome, then main)
        // buried the welcome dialog behind the main window and users had to
        // alt-tab to find it. The welcome window is opened afterwards and
        // explicitly activated below so the wizard overlays the main window.
        launch_main_app(cx);

        if is_first_run {
            log::info!("First run detected, showing bootstrap dialog");
            let bounds = ui::app::centered_window_bounds(px(560.), px(560.), cx);
            cx.spawn(async move |cx| {
                match cx.open_window(
                    WindowOptions {
                        titlebar: Some(gpui::TitlebarOptions {
                            title: Some("Verve - Welcome".into()),
                            appears_transparent: false,
                            traffic_light_position: Some(point(px(14.), px(16.))),
                        }),
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        window_min_size: Some(size(px(400.), px(400.))),
                        ..Default::default()
                    },
                    |window, cx| {
                        let dialog = ui::bootstrap_dialog::BootstrapDialog::new(window, cx);
                        cx.new(|cx| Root::new(dialog, window, cx))
                    },
                ) {
                    Ok(handle) => {
                        // Re-assert foreground over the main window opened
                        // just before this one; a no-op where the WM already
                        // stacked the welcome window on top.
                        let _ = handle
                            .update(cx, |_root, window, _cx| window.activate_window());
                    }
                    Err(err) => log::error!("Failed to open bootstrap window: {err}"),
                }
            })
            .detach();

            persistence::mark_bootstrap_done();
        }
    });

    let _ = Button::new("noop");
}

fn launch_main_app(cx: &mut App) {
    let app_state = state::AppState::init(cx);

    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-enter", ui::request_panel::SendRequest, None),
        KeyBinding::new("cmd-s", ui::app::SaveWorkspace, None),
        // New API request moved off cmd-n; use cmd-shift-n.
        KeyBinding::new("cmd-shift-n", ui::app::NewRequest, None),
        // Close the active request tab (cmd-w).
        KeyBinding::new("cmd-w", ui::app::CloseFile, None),
    ]);
    cx.bind_keys(ui::app::rail_slot_keybindings("cmd"));
    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("ctrl-enter", ui::request_panel::SendRequest, None),
        KeyBinding::new("ctrl-s", ui::app::SaveWorkspace, None),
        KeyBinding::new("ctrl-shift-n", ui::app::NewRequest, None),
        KeyBinding::new("ctrl-w", ui::app::CloseFile, None),
    ]);
    cx.bind_keys(ui::app::rail_slot_keybindings("ctrl"));

    let bounds = ui::app::centered_window_bounds(px(1400.), px(900.), cx);

    let mut options = WindowOptions {
        titlebar: Some(gpui::TitlebarOptions {
            title: Some("Verve".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(14.), px(16.))),
        }),
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        window_min_size: Some(size(px(960.), px(600.))),
        ..Default::default()
    };
    // Linux: drop the WM's server-side title bar — the app bar already
    // draws min/max/close, so the WM bar would add a second close button.
    ui::app::apply_window_chrome_fixes(&mut options, false);

    cx.spawn(async move |cx| {
        cx.open_window(
            options,
            |window, cx| {
                // The window close button (and the doc-level close) should hide
                // the window rather than quit the app, so the process — and any
                // in-memory state — stays alive until the user explicitly quits.
                // Returning `false` cancels the close; `minimize_window` hides it.
                window.on_window_should_close(cx, |window, _cx| {
                    window.minimize_window();
                    false
                });
                let view = cx.new(|cx| VerveApp::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx).bg(cx.theme().background))
            },
        )
        .expect("Failed to open window");
    })
    .detach();
}
