//! Managed provider accounts and their transient login workflow.

use crate::i18n::tr;
use crate::theme::{self, Tokens, space};
use crate::{CraftApp, Done, widgets};
use craft_anything_ai::provider_runtime::{Action, Event, LoginEvent, LoginResponse, Snapshot, Task};
use egui::{RichText, Ui};

#[derive(Default)]
pub(crate) struct ComposerModelPicker {
    query: String,
    pending: Option<u64>,
    error: Option<String>,
    retry: Option<(String, String)>,
    open: bool,
    focused: usize,
}

#[derive(Default)]
pub(crate) struct AccountsState {
    snapshot: Option<Snapshot>,
    task: Option<Task>,
    error: Option<String>,
    search: String,
    adding: bool,
    key_provider: Option<String>,
    key: String,
    key_pending: bool,
    connecting: bool,
    prompt: Option<LoginEvent>,
    login_url: Option<(String, Option<String>)>,
    progress: Option<String>,
    input: String,
    model_provider: Option<String>,
    confirm_remove: Option<(String, u64)>,
    started: bool,
    picker: ComposerModelPicker,
    loaded_models: std::collections::HashSet<String>,
}
pub(crate) fn set_demo_snapshot(app: &mut CraftApp, snapshot: Snapshot) {
    app.provider_accounts.loaded_models.extend(snapshot.providers.iter().map(|p| p.id.clone()));
    app.provider_accounts.snapshot = Some(snapshot);
    app.provider_accounts.started = true;
}
pub(crate) fn set_demo_scene(app: &mut CraftApp, scene: &str) {
    let state = &mut app.provider_accounts;
    match scene {
        "picker" => state.adding = true,
        "login" => state.login_event(LoginEvent::OpenUrl {
            url: "https://example.invalid/sign-in".into(),
            instructions: Some("Open the sign-in page to continue.".into()),
        }),
        "error" => state.error = Some("Provider runtime unavailable.".into()),
        _ => {}
    }
}
impl AccountsState {
    fn login_event(&mut self, event: LoginEvent) {
        match event {
            LoginEvent::OpenUrl { url, instructions } => self.login_url = Some((url, instructions)),
            LoginEvent::Progress { message } => self.progress = Some(message),
            event @ (LoginEvent::Input { .. } | LoginEvent::Confirm { .. }) => self.prompt = Some(event),
        }
    }
}

fn start(app: &CraftApp, state: &mut AccountsState, action: Action) {
    if (app.busy() || app.creating.is_some()) && !matches!(action, Action::Snapshot { .. } | Action::Models { .. }) {
        return;
    }
    if state.task.is_none() && state.picker.pending.is_none() {
        state.connecting = matches!(action, Action::Login { .. } | Action::SetKey { .. });
        state.error = None;
        state.task = Some(app.hub.provider_client().start(action));
    }
}
pub(crate) fn selection_ready(app: &CraftApp) -> bool {
    if model_pending(app) || app.provider_accounts.task.is_some() {
        return false;
    }
    let Some(name) = app.state.backend.as_ref() else { return false };
    let Some(backend) = app.backends.iter().find(|b| b.enabled && b.kind == craft_anything_ai::BackendKind::Managed && &b.name == name) else { return false };
    let Some(provider_id) = app.backend_options.get(&backend.name).and_then(|o| o.provider_id.as_ref()) else { return false };
    let Some(snapshot) = app.provider_accounts.snapshot.as_ref() else { return false };
    let Some(provider) = snapshot.providers.iter().find(|p| &p.id == provider_id) else { return false };
    if !provider.authenticated || provider.disabled_reason.is_some() {
        return false;
    }
    let account_ok = match provider.accounts.iter().find(|a| a.selected) {
        Some(account) => account.state == "available",
        None => provider.accounts.iter().any(|a| a.state == "available"),
    };
    account_ok
        && app.provider_accounts.loaded_models.contains(provider_id)
        && snapshot.models.iter().any(|m| &m.provider_id == provider_id && m.id == backend.model)
}

