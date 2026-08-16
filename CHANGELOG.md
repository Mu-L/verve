# Changelog

All notable changes to Verve will be documented in this file.

## [0.5.6] - 2026-08-15

### 🔧 Enhancements (synced from upstream)

- **`{{variable}}` auto-completion** — typing `{{` in URL / KV value / base-URL
  inputs now opens an LSP-style completion popover listing global + environment
  variables (and dynamic variables like `$random`) with scope badges; filters as
  you type.
- **Variable rename sync** — renaming an environment/global variable key now
  rewrites every `{{oldKey}}` placeholder reference across requests, folders,
  params/headers/cookies, and base-URL overrides, so existing references stay
  valid.
- **Base-URL plain display** — the folder base-URL input shows the resolved
  plain URL (no `{{var}}` braces) while persisting the original placeholder, so
  variable linkage stays intact.

### 📝 Docs

- READMEs: refreshed both iteration logs — Community (variable completion /
  rename sync) and Pro (SSH local terminal mode, notes/markdown per-frame perf,
  word-count fix).

## [0.5.5] - 2026-08-14

### 🔧 Enhancements (synced from upstream)

- **Folder base-URL dropdown scroll** — the "select from environment variables"
  popover for a folder's base URL now scrolls when the list is long, and shows
  the bare env-var name instead of the `{{name}}` placeholder.

### 📝 Docs

- READMEs: refreshed the Pro Edition iterations log — editor per-frame CPU
  optimization (word-count throttle, render-cache reuse, window bookkeeping),
  smoother markdown/notes scrolling, a Markdown paste ordered-list fix, and a
  native directory picker for the media root.

## [0.5.3] - 2026-08-13

### 🔧 Enhancements (synced from upstream)

- **Postman-style dynamic variables** — `{{$random}}`, `{{$uuid}}`,
  `{{$timestamp}}`, and the new `{{$sparkid}}` (21-char Base58, time-sortable)
  are expanded to a fresh value on every send; a user-defined variable of the
  same name still wins.
- **Clear response on send** — clicking Send immediately clears the previous
  response and shows a "请求中…" (requesting) state in the realtime panel until
  the reply arrives, so stale content is never mistaken for the live result.
- **Smarter request search** — the project tree search now matches request
  URLs/paths, not just names.
- **"Move to" picker** — move a request or folder into another folder via a
  searchable destination picker (in addition to drag-and-drop).
- **Tab overflow dropdown** — when too many request tabs are open, a `»`
  dropdown lists the overflow; `cmd-w` / `ctrl-w` closes the active tab.
- **Environment manager** — the window now fills its container, has an
  overflow-scroll, and confirms before deleting a non-empty row.
- **KV table** — value cells expand on focus when content overflows, with
  configurable value width / description flex / enabled-toggle.
- **Per-request base URL precedence fix** — an explicit override
  (disable/custom) now always takes precedence over host-based URL splitting on
  reload.
- **sparkid entity IDs** — new records use collision-resistant, time-sortable
  sparkids instead of UUID v4 (shorter short-codes, no near-simultaneous
  collisions). Affects project/folder/request/share/hosts-profile ids and the
  Postman export ids.

### 📦 Other

- Added `sparkid = "2.2.1"` dependency.
- READMEs: added a "Community Edition — Recent Iterations" section and refreshed
  the Pro Edition iterations log (notes full-text search, media storage,
  SSH duplicate-session skip-MFA, etc.).
- Pro-only modules (SSH / Docker / Kubernetes / stress testing / WYSIWYG notes
  / PDF / relocate / autotest / cloud sharing / verve-server / notes-index /
  media) remain exclusive to the Pro Edition.

## [0.5.0] - 2026-08-08

### 🔧 Enhancements (synced from upstream)

- **Per-request base URL override (tri-state)** — each request can now choose to
  inherit the folder's base URL, explicitly disable any prefix, or set a custom
  one (supports `{{var}}` placeholders). The dropdown shows the effective URL and
  the "不使用前置 URL" state is persisted per request.
- **Global parameters / headers / cookies** — project-level global
  params/headers/cookies are now auto-applied to every request, with a same-named
  per-request entry overriding the global one (matching the global-manager UI
  copy "接口级同名头覆盖"). Matching is case-insensitive for HTTP headers.
- **Method-colored Send button & refined tab bar** — the Send button and method
  chip now use a custom variant tinted by the HTTP method color, and the request
  tabs use an Apifox/Postman-style active underline.
- i18n: added the `export_failed` message (en + zh-CN).

### 📦 Other

