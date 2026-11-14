//! Render the OMCA UI offscreen (no window) and save PNGs of the main screens.
//!
//! ```sh
//! cargo run -p craft-anything-ui-egui --example snapshot -- --out-dir target/snapshots [--scale 2] [--light] [--only NAME] [--width 1440] [--height 900] [--lang fr]
//! ```
//!
//! The README's pictures: `simple-home` → make.png and `project-game` → project-game.png (both
//! `--height 1000`, so Start shows), `project-plan` → project-website.png, `experienced` → workbench.png.
//!
//! Scenes: first run (welcome, brain, purpose), Beginner Home, Beginner Ask (with an evidence
//! card), Experienced workbench, and Home in Studio Light.

use craft_anything_engine::Mode;
use craft_anything_engine::evidence::Confidence;
use craft_anything_ui_egui::state::{FirstRunStep, Screen};
use craft_anything_ui_egui::theme::{self, ThemeKind, Tokens, size, space};
use craft_anything_ui_egui::{CraftApp, demo, icons, widgets};
use egui_kittest::kittest::Queryable;
use std::path::PathBuf;

fn arg(args: &[String], name: &str) -> Option<String> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1).cloned())
}

/// `--lang`: the UI language for every scene (English when absent).
static LANG: std::sync::OnceLock<String> = std::sync::OnceLock::new();