pub(crate) fn model_pending(app: &CraftApp) -> bool {
    app.provider_accounts.picker.pending.is_some()
}

#[cfg(test)]
pub(crate) fn set_pending_for_test(app: &mut CraftApp, job: u64) {
    app.provider_accounts.picker.pending = Some(job);
}

pub(crate) fn failed_job(app: &mut CraftApp, job: u64, error: &str) {
    if app.provider_accounts.picker.pending == Some(job) {
        app.provider_accounts.picker.pending = None;
        app.provider_accounts.picker.error = Some(error.to_owned());
    }
}

pub(crate) fn commit(app: &mut CraftApp, provider_id: String, model_id: String) {
    if model_pending(app) || app.provider_accounts.task.is_some() || app.busy() || app.creating.is_some() {
        return;
    }
    app.backend_choices = app.backend_choices.wrapping_add(1);
    let choices = app.backend_choices;
    app.provider_accounts.picker.error = None;
    app.provider_accounts.picker.retry = Some((provider_id.clone(), model_id.clone()));
    let hub = app.hub.clone();
    let job = app.jobs.spawn(tr("Connect model"), true, move |_| Ok(Done::ManagedProvider { result: hub.connect_provider(&provider_id, &model_id), choices }));
    app.provider_accounts.picker.pending = Some(job);
}

pub(crate) fn completed(app: &mut CraftApp, job: u64, result: &Result<String, craft_anything_engine::evidence::ToolError>) -> bool {
    if app.provider_accounts.picker.pending != Some(job) {
        return false;
    }
    app.provider_accounts.picker.pending = None;
    app.provider_accounts.picker.error = result.as_ref().err().map(|e| e.for_people());
    if let (Ok(name), Some((_, model))) = (result, app.provider_accounts.picker.retry.as_ref()) {
        if let Some(backend) = app.backends.iter_mut().find(|b| &b.name == name) {
            backend.model.clone_from(model);
        }
        app.provider_accounts.picker.retry = None;
    }
    true
}

pub(crate) fn composer(app: &mut CraftApp, ui: &mut Ui) {
    composer_control(app, ui, "Change model");
}