- Non-code resources (locales) synced from upstream.
- Pro-only modules (SSH / Docker / Kubernetes / stress testing / WYSIWYG notes /
  PDF / relocate / autotest / cloud sharing / `verve-server`) remain exclusive to
  the Pro Edition and are not part of the Community Edition.

## [0.3.0] - 2026-08-02

### 🎉 Major Release — Rust + GPUI Rewrite

This is a complete rewrite of Verve in Rust using the GPUI framework (the same GPU-accelerated UI framework that powers the Zed editor). This release brings dramatic performance improvements and a native, lightweight experience.

### ⚡ Performance & Architecture

- **Rust + GPUI architecture** — Complete rewrite from the ground up in Rust
- **GPU-accelerated rendering** — Native GPU rendering via GPUI, no Electron/Chromium
- **< 1 second startup** — Instant launch compared to 3-5 seconds for Electron apps
- **< 100 MB RAM** — 5× lighter than Electron-based tools (typically 500 MB+)
- **~60 FPS rendering** — Fluid, responsive UI even under heavy load

### 🔧 Core Infrastructure

#### New Module Structure
- **Git operations** (`src/git/`) — Complete git integration for cross-machine sync
  - `ops.rs` — Git operations wrapper around gitoxide
  - `state.rs` — Git state management for workspaces
- **HTTP clients** (`src/http/`) — Multi-protocol client implementations
  - `client.rs` — HTTP client with reqwest
  - `grpc.rs` — gRPC-Web client support
  - `sse.rs` — Server-Sent Events client
  - `tcp.rs` — TCP client support
  - `ws.rs` — WebSocket client
  - `variable.rs` — Variable substitution system
- **Share system** (`src/share/`) — Document sharing infrastructure
  - `html.rs` — HTML documentation generator
  - `server.rs` — Local HTTP share server
  - `models.rs` — Share data models
  - `persist.rs` — Share persistence layer
  - `qrcode.rs` — QR code generation

#### UI Architecture Refactor
- **New app structure** (`src/ui/app/`)
  - `mod.rs` — Main app module
  - `construction.rs` — UI construction helpers
  - `rail.rs` — Navigation rail component
  - `titlebar.rs` — Native titlebar implementation
  - `workspaces.rs` — Workspace management UI
  - `actions.rs` — UI action handlers
  - `widgets.rs` — Reusable UI widgets
  - `share.rs` — Share dialog components

- **New specialized panels**
  - `bootstrap_dialog.rs` — First-run setup dialog
  - `console_panel.rs` — Console output panel
  - `env_panel.rs` — Environment variable editor
  - `environments_view.rs` — Multi-environment management UI
  - `hosts_panel.rs` — Hosts file management UI
  - `json_panel.rs` — JSON formatter panel
  - `kv_manager_view.rs` — Key-value pair manager
  - `kv_table.rs` — Key-value table component
  - `method_colors.rs` — HTTP method color coding
  - `mock_console_panel.rs` — Mock server console
  - `project_manage_panel.rs` — Project management UI
  - `project_tree_panel.rs` — Project tree with drag-reorder
  - `proxy_panel.rs` — HTTP capture proxy UI
  - `request_panel/` — Request editor (split into multiple files)
  - `response_panel.rs` — Response viewer
  - `settings_window.rs` — Settings dialog
  - `share_dialog.rs` — Share generation dialog
  - `share_panel.rs` — Share history panel
  - `theme.rs` — Theme system implementation

#### State Management
- **New state architecture** (`src/state/`)
  - `app_state.rs` — Global application state
  - `models.rs` — Data models for projects, requests, environments
  - `sample_data.rs` — Demo project data
  - `persistence.rs` — Enhanced workspace persistence with Git sync

### 🌍 Internationalization

- **Locale system** (`locales/`)
  - `en.yml` — English translations (500+ keys)
  - `zh-CN.yml` — Simplified Chinese translations (500+ keys)
- **rust-i18n integration** — Compile-time i18n with type-safe translation API

### 🎨 Themes

- **22 built-in themes** (`themes/`)
  - Catppuccin (Mocha/Latte/Frappe/Macchiato)
  - Gruvbox (Dark/Light)
  - Tokyo Night
  - Solarized (Dark/Light)
  - Everforest
  - Flexoki
  - Adventure
  - Alduin
  - Asciiinema
  - Aurora
  - Ayu
  - Fahrenheit
  - Harper
  - Hybrid
  - Jellybeans
  - Kibble
  - macOS Classic
  - Matrix
  - Mellifluous
  - Molokai
  - Spaceduck
  - Twilight