fn render(out: &std::path::Path, name: &str, w: f32, h: f32, scale: f32, make: impl FnOnce() -> CraftApp + 'static) {
    let mut harness =
        egui_kittest::Harness::builder().with_size(egui::vec2(w, h)).with_pixels_per_point(scale).with_max_steps(64).wgpu().build_eframe(move |cc| {
            CraftApp::setup_context(&cc.egui_ctx);
            let mut app = make();
            // A made-up folder: the default names the account of whoever renders the pictures.
            app.settings.projects_dir = Some(PathBuf::from("~/Documents/OMCA"));
            app.settings.language = LANG.get().cloned().unwrap_or_else(|| "en".into());
            craft_anything_ui_egui::i18n::set_language(&app.settings.language);
            app
        });
    // Let fonts, SVG icons and fades settle.
    for _ in 0..24 {
        harness.step();
    }
    match name {
        "composer-picker" | "composer-empty-picker" => {
            harness.get_by_label_contains(&craft_anything_ui_egui::i18n::tr("Change model")).click();
            harness.run_steps(3);
        }
        "providers-search" | "providers-connect" => {
            harness.get_by_label(&craft_anything_ui_egui::i18n::tr("Add account")).click();
            harness.run_steps(3);
            harness.get_by_label(&craft_anything_ui_egui::i18n::tr("Search providers")).click();
            harness.run_steps(2);
            harness.get_by_label(&craft_anything_ui_egui::i18n::tr("Search providers")).type_text("Other");
            harness.run_steps(3);
            if name == "providers-connect" {
                harness.get_by_label(&craft_anything_ui_egui::i18n::tr("API key")).click();
                harness.run_steps(3);
            }
        }
        "providers-detail" => {}
        _ => {}
    }
    match harness.render() {
        Ok(img) => {
            let path = out.join(format!("{name}.png"));
            match img.save(&path) {
                Ok(()) => println!("wrote {} ({}×{})", path.display(), img.width(), img.height()),
                Err(e) => eprintln!("{name}: {e}"),
            }
        }
        Err(e) => eprintln!("{name}: render failed: {e}"),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = PathBuf::from(arg(&args, "--out-dir").unwrap_or_else(|| "target/snapshots".into()));
    let scale: f32 = arg(&args, "--scale").and_then(|s| s.parse().ok()).unwrap_or(1.5);
    let only = arg(&args, "--only");
    if let Some(l) = arg(&args, "--lang") {
        let _ = LANG.set(l);
    }
    if let Err(e) = std::fs::create_dir_all(&out) {
        eprintln!("can't create {}: {e}", out.display());
        return;
    }
    let want = |n: &str| only.as_deref().is_none_or(|o| n.contains(o));
    let w = arg(&args, "--width").and_then(|s| s.parse().ok()).unwrap_or(1440.0);
    let h = arg(&args, "--height").and_then(|s| s.parse().ok()).unwrap_or(900.0);

    if want("first-run-welcome") {
        render(&out, "first-run-welcome", w, h, scale, || demo::app(demo::engine(Mode::Beginner, false)));
    }
    if want("first-run-brain") {
        render(&out, "first-run-brain", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, false));
            a.state.first_run = FirstRunStep::Brain;
            a.detected = Some(demo::detected());
            a.state.backend = Some("Claude Code".into());
            a.backends.clear();
            a
        });
    }
    if want("first-run-purpose") {
        render(&out, "first-run-purpose", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, false));
            a.state.first_run = FirstRunStep::Purpose;
            a
        });
    }
    if want("beginner-home") {
        render(&out, "beginner-home", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            a.state.screen = Screen::Home;
            a
        });
    }
    if want("beginner-ask") {
        render(&out, "beginner-ask", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_ask(&mut a);
            a
        });
    }
    if want("ask-working") {
        render(&out, "ask-working", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_ask(&mut a);
            a.state.chat.truncate(2);
            // A visible job keeps the app busy, so the working row shows.
            a.jobs.spawn("Wait", true, |_| {
                std::thread::sleep(std::time::Duration::from_secs(30));
                Err(String::new())
            });
            a
        });
    }
    if want("experienced") {
        render(&out, "experienced", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Experienced, true));
            demo::fill_workbench(&mut a);
            a
        });
    }
    if want("experienced-hex") {
        render(&out, "experienced-hex", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Experienced, true));
            demo::fill_workbench(&mut a);
            a.state.center = craft_anything_ui_egui::state::CenterTab::Hex;
            a.state.views.clear();
            a.state.right = craft_anything_ui_egui::state::RightTab::Inspector;
            a.state.bottom = craft_anything_ui_egui::state::BottomTab::Jobs;
            a
        });
    }
    if want("first-run-done") {
        render(&out, "first-run-done", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, false));
            a.state.first_run = FirstRunStep::Done;
            a.state.backend = Some("Claude Code".into());
            a
        });
    }
    if want("settings") {
        render(&out, "settings", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_projects(&mut a);
            a.state.settings_open = true;
            a
        });
    }
    if want("palette") {
        render(&out, "palette", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Experienced, true));
            demo::fill_workbench(&mut a);
            a.state.palette.open = true;
            a
        });
    }
    if want("simple-home") {
        render(&out, "simple-home", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_projects(&mut a);
            a
        });
    }
    if want("sidebar-folded") {
        render(&out, "sidebar-folded", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_projects(&mut a);
            a.state.sidebar_folded = true;
            a
        });
    }
    if want("project") {
        render(&out, "project", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_project(&mut a);
            // What the build took and used (side panel and title bar).
            let mut u = craft_anything_engine::workspace::ProjectUsage { work_ms: 725_000, ..Default::default() };
            u.add_turn(1_180_000, 42_000, Some(3.21), true);
            if let Some(root) = a.workspace.as_ref().map(|w| w.root.clone()) {
                a.usage.insert(root, craft_anything_ui_egui::usage::ProjectClock { saved: u.clone(), since: None });
            }
            a.session_usage = u;
            a
        });
    }
    // Notes sent while it builds, waiting under the conversation until the builder reads them.
    if want("project-notes") {
        render(&out, "project-notes", w, h, scale, || {
            use craft_anything_ai::rebuild::{RunOptions, RunSet, StandIns};
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_project(&mut a);
            a.state.feed.retain(|i| !matches!(i, craft_anything_ui_egui::state::FeedItem::Ask { .. }));
            a.runs = RunSet::new(1).with_starter(StandIns::new().starter());
            if let Some(ws) = a.workspace.clone() {
                let root = ws.root.clone();
                let _ = a.runs.start(&a.hub, ws, RunOptions::default());
                if let Some(run) = a.runs.get(&root) {
                    run.add_note("Make the header blue".into(), "Make the header blue".into(), Vec::new());
                    run.add_note("And add a contact form".into(), "And add a contact form".into(), Vec::new());
                }
            }
            a
        });
    }
    // A build the app closed on mid-way: the Continue card.
    if want("project-interrupted") {
        render(&out, "project-interrupted", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_project(&mut a);
            if let Some(ws) = a.workspace.as_mut() {
                ws.manifest.status = craft_anything_engine::workspace::Status::Building;
            }
            a.state.feed.retain(|i| !matches!(i, craft_anything_ui_egui::state::FeedItem::Ask { .. }));
            a
        });
    }
    // Game files: a game's archives, one pak open, and the permission question before saving.
    let game_app = |asking: bool| {
        use serde_json::json;
        let mut a = demo::app(demo::engine(Mode::Beginner, true));
        demo::fill_projects(&mut a);
        demo::deep(&mut a);
        a.state.screen = craft_anything_ui_egui::state::Screen::GameFiles;
        let g = &mut a.state.game;
        g.folder = Some(PathBuf::from("D:/Games/Cave Game"));
        g.archives = vec![
            json!({"path": "D:/Games/Cave Game/CaveGame/Content/Paks/CaveGame-WindowsNoEditor.pak", "name": "CaveGame/Content/Paks/CaveGame-WindowsNoEditor.pak", "kind": "unreal_pak", "size": 2_492_378_800u64, "list_tool": "game_pak_list"}),
            json!({"path": "D:/Games/Cave Game/Engine/Content/Paks/Engine.utoc", "name": "Engine/Content/Paks/Engine.utoc", "kind": "unreal_iostore", "size": 1_204_000u64, "list_tool": "game_iostore_list"}),
        ];
        g.profile = Some(json!({"game": {
            "engine": "Unity 2022.3.62f2",
            "found": {"note": "Its C# code was compiled to native code (IL2CPP), so it reads like any native program."},
            "stack": [
                {"name": "URP", "what": "rendering"},
                {"name": "FishNet", "what": "multiplayer"},
                {"name": "Steamworks.NET", "what": "Steam features"},
                {"name": "A* Pathfinding Project", "what": "characters finding their way"},
                {"name": "TextMeshPro", "what": "text"},
                {"name": "Input System", "what": "controls"},
                {"name": "Newtonsoft.Json", "what": "reading and writing data, often saves"}
            ],
            "other_groups": ["Assembly-CSharp-firstpass", "Assembly-CSharp", "CaveGame (3)"],
            "scenes": 3, "bytes": 6_764_000_000u64, "files": 109
        }}));
        g.open = Some("D:/Games/Cave Game/CaveGame/Content/Paks/CaveGame-WindowsNoEditor.pak".into());
        g.open_kind = "unreal_pak".into();
        g.rows = ["Arrow", "Logo", "Logo_Tagline", "Portrait_Captain", "HUD_Radar_Icon_Cart"]
            .iter()
            .map(|n| json!({"path": format!("CaveGame/Content/Art/UI/{n}.uasset"), "size": 930, "compression": "none"}))
            .collect();
        if asking {
            g.pending = Some(craft_anything_ui_egui::state::PendingPicture {
                archive: g.open.clone().unwrap_or_default(),
                kind: "unreal_pak".into(),
                item: json!({"path": "CaveGame/Content/Art/UI/Logo.uasset"}),
                name: "Logo.png".into(),
            });
            a.state.modal = Some(craft_anything_ui_egui::state::Modal::GamePermission);
        }
        a
    };
    if want("game-files") {
        render(&out, "game-files", w, h, scale, move || game_app(false));
    }
    if want("game-permission") {
        render(&out, "game-permission", w, h, scale, move || game_app(true));
    }
    // Game engines: a Unity game picked while Unity isn't here, the steps to get it, and a download.
    let engine_app = |modal: Option<craft_anything_engines::game::Builder>, downloading: bool| {
        use craft_anything_engines::game::{Builder, Found, Installed, Inventory, Made};
        let mut a = demo::app(demo::engine(Mode::Beginner, true));
        demo::fill_projects(&mut a);
        a.state.make.target = craft_anything_engine::workspace::TargetRef::Local { path: std::env::temp_dir() };
        a.state.make.game = true;
        a.state.make.request = "Recreate this game".into();
        a.state.make.found = Some(Found { engine: Made::Unity, version: Some("2021.3.16f1".into()), clue: "unityplayer.dll".into(), note: None });
        a.game_engines =
            Some(Inventory { engines: vec![Installed { builder: Builder::Rust, version: String::new(), program: "cargo".into() }], ..Default::default() });
        if let Some(b) = modal {
            a.state.modal = Some(craft_anything_ui_egui::state::Modal::Engine(b));
            a.engine_flow.start_after = true;
        }
        if downloading {
            let id = a.jobs.spawn("Getting Godot", true, |j| {
                j.report(craft_anything_engine::Progress { fraction: Some(0.42), message: "63 of 150 MB".into() });
                std::thread::sleep(std::time::Duration::from_secs(30));
                Ok(craft_anything_ui_egui::Done::Nothing)
            });
            a.engine_flow.job = Some((Builder::Godot, id));
        }
        a
    };
    if want("make-engine") {
        render(&out, "make-engine", w, h, scale, move || engine_app(None, false));
    }
    if want("engine-unity") {
        render(&out, "engine-unity", w, h, scale, move || engine_app(Some(craft_anything_engines::game::Builder::Unity), false));
    }
    if want("assets-offer") {
        render(&out, "assets-offer", w, h, scale, move || {
            let mut a = engine_app(None, false);
            a.asset_keys = Some([(craft_anything_engines::assets::fal::ID.to_string(), false)].into());
            a.state.make.generate_assets = true;
            a.state.modal = Some(craft_anything_ui_egui::state::Modal::Assets);
            a
        });
    }
    if want("look-inside") {
        render(&out, "look-inside", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_projects(&mut a);
            a.state.make.look = true;
            a
        });
    }
    if want("make-deep") {
        render(&out, "make-deep", w, h, scale, move || {
            let mut a = engine_app(None, false);
            a.settings.engines.store_dir = Some(std::env::temp_dir().join("ca-no-ghidra"));
            a.settings.deep_mode = true;
            a
        });
    }
    if want("make-deep-ready") {
        render(&out, "make-deep-ready", w, h, scale, move || {
            let mut a = engine_app(None, false);
            a.settings.deep_mode = true;
            a
        });
    }
    if want("ghidra-offer") {
        render(&out, "ghidra-offer", w, h, scale, move || {
            let mut a = engine_app(None, false);
            a.settings.engines.store_dir = Some(std::env::temp_dir().join("ca-no-ghidra"));
            a.settings.deep_mode = true;
            a.ghidra.start_after = Some(Box::new(a.state.make.clone()));
            a.state.modal = Some(craft_anything_ui_egui::state::Modal::Ghidra);
            a
        });
    }
    if want("settings-engines") {
        render(&out, "settings-engines", w, h, scale, move || {
            let mut a = engine_app(None, false);
            a.state.settings_open = true;
            a.state.settings_tab = craft_anything_ui_egui::state::SettingsTab::Engines;
            a
        });
    }
    // Settings → AI: each asset provider's key (none saved yet), and three AIs: one in use, one
    // building, one extracting a game's files; settings-ai-error adds a save that failed.
    let ai_app = move |error: bool| {
        use craft_anything_ai::{BackendConfig, BackendKind};
        let mut a = engine_app(None, false);
        a.state.settings_open = true;
        a.state.settings_tab = craft_anything_ui_egui::state::SettingsTab::Ai;
        a.asset_keys = Some([(craft_anything_engines::assets::fal::ID.to_string(), false)].into());
        let b = |name: &str, kind, model: &str| BackendConfig { name: name.into(), kind, model: model.into(), base_url: None, ssh_host: None, enabled: true };
        a.backends = vec![
            b("Claude Code", BackendKind::AgentCli, "claude-code"),
            b("Kimi K3.1 Pro (OpenRouter)", BackendKind::Openrouter, "moonshotai/kimi-k3.1-pro"),
            b("Claude", BackendKind::Anthropic, "claude-opus-5-5"),
        ];
        a.state.backend = Some("Claude".into());
        a.builder = Some("Claude Code".into());
        a.extractor = Some("Kimi K3.1 Pro (OpenRouter)".into());
        if error {
            a.toast("Couldn't save C:/Users/me/AppData/Roaming/craft-anything/config.toml: access is denied.", true);
        }
        a
    };
    if want("settings-ai") {
        render(&out, "settings-ai", w, h, scale, move || ai_app(false));
    }
    if want("settings-ai-error") {
        render(&out, "settings-ai-error", w, h, scale, move || ai_app(true));
    }
    for scene in ["composer-selected", "composer-picker", "composer-empty", "composer-empty-picker", "composer-project", "composer-make"] {
        if want(scene) {
            render(&out, scene, w, h, scale, move || {
                let mut a = demo::app(demo::engine(Mode::Beginner, true));
                if scene == "composer-project" {
                    demo::fill_project(&mut a);
                }
                demo::composer_scene(&mut a, scene.starts_with("composer-empty"));
                if scene == "composer-project" {
                    a.state.screen = Screen::Project;
                    a.state.project_input = "Add a dark theme without changing the layout.".into();
                } else if scene == "composer-make" {
                    a.state.screen = Screen::Home;
                    a.state.make.request = "Build a notes app with keyboard shortcuts.".into();
                }
                a
            });
        }
    }
    for scene in ["providers-empty", "providers-search", "providers-connect"] {
        if want(scene) {
            render(&out, scene, w, h, scale, move || {
                let mut a = engine_app(None, false);
                a.state.settings_open = true;
                a.state.settings_tab = craft_anything_ui_egui::state::SettingsTab::Ai;
                a.backends.clear();
                a.detected = Some(Vec::new());
                demo::provider_accounts(&mut a);
                a
            });
        }
    }
    if want("providers-list") {
        render(&out, "providers-list", w, h, scale, move || {
            let mut a = ai_app(false);
            demo::provider_accounts(&mut a);
            a
        });
    }
    if want("providers-detail") {
        render(&out, "providers-detail", w, h, scale, move || {
            let mut a = ai_app(false);
            demo::provider_accounts(&mut a);
            a
        });
    }
    for (scene, state) in [
        ("providers-narrow-accounts", "accounts"),
        ("providers-narrow-picker", "picker"),
        ("providers-narrow-login", "login"),
        ("providers-narrow-error", "error"),
    ] {
        if want(scene) {
            render(&out, scene, 760.0, 520.0, scale, move || {
                let mut a = engine_app(None, false);
                a.state.settings_open = true;
                a.state.settings_tab = craft_anything_ui_egui::state::SettingsTab::Ai;
                demo::provider_account_scene(&mut a, state);
                a
            });
        }
    }
    if want("engine-godot") {
        render(&out, "engine-godot", w, h, scale, move || engine_app(Some(craft_anything_engines::game::Builder::Godot), true));
    }
    if want("project-game") {
        // A finished game: a workspace with what `cargo build` leaves behind (nothing is run).
        use craft_anything_engine::workspace::{Status, TargetRef, Workspace};
        let root = out.join("demo-projects");
        let _ = std::fs::remove_dir_all(&root);
        if let Ok(mut ws) = Workspace::create(&root, "Snake", "A snake game", TargetRef::None) {
            let debug = ws.app_dir().join("target/debug");
            let _ = std::fs::create_dir_all(&debug);
            let _ = std::fs::write(ws.app_dir().join("Cargo.toml"), "[package]");
            let _ = std::fs::write(debug.join(if cfg!(windows) { "snake.exe" } else { "snake" }), "x");
            ws.manifest.status = Status::Ready;
            ws.manifest.stack = "Rust + macroquad".into();
            let _ = ws.save();
            render(&out, "project-game", w, h, scale, move || {
                let mut a = demo::app(demo::engine(Mode::Beginner, true));
                a.state.feed = vec![
                    craft_anything_ui_egui::state::FeedItem::User(craft_anything_ui_egui::state::UserItem::Text("A snake game".into())),
                    craft_anything_ui_egui::state::FeedItem::Agent {
                        text: "Your snake game is ready: arrow keys to steer, eat the dots to grow, and the speed goes up as you score.".into(),
                        streaming: false,
                    },
                ];
                a.workspace = Some(ws);
                a.state.screen = Screen::Project;
                a
            });
        }
    }
    if want("project-plan") {
        render(&out, "project-plan", w, h, scale, || {
            use craft_anything_ui_egui::state::{FeedItem, PlanStatus, PlanStep};
            let step = |text: &str, status| PlanStep { text: text.into(), doing: text.into(), status };
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_project(&mut a);
            a.state.feed.push(FeedItem::Plan(vec![
                step("Study the page's layout and colours", PlanStatus::Done),
                step("Build the heading and the card", PlanStatus::Done),
                step("Add the contact form", PlanStatus::Doing),
                step("Match the spacing on phones", PlanStatus::Todo),
            ]));
            a
        });
    }
    if want("project-long-prompt") {
        render(&out, "project-long-prompt", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            demo::fill_project(&mut a);
            let text =
                format!("{}https://example.com/{}", "Make it exactly like the original, with the same menus, weapons and maps. ".repeat(6), "a".repeat(160));
            a.state.feed.push(craft_anything_ui_egui::state::FeedItem::User(craft_anything_ui_egui::state::UserItem::Text(text)));
            a
        });
    }
    if want("light-project") {
        render(&out, "light-project", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            a.settings.theme = "studio-light".into();
            demo::fill_project(&mut a);
            a
        });
    }
    if want("light-home") {
        render(&out, "light-home", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            a.settings.theme = "studio-light".into();
            a
        });
    }
    if want("light-ask") {
        render(&out, "light-ask", w, h, scale, || {
            let mut a = demo::app(demo::engine(Mode::Beginner, true));
            a.settings.theme = "studio-light".into();
            demo::fill_ask(&mut a);
            a
        });
    }
    for (kind, name) in [(ThemeKind::Studio, "kit"), (ThemeKind::StudioLight, "kit-light")] {
        if want(name) {
            render_kit(&out, name, w, 1100.0, scale, kind);
        }
    }
}