fn composer_control(app: &mut CraftApp, ui: &mut Ui, accessible_name: &str) {
    let ready = selection_ready(app);
    let mut state = std::mem::take(&mut app.provider_accounts);
    if !state.started {
        state.started = true;
        start(app, &mut state, Action::Snapshot { refresh: false });
    }
    if state.task.is_some() || state.picker.pending.is_some() {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
    }
    let current =
        app.state.backend.as_ref().and_then(|name| app.backends.iter().find(|b| &b.name == name && b.kind == craft_anything_ai::BackendKind::Managed));
    let label = current
        .map(|b| {
            let provider = app
                .backend_options
                .get(&b.name)
                .and_then(|o| o.provider_id.as_ref())
                .and_then(|id| state.snapshot.as_ref().and_then(|s| s.providers.iter().find(|p| &p.id == id)))
                .map(|p| p.name.as_str())
                .unwrap_or(&b.name);
            let model = state
                .snapshot
                .as_ref()
                .and_then(|s| {
                    s.models
                        .iter()
                        .find(|m| m.id == b.model && app.backend_options.get(&b.name).and_then(|o| o.provider_id.as_deref()) == Some(m.provider_id.as_str()))
                })
                .map(|m| m.name.as_str())
                .unwrap_or(&b.model);
            format!("{provider} · {model}")
        })
        .unwrap_or_else(|| app.state.backend.as_ref().map(|name| format!("{} · {}", name, tr("Unavailable"))).unwrap_or_else(|| tr("Choose model")));
    let label = if current.is_some() && !ready {
        let status = if state.picker.pending.is_some() {
            tr("Connecting…")
        } else if state.snapshot.is_none() || state.task.is_some() {
            tr("Loading accounts…")
        } else {
            tr("Unavailable")
        };
        format!("{label} · {status}")
    } else {
        label
    };
    let busy = app.busy() || app.creating.is_some() || state.picker.pending.is_some();
    let mut selected = None;
    let mut manage = false;
    let menu = ui
        .add_enabled_ui(!busy, |ui| {
            ui.menu_button(&label, |ui| {
                let t = Tokens::get(ui.ctx());
                ui.set_width((ui.ctx().content_rect().width() - space::XL * 2.0).clamp(220.0, 420.0));
                let search_id = ui.make_persistent_id("model-search");
                let enter = ui.memory(|m| m.has_focus(search_id)) && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                let search = widgets::named(
                    ui.add(
                        egui::TextEdit::singleline(&mut state.picker.query)
                            .id(search_id)
                            .hint_text(tr("Search models"))
                            .desired_width(ui.available_width() - space::XL)
                            .frame(widgets::text_field_frame_compact(&t)),
                    ),
                    "Search models",
                );
                if !state.picker.open {
                    search.request_focus();
                    state.picker.open = true;
                }
                if search.changed() {
                    state.picker.focused = 0;
                }
                if state.task.is_some() {
                    ui.add(widgets::Spinner::new());
                }
                if let Some(error) = &state.picker.error {
                    ui.colored_label(Tokens::get(ui.ctx()).danger, error);
                    if ui.button(tr("Retry")).clicked() {
                        selected = state.picker.retry.clone();
                    }
                }
                if let Some(error) = &state.error {
                    ui.colored_label(Tokens::get(ui.ctx()).danger, error);
                    if ui.button(tr("Retry")).clicked() {
                        state.started = state.snapshot.is_some();
                        state.loaded_models.clear();
                        state.error = None;
                    }
                }
                if let Some(snapshot) = &state.snapshot {
                    let query = state.picker.query.to_lowercase();
                    let mut any = false;
                    let mut providers: Vec<_> = snapshot.providers.iter().filter(|p| p.authenticated && !p.accounts.is_empty()).collect();
                    providers.sort_by_key(|p| p.name.to_lowercase());
                    let has_accounts = !providers.is_empty();
                    let count = snapshot
                        .models
                        .iter()
                        .filter(|m| {
                            providers.iter().any(|p| {
                                p.id == m.provider_id
                                    && (p.name.to_lowercase().contains(&query)
                                        || m.name.to_lowercase().contains(&query)
                                        || m.id.to_lowercase().contains(&query))
                            })
                        })
                        .count();
                    if search.has_focus() && count > 0 {
                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
                            state.picker.focused = (state.picker.focused + 1).min(count - 1);
                        }
                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
                            state.picker.focused = state.picker.focused.saturating_sub(1);
                        }
                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Home)) {
                            state.picker.focused = 0;
                        }
                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::End)) {
                            state.picker.focused = count - 1;
                        }
                    }
                    let mut index = 0;
                    egui::ScrollArea::vertical().id_salt("composer-model-list").max_height((ui.ctx().content_rect().height() * 0.55).min(420.0)).show(
                        ui,
                        |ui| {
                            for provider in providers {
                                let mut models: Vec<_> = snapshot
                                    .models
                                    .iter()
                                    .filter(|m| {
                                        m.provider_id == provider.id
                                            && (provider.name.to_lowercase().contains(&query)
                                                || m.name.to_lowercase().contains(&query)
                                                || m.id.to_lowercase().contains(&query))
                                    })
                                    .collect();
                                models.sort_by_key(|m| m.name.to_lowercase());
                                if !models.is_empty() {
                                    ui.strong(&provider.name);
                                }
                                for model in models {
                                    any = true;
                                    let checked = current.is_some_and(|b| {
                                        b.model == model.id
                                            && app.backend_options.get(&b.name).and_then(|o| o.provider_id.as_deref()) == Some(provider.id.as_str())
                                    });
                                    let active = index == state.picker.focused && search.has_focus();
                                    let row = ui
                                        .add_enabled_ui(state.task.is_none() && provider.disabled_reason.is_none(), |ui| {
                                            widgets::model_option(ui, &model.name, checked, active)
                                        })
                                        .inner;
                                    if active && ui.input(|i| i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowUp)) {
                                        row.scroll_to_me(Some(egui::Align::Center));
                                    }
                                    if row.clicked() || (state.task.is_none() && enter && index == state.picker.focused) {
                                        if !checked {
                                            selected = Some((provider.id.clone(), model.id.clone()));
                                        }
                                        ui.close();
                                    }
                                    index += 1;
                                }
                            }
                        },
                    );
                    if !any && state.task.is_none() {
                        ui.label(tr(if !has_accounts {
                            "No provider accounts connected"
                        } else if !query.is_empty() {
                            "No models match your search"
                        } else {
                            "No models available for this provider."
                        }));
                    }
                } else if state.task.is_none() {
                    ui.label(tr("Choose model"));
                }
                ui.separator();
                if ui.button(tr("Manage provider accounts")).clicked() {
                    manage = true;
                    ui.close();
                }
            })
        })
        .inner;
    if menu.inner.is_none() {
        state.picker.open = false;
    }
    widgets::named(menu.response, &format!("{} · {label}", tr(accessible_name)))
        .on_hover_text(&label)
        .on_disabled_hover_text(tr("Wait for active work to finish before changing the model."));
    if state.task.is_none() && state.model_provider.is_none() {
        if let Some(provider) = state
            .snapshot
            .as_ref()
            .and_then(|s| s.providers.iter().find(|p| p.authenticated && !p.accounts.is_empty() && !state.loaded_models.contains(&p.id)))
            .map(|p| p.id.clone())
        {
            state.loaded_models.insert(provider.clone());
            state.model_provider = Some(provider.clone());
            start(app, &mut state, Action::Models { provider_id: provider });
        }
    }
    app.provider_accounts = state;
    if let Some((provider, model)) = selected {
        commit(app, provider, model);
    }
    if manage {
        app.state.settings_open = true;
        app.state.settings_tab = crate::state::SettingsTab::Ai;
    }
}

