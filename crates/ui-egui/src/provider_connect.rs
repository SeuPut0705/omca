//! Managed provider accounts and their transient login workflow.

use crate::i18n::tr;
use crate::theme::{self, Tokens, space};
use crate::{CraftApp, Done, widgets};
use craft_anything_ai::provider_runtime::{Action, Event, LoginEvent, LoginResponse, Snapshot, Task};
use egui::{RichText, Ui};

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
    prompt: Option<LoginEvent>,
    login_url: Option<(String, Option<String>)>,
    progress: Option<String>,
    input: String,
    model_provider: Option<String>,
    confirm_remove: Option<(String, u64)>,
    started: bool,
}
pub(crate) fn set_demo_snapshot(app: &mut CraftApp, snapshot: Snapshot) {
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
    if state.task.is_none() {
        state.error = None;
        state.task = Some(app.hub.provider_client().start(action));
    }
}

pub(crate) fn poll(app: &mut CraftApp) {
    let state = &mut app.provider_accounts;
    let mut finished = false;
    let mut snapshot_finished = false;
    let mut task = state.task.take();
    if let Some(task) = &mut task {
        while let Some(event) = task.try_event() {
            match event {
                Event::Login(event) => state.login_event(event),
                Event::Finished(result) => {
                    finished = true;
                    snapshot_finished = true;
                    state.prompt = None;
                    state.login_url = None;
                    state.progress = None;
                    match result {
                        Ok(snapshot) => {
                            state.snapshot = Some(snapshot);
                            if state.key_pending {
                                state.key.clear();
                                state.key_provider = None;
                            }
                            if state.model_provider.as_ref().is_some_and(|provider_id| {
                                app.backends.iter().any(|backend| {
                                    backend.kind == craft_anything_ai::BackendKind::Managed
                                        && app.backend_options.get(&backend.name).and_then(|options| options.provider_id.as_ref()) == Some(provider_id)
                                })
                            }) {
                                state.model_provider = None;
                                state.adding = false;
                            }
                        }
                        Err(error) => {
                            state.error = Some(error.for_people());
                            state.model_provider = None;
                        }
                    }
                    state.key_pending = false;
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
                }
            }
        }
    }
    state.task = task;
    if finished {
        state.task = None;
        if snapshot_finished
            && state.error.is_none()
            && let (Some(provider_id), Some(snapshot)) = (&state.model_provider, &state.snapshot)
            && !snapshot.models.iter().any(|model| &model.provider_id == provider_id)
        {
            state.task = Some(app.hub.provider_client().start(Action::Models { provider_id: provider_id.clone() }));
        }
    }
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
    let mut state = std::mem::take(&mut app.provider_accounts);
    if !state.started {
        state.started = true;
        start(app, &mut state, Action::Snapshot { refresh: false });
    }
    let t = Tokens::get(ui.ctx());
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
                state.model_provider = Some(provider_id.clone());
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
                    ui.label(RichText::new(&provider.name).font(theme::semibold(theme::SMALL)));
                    let add = widgets::secondary(ui, Some("plus"), "Add account");
                    let add = widgets::named(add, &crate::i18n::trf("Add account to {name}", &[("name", &provider.name)]));
                    if add.clicked() {
                        state.adding = true;
                        state.model_provider = None;
                        state.search = provider.name.clone();
                        state.key_provider = None;
                        state.key.clear();
                    }
                    if state.task.is_none() && widgets::secondary(ui, None, "Choose model").clicked() {
                        state.model_provider = Some(provider.id.clone());
                        start(app, &mut state, Action::Models { provider_id: provider.id.clone() });
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
                            widgets::pill(ui, "Last used", t.on_accent, t.accent);
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
                                widgets::pill(ui, &text, t.text_dim, t.raised);
                                if let Some(fraction) = limit.used_fraction {
                                    ui.add(widgets::ProgressBar::new(Some(fraction as f32)));
                                }
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
        if state.task.is_none()
            && let Some(provider_id) = state.model_provider.clone()
        {
            ui.label(RichText::new(tr("Choose model")).strong());
            let mut found = false;
            for model in snapshot.models.iter().filter(|m| m.provider_id == provider_id) {
                found = true;
                if ui.button(&model.name).clicked() {
                    let hub = app.hub.clone();
                    let provider_id = provider_id.clone();
                    let model_id = model.id.clone();
                    let choices = app.backend_choices;
                    app.jobs.spawn(tr("Connect model"), true, move |_| {
                        Ok(Done::ManagedProvider { result: hub.connect_provider(&provider_id, &model_id), choices })
                    });
                    state.model_provider = None;
                    state.adding = false;
                }
            }
            if !found && state.task.is_none() {
                ui.label(tr("No models available for this provider."));
            }
        }
        if state.adding && state.key_provider.is_none() && state.model_provider.is_none() && state.task.is_none() {
            ui.label(RichText::new(tr("Add account")).strong());
            widgets::named(
                ui.add(egui::TextEdit::singleline(&mut state.search).hint_text(tr("Search providers")).desired_width(ui.available_width())),
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
                                        ui.label(RichText::new(&provider.name).font(theme::semibold(theme::SMALL)));
                                        if let Some(reason) = &provider.disabled_reason {
                                            ui.label(RichText::new(reason).color(t.warning));
                                        } else {
                                            if state.task.is_none() && provider.login_available && widgets::secondary(ui, None, "Login").clicked() {
                                                state.model_provider = Some(provider.id.clone());
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