/// The UI kit on one page: every token and widget in one theme, the reference a redesign is
/// checked against (`--only kit`).
fn render_kit(out: &std::path::Path, name: &str, w: f32, h: f32, scale: f32, kind: ThemeKind) {
    let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(w, h)).with_pixels_per_point(scale).with_max_steps(64).wgpu().build_ui_state(
        move |ui, ready: &mut bool| {
            if !*ready {
                CraftApp::setup_context(ui.ctx());
                theme::apply(ui.ctx(), kind, false);
                craft_anything_ui_egui::i18n::set_language(LANG.get().map_or("en", String::as_str));
                *ready = true;
                // The fonts are bound from the next frame on.
                return;
            }
            let t = Tokens::get(ui.ctx());
            egui::Frame::new().fill(t.canvas).inner_margin(theme::pad(space::XL)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.columns(3, |cols| {
                    kit_tokens(&mut cols[0], &t);
                    kit_controls(&mut cols[1], &t);
                    kit_surfaces(&mut cols[2], &t);
                });
            });
        },
        false,
    );
    for _ in 0..24 {
        harness.step();
    }
    match harness.render() {
        Ok(img) => match img.save(out.join(format!("{name}.png"))) {
            Ok(()) => println!("wrote {}", out.join(format!("{name}.png")).display()),
            Err(e) => eprintln!("{name}: {e}"),
        },
        Err(e) => eprintln!("{name}: render failed: {e}"),
    }
}