pub(crate) fn poll(app: &mut CraftApp) {
    let state = &mut app.provider_accounts;
    let mut finished = false;
    let mut task = state.task.take();
    if let Some(task) = &mut task {
        while let Some(event) = task.try_event() {
            match event {
                Event::Login(event) => state.login_event(event),
                Event::Finished(result) => {
                    finished = true;
                    state.prompt = None;
                    state.login_url = None;
                    state.progress = None;
                    match result {
                        Ok(snapshot) => {
                            state.loaded_models.clear();
                            state.snapshot = Some(snapshot);
                            if state.key_pending {
                                state.key.clear();
                                state.key_provider = None;
                            }
                            if state.connecting {
                                state.adding = false;
                            }
                        }
                        Err(error) => state.error = Some(error.for_people()),
                    }
                    state.key_pending = false;
                    state.connecting = false;
                }
                Event::Models(result) => {
                    finished = true;
                    match result {
                        Ok(models) => {
                            if let Some(snapshot) = &mut state.snapshot {
                                snapshot.models.retain(|m| Some(&m.provider_id) != state.model_provider.as_ref());
                                snapshot.models.extend(models);
                            }
                        }
                        Err(error) => state.error = Some(error.for_people()),
                    }
                    state.model_provider = None;
                }
            }
        }
    }
    state.task = if finished { None } else { task };
}

fn response(state: &mut AccountsState, request_id: String, value: Option<String>, accepted: Option<bool>, cancelled: bool) {
    if let Some(task) = &state.task {
        match task.respond(LoginResponse { request_id, value, accepted, cancelled }) {
            Ok(()) => {
                state.prompt = None;
                state.input.clear();
            }
            Err(error) => state.error = Some(error.for_people()),
        }
    }
}

