//! Composer selection must match the provider and model used by the next turn.

#![cfg(unix)]

use craft_anything_ai::{AiHub, BackendConfig, BackendKind, Detected, KeyStore};
use craft_anything_engine::Mode;
use craft_anything_ui_egui::{
    CraftApp, demo,
    state::{ChatItem, Screen},
};
use egui_kittest::{Harness, kittest::Queryable};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn executable(path: &Path, text: &str) {
    std::fs::write(path, text).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
}

fn isolated(test: &str, connected: bool) -> bool {
    if std::env::var_os("OMCA_COMPOSER_TEST_CHILD").is_some() {
        return false;
    }
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let helper = dir.path().join("provider-runtime");
    executable(
        &helper,
        r##"#!/bin/sh
IFS= read -r request
/bin/mkdir -p "$OMCA_PROVIDER_DATA_DIR"
models='[{"provider_id":"openai-codex","id":"codex-initial","name":"Codex initial","reasoning":false},{"provider_id":"openai-codex","id":"codex-next","name":"Codex next","reasoning":false}]'
case "$request" in
  *'"command":"turn"'*)
    provider=$(printf '%s' "$request" | /usr/bin/sed -n 's/.*"provider_id":"\([^"]*\)".*/\1/p')
    model=$(printf '%s' "$request" | /usr/bin/sed -n 's/.*"model_id":"\([^"]*\)".*/\1/p')
    text="routed:$provider/$model"
    printf '%s\n' "$text" >> "$OMCA_PROVIDER_DATA_DIR/turns"
    printf '{"id":"1","kind":"delta","delta":{"type":"text","text":"%s"}}\n' "$text"
    printf '{"id":"1","kind":"result","data":{"message":{"role":"assistant","blocks":[{"type":"text","text":"%s"}]},"stop":"end_turn","usage":{"input_tokens":1,"output_tokens":1,"cache_read_tokens":0,"cache_write_tokens":0},"model":"%s"}}\n' "$text" "$model"
    exit ;;
  *'"command":"models"'*) printf '{"id":"1","kind":"result","data":%s}\n' "$models"; exit ;;
esac
if [ -f "$OMCA_PROVIDER_DATA_DIR/connected" ]; then
  printf '{"id":"1","kind":"result","data":{"providers":[{"id":"openai-codex","name":"Codex","login_available":true,"authenticated":true,"auth_kind":"oauth","accounts":[{"id":7,"label":"Codex account","kind":"oauth","selected":true,"in_use":false,"state":"available","limits":[]}]}],"models":%s}}\n' "$models"
else
  echo '{"id":"1","kind":"result","data":{"providers":[],"models":[]}}'
fi
"##,
    );
    executable(&bin.join("gemini"), &format!("#!/bin/sh\necho launched >> '{}'\nexit 41\n", dir.path().join("gemini-launched").display()));
    let providers = dir.path().join("providers");
    std::fs::create_dir(&providers).unwrap();
    if connected {
        std::fs::write(providers.join("connected"), "").unwrap();
    }
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env_clear()
        .env("OMCA_COMPOSER_TEST_CHILD", "1")
        .env("OMCA_TEST_ROOT", dir.path())
        .env("OMCA_PROVIDER_RUNTIME", helper)
        .env("HOME", dir.path())
        .env("PATH", format!("{}:/usr/bin:/bin", bin.display()))
        .status()
        .unwrap();
    assert!(status.success(), "isolated composer regression failed");
    assert!(!dir.path().join("gemini-launched").exists(), "the local Gemini CLI was launched");
    true
}

fn gemini() -> BackendConfig {
    BackendConfig { name: "Gemini CLI".into(), kind: BackendKind::AgentCli, model: "gemini".into(), base_url: None, ssh_host: None, enabled: true }
}

fn harness(connected: bool, project: bool) -> Harness<'static, CraftApp> {
    Harness::builder().with_size(egui::vec2(1280.0, 820.0)).with_max_steps(32).build_eframe(move |cc| {
        CraftApp::setup_context(&cc.egui_ctx);
        let root = std::path::PathBuf::from(std::env::var_os("OMCA_TEST_ROOT").unwrap());
        let engine = demo::engine(Mode::Beginner, true);
        let hub = AiHub::with_stores(engine.clone(), root.join("config.toml"), KeyStore::file_only(root.join("keys.toml"))).managed_only();
        hub.add_backend(gemini(), None).unwrap();
        let selected = connected.then(|| hub.connect_provider("openai-codex", "codex-initial").unwrap());
        let mut app = demo::app(engine);
        app.hub = Arc::new(hub);
        app.backends = app.hub.backends();
        app.backend_options = app.hub.options();
        app.detected = Some(vec![Detected { suggestion: gemini(), reason: "Found Gemini CLI".into(), needs_key: false }]);
        app.state.backend = selected;
        if project {
            let ws = craft_anything_engine::workspace::Workspace::create(
                &root.join("projects"),
                "Notes",
                "A notes app",
                craft_anything_engine::workspace::TargetRef::None,
            )
            .unwrap();
            app.workspace = Some(ws);
            app.state.screen = Screen::Project;
        } else {
            app.state.screen = Screen::Ask;
        }
        app
    })
}