fn kit_heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(space::M);
    ui.label(theme::heading(text, theme::SECTION));
    ui.add_space(space::XS);
}

fn kit_tokens(ui: &mut egui::Ui, t: &Tokens) {
    kit_heading(ui, "Colour");
    for row in [
        [("canvas", t.canvas), ("chrome", t.chrome), ("card", t.card), ("raised", t.raised)],
        [("text", t.text), ("text_dim", t.text_dim), ("text_faint", t.text_faint), ("icon", t.icon)],
        [("accent", t.accent), ("accent_ink", t.accent_ink), ("accent_soft", t.accent_soft), ("separator", t.separator)],
        [("success", t.success), ("warning", t.warning), ("danger", t.danger), ("field_border", t.field_border)],
    ] {
        ui.horizontal(|ui| {
            for (name, c) in row {
                ui.vertical(|ui| {
                    let (r, _) = ui.allocate_exact_size(egui::vec2(size::TILE_LG * 2.0, size::TILE), egui::Sense::hover());
                    ui.painter().rect(r, t.radius_sm, c, theme::hairline(t.card_border), egui::StrokeKind::Inside);
                    ui.label(egui::RichText::new(name).size(theme::CAPTION).color(t.text_faint));
                });
            }
        });
    }
    kit_heading(ui, "Type");
    for (name, s) in [("TITLE", theme::TITLE), ("SECTION", theme::SECTION), ("CARD", theme::CARD)] {
        ui.label(theme::heading(format!("{name} {s}"), s));
    }
    for (name, s) in [("BODY", theme::BODY), ("SMALL", theme::SMALL), ("CAPTION", theme::CAPTION), ("TINY", theme::TINY)] {
        ui.label(egui::RichText::new(format!("{name} {s} · 빠른 갈색 여우")).size(s).color(t.text));
    }
    kit_heading(ui, "Space");
    for (name, s) in [
        ("XXS", space::XXS),
        ("XS", space::XS),
        ("S", space::S),
        ("SM", space::SM),
        ("M", space::M),
        ("L", space::L),
        ("XL", space::XL),
        ("XXL", space::XXL),
        ("XXXL", space::XXXL),
    ] {
        ui.horizontal(|ui| {
            ui.add_sized([size::TILE_LG, size::ROW_DENSE], egui::Label::new(egui::RichText::new(name).size(theme::CAPTION).color(t.text_faint)));
            let (r, _) = ui.allocate_exact_size(egui::vec2(s, space::SM), egui::Sense::hover());
            ui.painter().rect_filled(r, 0.0, t.accent);
        });
    }
    kit_heading(ui, "Icons");
    ui.horizontal(|ui| {
        for s in [size::ICON_XS, size::ICON_SM, size::ICON, size::ICON_LG, size::ICON_XL, size::ICON_HERO] {
            icons::show(ui, "sparkles", s, t.icon);
        }
    });
}

