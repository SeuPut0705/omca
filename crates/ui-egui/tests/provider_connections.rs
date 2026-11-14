//! Managed account setup through an isolated provider runtime.

#![cfg(unix)]

use craft_anything_ai::{AiHub, KeyStore};
use craft_anything_engine::Mode;
use craft_anything_ui_egui::{CraftApp, demo, state::SettingsTab};
use egui_kittest::{Harness, kittest::Queryable};
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[track_caller]
fn wait(h: &mut Harness<'static, CraftApp>, done: impl Fn(&Harness<'static, CraftApp>) -> bool) {
    let start = Instant::now();
    while !done(h) {
        assert!(start.elapsed() < Duration::from_secs(10), "provider workflow timed out: {:?}; {:?}", h.state().state.toasts, h.state().state.chat);
        h.step();
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn managed_key_login_selection_and_signout_use_the_runtime() {
    let dir = tempfile::tempdir().unwrap();
    let helper = dir.path().join("provider-runtime");
    std::fs::write(&helper, r#"#!/bin/sh
IFS= read -r request
mkdir -p "$OMCA_PROVIDER_DATA_DIR"
state="$OMCA_PROVIDER_DATA_DIR/test-account"
models='[{"provider_id":"fake","id":"fake-model","name":"Fake model","reasoning":false},{"provider_id":"fake","id":"fake-next","name":"Fake next model","reasoning":false}]'
empty='{"providers":[{"id":"fake","name":"Fake provider","login_available":true,"authenticated":false,"auth_kind":"api_key","accounts":[]}],"models":[]}'
account='{"providers":[{"id":"fake","name":"Fake provider","login_available":true,"authenticated":true,"auth_kind":"api_key","accounts":[{"id":7,"label":"Fake account","kind":"api_key","selected":true,"in_use":false,"state":"available","limits":[{"id":"weekly","label":"Weekly","used_fraction":0.5}]}]}],"models":[]}'
case "$request" in
  *'"command":"turn"'*)
    case "$request" in *'"model_id":"fake-next"'*) text='New model answered' ;; *) text='Original model answered' ;; esac
    printf '{"id":"1","kind":"delta","delta":{"type":"text","text":"%s"}}\n' "$text"
    printf '{"id":"1","kind":"result","data":{"message":{"role":"assistant","blocks":[{"type":"text","text":"%s"}]},"stop":"end_turn","usage":{"input_tokens":1,"output_tokens":1,"cache_read_tokens":0,"cache_write_tokens":0},"model":"fake"}}\n' "$text"
    exit ;;
  *'"command":"models"'*) printf '{"id":"1","kind":"result","data":%s}\n' "$models"; exit ;;
  *'"command":"set_key"'*)
    case "$request" in *bad-key*) echo '{"id":"1","kind":"error","error":"rejected"}'; exit ;; esac
    : > "$state" ;;
  *'"command":"login"'*)
    echo '{"id":"1","kind":"login","event":{"type":"confirm","request_id":"risk","title":"Risk confirmation","message":"Approve only if you agree"}}'
    IFS= read -r answer
    case "$answer" in *'"accepted":true'*) : > "$state" ;; *) echo '{"id":"1","kind":"error","error":"declined"}'; exit ;; esac ;;
  *'"command":"remove_account"'*) rm -f "$state" ;;
esac
if [ -f "$state" ]; then data="$account"; else data="$empty"; fi
printf '{"id":"1","kind":"result","data":%s}\n' "$data"
"#).unwrap();
    std::fs::set_permissions(&helper, std::fs::Permissions::from_mode(0o700)).unwrap();
    if std::env::var_os("OMCA_ACCOUNT_TEST_CHILD").is_none() {
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "managed_key_login_selection_and_signout_use_the_runtime", "--nocapture"])
            .env("OMCA_ACCOUNT_TEST_CHILD", "1")
            .env("OMCA_PROVIDER_RUNTIME", &helper)
            .status()
            .unwrap();
        assert!(status.success(), "isolated account workflow failed");
        return;
    }
    let path = dir.path().to_owned();
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 820.0)).with_max_steps(32).build_eframe(move |cc| {
        CraftApp::setup_context(&cc.egui_ctx);
        let engine = demo::engine(Mode::Experienced, true);
        let mut app = demo::app(engine.clone());
        app.hub = Arc::new(AiHub::with_stores(engine, path.join("config.toml"), KeyStore::file_only(path.join("keys.toml"))));
        app.backends.clear();
        app.state.settings_open = true;
        app.state.settings_tab = SettingsTab::Ai;
        app
    });
    h.run_steps(3);
    h.get_by_label("Add account").click();
    wait(&mut h, |h| h.query_by_label("Fake provider").is_some());
    h.get_by_label("Search providers").click();
    h.run_steps(2);
    h.get_by_label("Search providers").type_text("Fake");
    h.run_steps(2);
    h.get_by_label("API key").click();
    h.run_steps(2);
    h.get_by_label("Enter API key").focus();
    h.get_by_label("Enter API key").type_text("bad-key");
    h.run_steps(2);
    h.get_by_label("Save API key").click();
    wait(&mut h, |h| h.query_by_label_contains("rejected").is_some());
    assert!(h.state().backends.is_empty());
    h.get_by_label("Enter API key").focus();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.get_by_label("Enter API key").type_text("good-key");
    h.run_steps(2);
    h.get_by_label("Save API key").click();
    wait(&mut h, |h| h.query_by_label("Fake model").is_some());
    h.get_by_label("Fake model").click();
    wait(&mut h, |h| h.state().backends.iter().any(|b| b.model == "fake-model"));
    h.state_mut().send("First question");
    wait(&mut h, |h| {
        h.state()
            .state
            .chat
            .iter()
            .any(|item| matches!(item, craft_anything_ui_egui::state::ChatItem::Assistant { text, .. } if text.contains("Original model answered")))
    });
    h.get_by_label("Choose model").click();
    wait(&mut h, |h| h.query_by_label("Fake next model").is_some());
    h.get_by_label("Fake next model").click();
    wait(&mut h, |h| h.state().backends.iter().any(|b| b.model == "fake-next"));
    h.state_mut().send("Second question");
    wait(&mut h, |h| {
        h.state()
            .state
            .chat
            .iter()
            .any(|item| matches!(item, craft_anything_ui_egui::state::ChatItem::Assistant { text, .. } if text.contains("New model answered")))
    });
    h.get_by_label("Fake account");
    h.get_by_label_contains("50%");
    h.get_by_label("Account actions").click();
    h.run_steps(2);
    h.get_by_label("Sign out").click();
    h.run_steps(2);
    h.get_by_label("Keep account").click();
    h.run_steps(2);
    h.get_by_label("Fake account");
    h.get_by_label("Account actions").click();
    h.run_steps(2);
    h.get_by_label("Sign out").click();
    h.run_steps(2);
    h.get_by_label("Confirm sign out").click();
    wait(&mut h, |h| h.query_by_label("Fake account").is_none());
    h.get_by_label("Add account").click();
    h.run_steps(2);
    h.get_by_label("Login").click();
    wait(&mut h, |h| h.query_by_label("Risk confirmation").is_some());
    h.get_by_label("Decline").click();
    wait(&mut h, |h| h.query_by_label_contains("declined").is_some());
    assert!(h.query_by_label("Fake account").is_none());
    h.get_by_label("Login").click();
    wait(&mut h, |h| h.query_by_label("Risk confirmation").is_some());
    h.get_by_label("Accept").click();
    wait(&mut h, |h| h.query_by_label("Fake account").is_some());
}