fn login_prompt(state: &mut AccountsState, ui: &mut Ui) {
    if let Some((url, instructions)) = &state.login_url {
        if let Some(instructions) = instructions {
            ui.label(instructions);
        }
        if widgets::secondary(ui, None, "Open sign-in page").clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url.clone()));
        }
    }
    if let Some(message) = &state.progress {
        ui.label(message);
    }
    let Some(prompt) = state.prompt.clone() else { return };
    match prompt {
        LoginEvent::OpenUrl { .. } | LoginEvent::Progress { .. } => {}
        LoginEvent::Input { request_id, title, placeholder, secret } => {
            ui.label(title);
            ui.add(egui::TextEdit::singleline(&mut state.input).password(secret).hint_text(placeholder.unwrap_or_default()));
            if widgets::primary(ui, "Continue").clicked() {
                let value = std::mem::take(&mut state.input);
                response(state, request_id, Some(value), None, false);
            }
        }
        LoginEvent::Confirm { request_id, title, message } => {
            ui.label(RichText::new(title).strong());
            ui.label(message);
            ui.horizontal_wrapped(|ui| {
                if widgets::secondary(ui, None, "Decline").clicked() {
                    response(state, request_id.clone(), None, Some(false), false);
                }
                if widgets::primary(ui, "Accept").clicked() {
                    response(state, request_id.clone(), None, Some(true), false);
                }
            });
        }
    }
}