#[track_caller]
fn wait(h: &mut Harness<'static, CraftApp>, done: impl Fn(&Harness<'static, CraftApp>) -> bool) {
    let start = Instant::now();
    while !done(h) {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "composer workflow timed out: {:?}; {:?}; backends={:?}; busy={}; picker_open={}",
            h.state().state.toasts,
            h.state().state.chat,
            h.state().hub.backends(),
            h.state().busy(),
            h.query_by_label("Search models").is_some()
        );
        h.step();
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn chat_model_picker_routes_the_next_turn_without_losing_the_draft() {
    if isolated("chat_model_picker_routes_the_next_turn_without_losing_the_draft", true) {
        return;
    }
    let mut h = harness(true, false);
    h.run_steps(3);
    wait(&mut h, |h| h.query_by_label_contains("Codex initial").is_some());
    h.state_mut().send("First question");
    wait(&mut h, |h| {
        h.state()
            .state
            .chat
            .iter()
            .any(|item| matches!(item, ChatItem::Assistant { text, streaming: false, .. } if text.contains("routed:openai-codex/codex-initial")))
    });
    h.get_by_label("Ask a question").focus();
    h.get_by_label("Ask a question").type_text("Keep this draft");
    h.run_steps(2);
    h.get_by_label_contains("Change model").click();
    wait(&mut h, |h| h.query_by_label("Search models").is_some());
    h.get_by_label("Search models").type_text("Codex next");
    h.run_steps(2);
    wait(&mut h, |h| h.query_by_label("Codex next").is_some());
    assert!(h.query_by_label_contains("Gemini").is_none());
    h.get_by_label("Codex next").click();
    wait(&mut h, |h| h.state().backends.iter().any(|b| b.model == "codex-next"));
    h.run_steps(3);
    h.get_by_label_contains("Codex next");
    assert_eq!(h.state().state.input, "Keep this draft");
    h.get_by_label("Ask a question").focus();
    h.key_press(egui::Key::Enter);
    wait(&mut h, |h| {
        h.state()
            .state
            .chat
            .iter()
            .any(|item| matches!(item, ChatItem::Assistant { text, streaming: false, .. } if text.contains("routed:openai-codex/codex-next")))
    });
    assert!(h.state().state.chat.iter().any(|item| matches!(item, ChatItem::User(text) if text == "Keep this draft")));
    h.state_mut().state.input = "Keep the keyboard draft".into();
    h.get_by_label_contains("Change model").click();
    h.run_steps(2);
    h.get_by_label("Search models").focus();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.get_by_label("Search models").type_text("initial");
    h.run_steps(2);
    h.key_press(egui::Key::Enter);
    wait(&mut h, |h| h.state().backends.iter().any(|b| b.model == "codex-initial"));
    assert_eq!(h.state().state.input, "Keep the keyboard draft");
    assert!(
        h.state().hub.config_to_change().unwrap().backends.iter().any(|entry| entry.config.name == "Gemini CLI"),
        "changing models must preserve saved settings"
    );
}

#[test]
fn empty_project_picker_offers_setup_and_escape_preserves_the_draft() {
    if isolated("empty_project_picker_offers_setup_and_escape_preserves_the_draft", false) {
        return;
    }
    let mut h = harness(false, true);
    h.run_steps(3);
    h.get_by_label("Ask for a change").focus();
    h.get_by_label("Ask for a change").type_text("Keep my build draft");
    h.run_steps(2);
    h.get_by_label_contains("Change model").click();
    wait(&mut h, |h| h.query_by_label("No provider accounts connected").is_some());
    h.get_by_label("Manage provider accounts");
    assert!(h.query_by_label_contains("Gemini").is_none());
    h.key_press(egui::Key::Escape);
    h.run_steps(3);
    assert!(h.query_by_label("Search models").is_none());
    assert_eq!(h.state().state.project_input, "Keep my build draft");
}
