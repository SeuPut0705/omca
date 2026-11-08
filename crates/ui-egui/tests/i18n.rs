//! Keep locale-changing UI checks in one integration-test executable: i18n is process-wide.

use craft_anything_engine::{Mode, Session, Settings};
use craft_anything_ui_egui::{CraftApp, demo, i18n};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use std::path::Path;

fn harness(mut settings: Settings, dir: &Path) -> Harness<'static, CraftApp> {
    let dir = dir.to_path_buf();
    // Keep every engine path under the test directory, including after a saved-settings reload.
    settings.engines.store_dir = Some(dir.join("engines"));
    settings.projects_dir = Some(dir.join("projects"));
    Harness::builder().with_size(egui::vec2(1280.0, 1120.0)).with_max_steps(32).build_eframe(move |cc| {
        CraftApp::setup_context(&cc.egui_ctx);
        let engine = demo::engine(Mode::Beginner, true);
        {
            // The home/settings scenario needs no sample target. Isolate the session and set
            // its language BEFORE CraftApp::new (called by demo::app) reads its settings.
            let mut session = engine.session();
            *session = Session::isolated(&dir);
            session.settings = settings;
        }
        let mut app = demo::app(engine);
        // demo::app keeps persistence disabled and uses an isolated, file-only AI key store.
        // Never enable the real settings worker in this test: Settings::save uses the global
        // config path. The roundtrip below uses save_to with an explicit temporary path.
        assert!(!app.persist);
        assert!(app.headless);
        app.state.settings_open = true;
        app
    })
}

#[test]
fn korean_startup_language_menu_and_saved_selection() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path().join("ko");
    let settings = Settings { mode: Mode::Beginner, first_run_done: true, language: "ko".into(), ..Default::default() };
    let mut h = harness(settings, &dir);
    h.run_steps(3);

    // Startup must apply the saved locale itself, rather than relying on a manual call to
    // set_language after app construction. Check both lookup and a rendered settings row.
    assert_eq!(i18n::language(), "ko");
    h.get_by_label("언어");
    assert_eq!(i18n::trf("Opened {name}", &[("name", "sample.exe")]), "sample.exe 열림");
    assert_eq!(i18n::tr("A deliberately uncatalogued message"), "A deliberately uncatalogued message");
    assert_eq!(i18n::trf("Uncatalogued {name}", &[("name", "sample.exe")]), "Uncatalogued sample.exe");
    assert_eq!(craft_anything_ui_egui::usage::asset_cost_words(0.0031), "$0.01 미만");
    let kinds = craft_anything_ai::rebuild::KindRegistry::built_in();
    for (suggestion, expected) in [("A website for my bakery", "website"), ("A simple snake game", "game")] {
        let translated = i18n::tr(suggestion);
        assert_ne!(translated, suggestion);
        let (kind, _) = kinds.choose(&translated, &craft_anything_engine::workspace::TargetRef::None, false).unwrap();
        assert_eq!(kind.id(), expected, "{translated}");
    }
    let title = h.state().engine.registry().get("native_functions_list").unwrap().title;
    let translated_title = i18n::tr(title);
    assert_ne!(translated_title, title);
    h.state_mut().state.settings_open = false;
    h.state_mut().state.palette.open = true;
    h.state_mut().state.palette.query = translated_title.clone();
    h.run_steps(3);
    h.get_by_label(&translated_title);
    h.state_mut().state.palette.open = false;
    h.state_mut().state.settings_open = true;
    h.run_steps(3);

    // Exercise the real ComboBox and selectable row, including the language-change callback.
    h.get_by_value("한국어").click();
    h.run();
    h.get_by_label("English").scroll_to_me();
    h.run();
    h.get_by_label("English").click();
    h.run_steps(3);
    assert_eq!(i18n::language(), "en");
    assert_eq!(h.state().settings.language, "en");
    let selected = h.state().engine.session().settings.clone();
    assert_eq!(selected.language, "en", "the UI must propagate the choice to the engine session");
    h.get_by_label("Language");
    assert!(h.query_by_label("언어").is_none());

    // Verify the persisted settings representation and the next startup using only a
    // temporary config.toml. This does not claim to test the UI's debounce worker path.
    let config = dir.join("config.toml");
    selected.save_to(&config).unwrap();
    let loaded = Settings::load_from(&config);
    assert_eq!(loaded.language, "en");
    drop(h);
    let mut restored = harness(loaded, &dir);
    restored.run_steps(3);
    assert_eq!(i18n::language(), "en");

    // The menu offers exactly System, English and Korean; Korean takes effect immediately.
    restored.get_by_value("English").click();
    restored.run();
    for gone in ["日本語", "简体中文", "Deutsch", "Français"] {
        assert!(restored.query_by_label(gone).is_none(), "{gone} is not offered");
    }
    restored.get_by_label("한국어").scroll_to_me();
    restored.run();
    restored.get_by_label("한국어").click();
    restored.run_steps(3);
    assert_eq!(i18n::language(), "ko");
    restored.get_by_label("언어");

    restored.get_by_value("한국어").click();
    restored.run();
    restored.get_by_label("시스템").scroll_to_me();
    restored.run();
    restored.get_by_label("시스템").click();
    restored.run_steps(3);
    assert!(restored.state().settings.language.is_empty());
    let expected = sys_locale::get_locales().find_map(|tag| i18n::language_from_tag(&tag)).unwrap_or("en");
    assert_eq!(i18n::language(), expected);
    assert!(restored.state().engine.session().settings.language.is_empty());

    i18n::set_language("ja");
    assert_eq!(i18n::language(), "en", "a removed language falls back to English");
    assert_eq!(i18n::tr("Settings"), "Settings");
}