pub(crate) fn show(app: &mut CraftApp, ui: &mut Ui) {
    composer_control(app, ui, "Settings model");
    ui.add_space(space::M);
    let mut state = std::mem::take(&mut app.provider_accounts);
    if !state.started {
        state.started = true;
        start(app, &mut state, Action::Snapshot { refresh: false });
    }
    let t = Tokens::get(ui.ctx());
    if app.busy() || app.creating.is_some() {
        ui.label(RichText::new(tr("Wait for active work to finish before changing the model.")).color(t.warning));
        ui.disable();
    }
    if state.task.is_some() {
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
    }
    ui.horizontal_wrapped(|ui| {
        if widgets::secondary(ui, Some("plus"), "Add account").clicked() {
            state.adding = !state.adding;
            state.model_provider = None;
            state.key_provider = None;
            state.key.clear();
        }
        if widgets::secondary(ui, Some("refresh-cw"), "Refresh accounts").clicked() {
            start(app, &mut state, Action::Snapshot { refresh: true });
        }
        if state.task.is_some() {
            ui.add(widgets::Spinner::new());
            if widgets::secondary(ui, None, "Cancel").clicked()
                && let Some(task) = &state.task
            {
                task.cancel();
            }
        }
    });
    if let Some(error) = &state.error {
        ui.label(RichText::new(error).color(t.danger));
    }
    login_prompt(&mut state, ui);
    if let Some(provider_id) = state.key_provider.clone() {
        let label = ui.label(tr("Enter API key"));
        ui.add_enabled(
            !state.key_pending,
            egui::TextEdit::singleline(&mut state.key).password(true).hint_text(tr("API key")).desired_width(ui.available_width()),
        )
        .labelled_by(label.id);
        ui.horizontal_wrapped(|ui| {
            if widgets::secondary(ui, None, "Cancel").clicked() {
                state.key.clear();
                state.key_provider = None;
            }
            if ui.add_enabled(state.task.is_none() && !state.key.trim().is_empty(), egui::Button::new(tr("Save API key"))).clicked() {
                let key = state.key.clone();
                state.key_pending = true;
                start(app, &mut state, Action::SetKey { provider_id: provider_id.clone(), key });
            }
        });
    }
    // The snapshot is token-free. Taking it out lets actions update transient state while drawing.
    if let Some(snapshot) = state.snapshot.take() {
        for provider in &snapshot.providers {
            if state.adding || state.model_provider.is_some() || state.key_provider.is_some() || provider.accounts.is_empty() {
                continue;
            }
            widgets::panel(&t).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    widgets::provider_mark(ui, &provider.name);
                    ui.label(RichText::new(&provider.name).font(theme::semibold(theme::SMALL)));
                    let add = widgets::secondary(ui, Some("plus"), "Add account");
                    if widgets::named(add, &crate::i18n::trf("Add account to {name}", &[("name", &provider.name)])).clicked() {
                        state.adding = true;
                        state.search = provider.name.clone();
                        state.key_provider = None;
                        state.key.clear();
                    }
                });
                if let Some(error) = &provider.usage_error {
                    ui.label(RichText::new(error).color(t.warning));
                }
                for account in &provider.accounts {
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(&account.label).strong());
                        if account.selected {
                            widgets::pill(ui, "Selected", t.on_accent, t.accent);
                        }
                        if account.in_use {
                            widgets::pill(ui, "Last used", t.text_dim, t.raised);
                        }
                        ui.label(RichText::new(&account.state).color(t.text_faint).size(theme::CAPTION));
                        ui.push_id((&provider.id, account.id), |ui| {
                            ui.menu_button(tr("Account actions"), |ui| {
                                ui.add_enabled_ui(state.task.is_none(), |ui| {
                                    if ui.button(tr("Select account")).clicked() {
                                        start(app, &mut state, Action::SelectAccount { provider_id: provider.id.clone(), credential_id: Some(account.id) });
                                        ui.close();
                                    }
                                    if ui.button(tr("Automatic")).clicked() {
                                        start(app, &mut state, Action::SelectAccount { provider_id: provider.id.clone(), credential_id: None });
                                        ui.close();
                                    }
                                    if ui.button(tr("Sign out")).clicked() {
                                        state.confirm_remove = Some((provider.id.clone(), account.id));
                                        ui.close();
                                    }
                                });
                            });
                        });
                    });
                    if let Some(detail) = &account.detail {
                        ui.label(RichText::new(detail).color(t.text_dim).size(theme::CAPTION));
                    }
                    ui.horizontal_wrapped(|ui| {
                        for limit in &account.limits {
                            let mut text = limit.label.clone();
                            if let Some(fraction) = limit.used_fraction {
                                text.push_str(&format!(" · {:.0}%", fraction * 100.0));
                            }
                            if let (Some(used), Some(total)) = (limit.used, limit.limit) {
                                text.push_str(&format!(" · {used}/{total} {}", limit.unit.as_deref().unwrap_or("")));
                            }
                            ui.vertical(|ui| {
                                widgets::quota_badge(ui, &text, limit.used_fraction);
                                if let Some(reset) = limit.resets_at {
                                    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
                                    let minutes = reset.saturating_sub(now).div_ceil(60);
                                    let remaining = if minutes >= 1440 {
                                        format!("{}d {}h", minutes / 1440, minutes % 1440 / 60)
                                    } else if minutes >= 60 {
                                        format!("{}h {}m", minutes / 60, minutes % 60)
                                    } else {
                                        format!("{minutes}m")
                                    };
                                    ui.label(RichText::new(format!("{} {remaining}", tr("Resets in"))).size(theme::CAPTION).color(t.text_faint));
                                }
                            });
                        }
                        if let Some(credits) = &account.credits {
                            widgets::pill(ui, credits, t.text_dim, t.raised);
                        }
                    });
                    if let Some(error) = &account.usage_error {
                        ui.label(RichText::new(error).color(t.warning).size(theme::CAPTION));
                    }
                    if account.limits.is_empty() && account.credits.is_none() && account.usage_error.is_none() {
                        ui.label(RichText::new(tr("Usage unavailable")).color(t.text_faint).size(theme::CAPTION));
                    }
                    if state.confirm_remove.as_ref() == Some(&(provider.id.clone(), account.id)) {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(tr("Sign out this account?"));
                            if state.task.is_none() && widgets::secondary(ui, None, "Confirm sign out").clicked() {
                                start(app, &mut state, Action::RemoveAccount { provider_id: provider.id.clone(), credential_id: account.id });
                                state.confirm_remove = None;
                            }
                            if widgets::secondary(ui, None, "Keep account").clicked() {
                                state.confirm_remove = None;
                            }
                        });
                    }
                }
            });
            ui.add_space(space::S);
        }
        if state.adding && state.key_provider.is_none() && state.model_provider.is_none() && state.task.is_none() {
            ui.label(RichText::new(tr("Add account")).strong());
            widgets::named(
                ui.add(
                    egui::TextEdit::singleline(&mut state.search)
                        .hint_text(tr("Search providers"))
                        .desired_width(ui.available_width())
                        .frame(widgets::text_field_frame(&t)),
                ),
                &tr("Search providers"),
            );
            let query = state.search.trim().to_lowercase();
            let recommended = |id: &str| matches!(id, "anthropic" | "openai" | "openai-codex" | "google" | "openrouter");
            let mut providers: Vec<_> = snapshot
                .providers
                .iter()
                .filter(|provider| provider.name.to_lowercase().contains(&query) || provider.id.to_lowercase().contains(&query))
                .collect();
            providers.sort_by_key(|provider| !recommended(&provider.id));
            if providers.is_empty() {
                ui.label(tr("No matching providers."));
            } else {
                let split = providers.partition_point(|provider| recommended(&provider.id));
                let (featured, other) = providers.split_at(split);
                let columns = if ui.available_width() >= 520.0 { 2 } else { 1 };
                for (title, group, featured) in [("Recommended", featured, true), ("Other providers", other, false)] {
                    if group.is_empty() {
                        continue;
                    }
                    if query.is_empty() {
                        widgets::caption(ui, title);
                    }
                    for row in group.chunks(columns) {
                        ui.columns(row.len(), |columns| {
                            for (ui, provider) in columns.iter_mut().zip(row.iter()) {
                                let mut draw = |ui: &mut Ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        widgets::provider_mark(ui, &provider.name);
                                        ui.label(RichText::new(&provider.name).font(theme::semibold(theme::SMALL)));
                                        if let Some(reason) = &provider.disabled_reason {
                                            ui.label(RichText::new(reason).color(t.warning));
                                        } else {
                                            if state.task.is_none() && provider.login_available && widgets::secondary(ui, None, "Login").clicked() {
                                                start(app, &mut state, Action::Login { provider_id: provider.id.clone() });
                                            }
                                            if state.task.is_none()
                                                && provider.auth_kind.as_deref() == Some("api_key")
                                                && widgets::secondary(ui, None, "API key").clicked()
                                            {
                                                state.key_provider = Some(provider.id.clone());
                                            }
                                        }
                                    });
                                    if let Some(warning) = &provider.risk_warning {
                                        ui.label(RichText::new(warning).color(t.warning));
                                    }
                                };
                                if featured {
                                    widgets::card(&t).show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        draw(ui);
                                    });
                                } else {
                                    draw(ui);
                                }
                            }
                        });
                        ui.add_space(space::XS);
                    }
                }
            }
        }
        state.snapshot = Some(snapshot);
    } else if state.task.is_some() {
        ui.label(tr("Loading accounts…"));
    }
    app.provider_accounts = state;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_progress_keeps_browser_link_and_input_visible() {
        let mut state = AccountsState::default();
        state.login_event(LoginEvent::OpenUrl { url: "https://example.invalid/login".into(), instructions: None });
        state.login_event(LoginEvent::Input { request_id: "code".into(), title: "Paste the code".into(), placeholder: None, secret: false });
        state.login_event(LoginEvent::Progress { message: "Waiting for sign-in".into() });
        let ctx = egui::Context::default();
        theme::install_fonts(&ctx);
        let mut output = ctx.run_ui(Default::default(), |ui| login_prompt(&mut state, ui));
        output.textures_delta.clear();
        let text = output
            .shapes
            .iter()
            .filter_map(|shape| match &shape.shape {
                egui::epaint::Shape::Text(text) => Some(text.galley.text().to_owned()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("Open sign-in page"), "{text}");
        assert!(text.contains("Paste the code"), "{text}");
        assert!(text.contains("Waiting for sign-in"), "{text}");
    }
}