fn kit_controls(ui: &mut egui::Ui, t: &Tokens) {
    kit_heading(ui, "Buttons");
    ui.horizontal_wrapped(|ui| {
        widgets::primary_icon(ui, Some("sparkles"), "Start", true);
        widgets::primary_icon(ui, None, "Disabled", false);
        widgets::secondary(ui, Some("folder-open"), "Open folder");
        widgets::link(ui, "Change…");
        icons::button(ui, "x", size::CONTROL_SM, false, "Close");
        icons::button(ui, "pin", size::CONTROL_SM, true, "Pinned");
    });
    ui.add_space(space::S);
    widgets::primary_sized(ui, Some("sparkles"), "Main action", true, size::CONTROL_XL, 14.0);
    kit_heading(ui, "Choices");
    ui.horizontal_wrapped(|ui| {
        widgets::choice_chip(ui, "globe", "A website", true);
        widgets::choice_chip(ui, "app-window", "An app", false);
        widgets::choice_chip(ui, "puzzle", "A game", false);
    });
    ui.add_space(space::S);
    widgets::segmented(ui, &["Simple", "Advanced"], 0, size::CONTROL);
    ui.add_space(space::S);
    let mut on = true;
    widgets::toggle(ui, &mut on, "Explain like I'm new");
    kit_heading(ui, "Fields");
    let mut text = String::from("https://example.com");
    ui.add(egui::TextEdit::singleline(&mut text).desired_width(f32::INFINITY).frame(widgets::text_field_frame(t)));
    ui.add_space(space::S);
    let mut dense = String::from("0x401a30");
    ui.add(egui::TextEdit::singleline(&mut dense).desired_width(f32::INFINITY).frame(widgets::text_field_frame_compact(t)));
    kit_heading(ui, "Status");
    ui.horizontal_wrapped(|ui| {
        widgets::pill(ui, "Ready", t.success, t.raised);
        widgets::pill(ui, "Needs you", t.warning, t.raised);
        widgets::pill(ui, "Deep", t.accent_ink, t.accent_soft);
        widgets::confidence_badge(ui, Some(Confidence::Observed));
        widgets::confidence_badge(ui, Some(Confidence::Inferred));
        widgets::confidence_badge(ui, None);
    });
    ui.add_space(space::S);
    widgets::working(ui, "Building", Some("Running cargo test"));
}