### 🛠️ Build & Tooling

- **Enhanced build script** (`scripts/build.sh`)
  - Cross-platform build automation (macOS/Linux/Windows)
  - Native dependency detection and auto-install for Linux
  - Support for `.app`, `.deb`, `.AppImage`, and `.msi` packages
  - CI-friendly `--no-auto-install` flag
- **Dependency install script** (`scripts/install-deps.sh`)
  - Automatic detection of package manager (apt/yum/dnf/pacman)
  - Installs FreeType, fontconfig, and other native dependencies
- **Icon generation script** (`scripts/gen_icon.py`)
  - Automated icon asset generation from SVG sources

### 📦 New Dependencies

Major dependency updates and additions:
- `gpui` + `gpui_platform` — GPU-accelerated UI framework
- `gpui-component` — UI component library (custom fork with drag-and-drop)
- `gix` (gitoxide) — Pure-Rust git operations
- `tokio-tungstenite` — WebSocket client
- `boa_engine` — JavaScript engine for scripts
- `rust-embed` — Asset embedding
- `rust-i18n` — Internationalization
- `qrcode` — QR code generation
- `semver` — Semantic versioning

### 🔧 Configuration Updates

- **Cargo.toml** — Updated to v0.3.0 with new dependencies
- **Cargo.lock** — Regenerated for Rust rewrite
- **Desktop entries** — Updated `.desktop` file for Linux
- **macOS bundle metadata** — Updated Info.plist values

### 📚 Documentation

- **Updated README.md** — Comprehensive feature documentation
  - Rust + GPUI architecture explanation
  - Community vs Pro edition comparison
  - Detailed feature lists for both editions
  - Performance benchmarks
  - Installation instructions
- **Updated README.zh-CN.md** — Chinese version of documentation
- **New demo assets** — Screenshots and demo videos
- **Removed old assets** — Cleaned up legacy video files

### 🗑️ Removed Files

Legacy files from pre-Rust version:
- `src/bin/verve_server.rs` — Old server binary (relocated)
- `src/git.rs`, `src/http.rs`, `src/share.rs`, `src/ssh.rs` — Consolidated into modules
- `src/ui/app.rs`, `src/ui/request_panel.rs` — Restructured UI architecture
- Old demo videos — Replaced with new thumbnail-based links

### ✅ Features Retained

All major features from v0.2.0 have been reimplemented in the new architecture:
- ✅ HTTP API debugging (all methods)
- ✅ Multi-protocol clients (HTTP/gRPC/TCP/SSE/WebSocket)
- ✅ Project tree with multi-level nesting
- ✅ Environment variable management
- ✅ Request/response history
- ✅ Mock server (Exact/Prefix/Regex matching)
- ✅ HTTP capture proxy
- ✅ Document sharing (local HTML generation)
- ✅ Hosts file manager
- ✅ JSON formatter
- ✅ Import/Export (Postman/OpenAPI/Swagger)
- ✅ Git-based cross-machine sync

### 💎 Pro Edition Features

Pro-only features (not in Community Edition):
- ✅ SSH terminal with SFTP
- ✅ Docker container management
- ✅ Kubernetes pod inspection
- ✅ Stress testing
- ✅ Automated test suites
- ✅ Markdown notes editor
- ✅ PDF viewer/editor
- ✅ Cloud document sharing (verve-server)

### 🔒 Security

- **Secure credential storage** — OS keychain integration
- **AES-256-GCM encryption** — For SSH credentials
- **Argon2id key derivation** — Password hashing
- **TOFU known_hosts verification** — SSH host key validation
- **Token-based Git auth** — PAT never touches disk

### 🐛 Bug Fixes

- Fixed drag-and-drop reordering in project tree
- Fixed theme switching consistency
- Fixed environment variable priority resolution
- Fixed JSON pretty-printing edge cases
- Fixed share server port binding conflicts

### ⚠️ Breaking Changes

- **Minimum OS requirements updated**
  - macOS: 11.0+
  - Linux: glibc 2.31+
- **Configuration format** — Workspace JSON format updated (migration handled automatically)
- **GPUI version pinning** — Locked to specific Zed commit for stability

### 📝 Migration Notes

Users upgrading from v0.2.x:
1. Workspaces will be automatically migrated to the new format
2. Theme selections will be reset (22 new themes available)
3. Re-authenticate Git remotes if using cross-machine sync
4. Some keyboard shortcuts have changed — see Settings > Shortcuts

---

## [0.2.0] - Previous Release

(Release notes archived in git history)

---

**For older releases, please refer to the git commit history.**
