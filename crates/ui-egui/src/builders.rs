//! The managed model selected for builds and the account setup prompt.

use crate::i18n::tr;
use crate::state::Modal;
use crate::{CraftApp, widgets};
use craft_anything_ai::BackendKind;
use egui::Ui;

impl CraftApp {
    pub fn default_builder(&self) -> Option<String> {
        if !crate::provider_connect::selection_ready(self) {
            return None;
        }
        self.state.backend.as_ref().filter(|name| self.backends.iter().any(|b| b.enabled && b.kind == BackendKind::Managed && &b.name == *name)).cloned()
    }

    /// Something can build.
    pub fn can_build(&self) -> bool {
        self.default_builder().is_some()
    }

    pub fn need_builder(&mut self) {
        self.state.modal = Some(Modal::NeedBuilder);
    }
}

pub fn need_builder_modal(app: &mut CraftApp, ui: &mut Ui, close: &mut bool) {
    ui.label(tr("Choose model"));
    if widgets::secondary(ui, None, "Add account").clicked() {
        *close = true;
        app.state.settings_open = true;
        app.state.settings_tab = crate::state::SettingsTab::Ai;
    }
    if widgets::secondary(ui, None, "Cancel").clicked() {
        *close = true;
    }
}