fn kit_surfaces(ui: &mut egui::Ui, t: &Tokens) {
    kit_heading(ui, "Properties");
    widgets::panel(t).inner_margin(theme::pad(space::M)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            widgets::icon_tile(ui, "globe", size::TILE);
            ui.label(theme::heading("example.com", theme::CARD));
        });
        widgets::property_text(ui, "From", "https://example.com");
        widgets::property_text(ui, "Built with", "React + Vite");
        widgets::section(ui, "Usage", false);
        widgets::property_text(ui, "Time spent", "12m 05s");
        widgets::property_text(ui, "Tokens", "1.2M");
    });
    kit_heading(ui, "Cards");
    widgets::card(t).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(theme::heading("Card", theme::CARD));
        ui.label(egui::RichText::new("A soft rounded box with a hairline edge.").color(t.text_dim));
    });
    ui.add_space(space::S);
    widgets::callout(t, t.warning).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(theme::heading("May I run a command?", theme::CARD));
    });
    ui.add_space(space::S);
    widgets::callout_soft(t).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label("A soft note on the accent wash.");
    });
    ui.add_space(space::S);
    widgets::code_block(t).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(egui::RichText::new("void main() { return 0; }").font(theme::mono(theme::SMALL)));
    });
    kit_heading(ui, "Empty state");
    widgets::empty_state(ui, "folder-open", "Nothing here yet", "What you make shows up here.");
}
