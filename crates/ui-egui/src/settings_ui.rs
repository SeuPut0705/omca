//! Settings: General (mode, theme, language), Engines (Ghidra, tool paths), AI, Builds (when a
//! build is good enough, generated art), Plugins (MCP servers), Safety.

use crate::i18n::{LANGUAGES, tr, trf};
use crate::state::SettingsTab;
use crate::theme::{self, ThemeKind, Tokens, size, space};
use crate::{CraftApp, Pick, icons, widgets};
use craft_anything_engine::Mode;
use egui::{RichText, Ui, Vec2};
use std::path::PathBuf;

/// Draws the Settings modal and the content for its selected tab.
pub fn show(app: &mut CraftApp, ctx: &egui::Context) {
    if !app.state.settings_open {
        return;
    }
    let t = Tokens::get(ctx);
    let mut open = true;
    // A modal (dimmed backdrop, Esc or a click outside closes) with a full-size close button;
    // egui's window close box is 14 px.
    let resp = egui::Modal::new(egui::Id::new("settings-window")).frame(widgets::dialog(&t)).show(ctx, |ui| {
        let width = 1040.0_f32.min((ctx.content_rect().width() - space::XXL * 2.0).max(280.0));
        ui.set_width(width);
        ui.horizontal(|ui| {
            ui.label(theme::heading(tr("Settings"), theme::SECTION));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icons::button(ui, "x", size::CONTROL_LG, false, "Close").clicked() {
                    open = false;
                }
            });
        });
        // An error (a save that failed) shows here too: the message bar is behind this window.
        if let Some(i) = app.state.toasts.iter().rposition(|(_, error)| *error) {
            let text = app.state.toasts[i].0.clone();
            ui.add_space(space::SM);
            ui.horizontal(|ui| {
                icons::show(ui, "triangle-alert", size::ICON, t.danger);
                ui.add(egui::Label::new(RichText::new(text).color(t.danger)).wrap());
                if icons::button(ui, "x", size::CONTROL_SM, false, "Dismiss").clicked() {
                    app.state.toasts.remove(i);
                }
            });
        }
        ui.add_space(space::L);
        const TABS: [(SettingsTab, &str, &str); 7] = [
            (SettingsTab::General, "settings", "General"),
            (SettingsTab::Engines, "cpu", "Engines"),
            (SettingsTab::Ai, "bot", "Models & accounts"),
            (SettingsTab::Builds, "hammer", "Builds"),
            (SettingsTab::Plugins, "puzzle", "Plugins"),
            (SettingsTab::Safety, "shield", "Safety"),
            (SettingsTab::Updates, "refresh-cw", "Updates"),
        ];
        let compact = width < 760.0;
        if compact {
            let title = TABS.iter().find(|(tab, _, _)| *tab == app.state.settings_tab).map(|(_, _, title)| *title).unwrap_or("General");
            let response = egui::ComboBox::from_id_salt("settings-section")
                .selected_text(tr(title))
                .width(ui.available_width())
                .icon(widgets::combo_icon)
                .show_ui(ui, |ui| {
                    for (tab, _, title) in TABS {
                        ui.selectable_value(&mut app.state.settings_tab, tab, tr(title));
                    }
                });
            widgets::named(response.response, &tr("Settings section"));
            ui.add_space(space::M);
            settings_content(app, ui, ctx);
        } else {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(184.0);
                    for (tab, icon, title) in TABS {
                        let on = app.state.settings_tab == tab;
                        let (rect, response) = ui.allocate_exact_size(Vec2::new(184.0, size::ROW_LG), egui::Sense::click());
                        response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, tr(title)));
                        if on || response.hovered() || response.has_focus() {
                            ui.painter().rect_filled(rect, t.radius_sm, if on { t.accent_soft } else { t.hover });
                        }
                        icons::paint(
                            ui,
                            egui::Rect::from_center_size(egui::pos2(rect.left() + space::L + size::ICON / 2.0, rect.center().y), Vec2::splat(size::ICON)),
                            icon,
                            size::ICON,
                            if on { t.accent_ink } else { t.icon },
                        );
                        ui.painter().text(
                            egui::pos2(rect.left() + space::L + size::ICON + space::M, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            tr(title),
                            theme::medium(theme::SMALL),
                            if on { t.text } else { t.text_dim },
                        );
                        if response.clicked() {
                            app.state.settings_tab = tab;
                        }
                    }
                });
                ui.add_space(space::M);
                ui.separator();
                ui.add_space(space::M);
                ui.vertical(|ui| settings_content(app, ui, ctx));
            });
        }
    });
    if !open || resp.should_close() {
        app.state.settings_open = false;
    }
}

fn settings_content(app: &mut CraftApp, ui: &mut Ui, ctx: &egui::Context) {
    let width = ui.available_width();
    let height = (ctx.content_rect().height() - 200.0).clamp(180.0, 640.0);
    egui::ScrollArea::vertical()
        .id_salt(("settings-content", std::mem::discriminant(&app.state.settings_tab)))
        .auto_shrink([false, false])
        .min_scrolled_height(height)
        .max_height(height)
        .max_width(width)
        .show(ui, |ui| {
            ui.set_width((width - ui.spacing().scroll.allocated_width() - space::L).max(120.0));
            match app.state.settings_tab {
                SettingsTab::General => general(app, ui),
                SettingsTab::Engines => engines(app, ui),
                SettingsTab::Ai => ai(app, ui),
                SettingsTab::Builds => builds(app, ui),
                SettingsTab::Plugins => plugins(app, ui),
                SettingsTab::Safety => safety(app, ui),
                SettingsTab::Updates => updates(app, ui),
            }
        });
}

/// One setting: its name and what it does on the left, taking all the room the control leaves,
/// and the control on the right edge; a hairline between settings (Attio's and Replit's settings).
/// The control's width is measured each frame and kept for the next, so the words wrap only when
/// they must; a change lays the frame out again.
fn row(ui: &mut Ui, title: &str, sub: &str, add: impl FnOnce(&mut Ui)) {
    let t = Tokens::get(ui.ctx());
    let id = ui.id().with(("setting-control-w", title));
    let control_w = ui.data(|d| d.get_temp::<f32>(id)).unwrap_or(size::TILE_LG * 5.0);
    let full = ui.available_width();
    // egui rounds widget rects to whole points: keep that much in hand so the control never
    // ends past the row.
    let text_w = ((full - control_w - space::L).floor() - space::XXS).max(full * 0.4);
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(Vec2::new(text_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
            ui.set_width(text_w);
            ui.label(RichText::new(tr(title)).font(theme::medium(theme::SMALL)));
            if !sub.is_empty() {
                ui.add(egui::Label::new(RichText::new(tr(sub)).color(t.text_faint).size(theme::CAPTION)).wrap());
            }
        });
        ui.add_space(space::L - ui.spacing().item_spacing.x);
        let measured = ui.scope(add).response.rect.width();
        if (measured - control_w).abs() > 0.5 {
            ui.data_mut(|d| d.insert_temp(id, measured));
            ui.ctx().request_discard("a setting's control changed width");
        }
    });
    ui.add_space(space::SM);
    let y = ui.cursor().top();
    ui.painter().hline(ui.max_rect().x_range(), y, theme::hairline(t.separator));
    ui.add_space(space::SM);
}

fn general(app: &mut CraftApp, ui: &mut Ui) {
    row(ui, "Mode", "Simple: say what you want and it's handled for you. Advanced: every tool, the evidence and the logs.", |ui| {
        let sel = usize::from(app.settings.mode == Mode::Experienced);
        if let Some(i) = widgets::segmented(ui, &["Simple", "Advanced"], sel, 30.0) {
            app.set_mode(if i == 0 { Mode::Beginner } else { Mode::Experienced });
        }
    });
    let dir = app.settings.projects_dir();
    row(ui, "New projects go in", &dir.display().to_string(), |ui| {
        if widgets::secondary(ui, Some("folder-open"), "Change…").clicked() {
            app.pick(crate::Pick::ProjectsDir);
        }
    });
    row(ui, "Also list projects from", "Folders of projects, and projects started in a folder you picked. Removing one only takes it off the list.", |ui| {
        if widgets::secondary(ui, Some("plus"), "Add a folder…").clicked() {
            app.pick(crate::Pick::ExtraProjectsDir);
        }
    });
    let mut remove = None;
    for f in &app.settings.project_folders {
        ui.horizontal(|ui| {
            ui.add_space(space::M);
            crate::icons::show(ui, "folder", size::ICON_SM, Tokens::get(ui.ctx()).text_faint);
            ui.add(egui::Label::new(RichText::new(f.display().to_string()).font(theme::mono(theme::CAPTION))).truncate());
            if crate::icons::button(ui, "x", size::CONTROL_SM, false, &trf("Stop listing {folder}", &[("folder", &f.display().to_string())])).clicked() {
                remove = Some(f.clone());
            }
        });
    }
    if let Some(f) = remove {
        app.remove_project_folder(&f);
    }
    row(ui, "When a folder you pick has files", "Whether they're the app to edit in place, or material the app starts from beside them.", |ui| {
        let current = crate::folders::EXISTING_CHOICES.iter().find(|(_, v)| *v == app.settings.existing_files).map_or("Ask each time", |(l, _)| *l);
        egui::ComboBox::from_id_salt("existing-files").icon(widgets::combo_icon).selected_text(tr(current)).show_ui(ui, |ui| {
            for (label, value) in crate::folders::EXISTING_CHOICES {
                if ui.selectable_label(app.settings.existing_files == value, tr(label)).clicked() {
                    app.settings.existing_files = value;
                    app.settings_changed();
                }
            }
        });
    });
    row(ui, "Theme", "", |ui| {
        let sel = usize::from(app.theme_kind() == ThemeKind::StudioLight);
        if let Some(i) = widgets::segmented(ui, &["Studio", "Studio Light"], sel, 30.0) {
            app.settings.theme = ThemeKind::ALL.get(i).copied().unwrap_or_default().id().into();
            app.settings_changed();
        }
    });
    // A choice of one isn't a choice: the row appears once there's a second language.
    if LANGUAGES.len() > 1 {
        language(app, ui);
    }
    row(ui, "Show the welcome again", "Runs the first-run steps next time.", |ui| {
        if widgets::secondary(ui, Some("refresh-cw"), "Reset").clicked() {
            app.settings.first_run_done = false;
            app.state.first_run = crate::state::FirstRunStep::Welcome;
            app.state.settings_open = false;
            app.settings_changed();
        }
    });
}

fn updates(app: &mut CraftApp, ui: &mut Ui) {
    row(ui, "Current version", "", |ui| {
        ui.label(format!("OMCA {}", env!("CARGO_PKG_VERSION")));
    });
    #[cfg(not(target_arch = "wasm32"))]
    row(ui, "Check for Updates", "", |ui| {
        if widgets::secondary(ui, Some("refresh-cw"), "Check for Updates").clicked() {
            app.updates.check(ui.ctx());
            app.state.settings_open = false;
        }
    });
    ui.hyperlink_to(tr("Releases"), "https://github.com/seuput-inc/omca/releases");
    ui.add_space(space::SM);
    ui.add(egui::Label::new(tr("Use Releases or your package manager to update this installation.")).wrap());
}

fn language(app: &mut CraftApp, ui: &mut Ui) {
    row(ui, "Language", "", |ui| {
        let selected = (!app.settings.language.trim().is_empty()).then(|| crate::i18n::language_from_tag(&app.settings.language).unwrap_or("en"));
        let current = LANGUAGES.iter().find(|l| Some(l.code) == selected).map(|l| l.name).unwrap_or("System");
        egui::ComboBox::from_id_salt("lang").icon(widgets::combo_icon).selected_text(tr(current)).show_ui(ui, |ui| {
            if ui.selectable_label(app.settings.language.trim().is_empty(), tr("System")).clicked() {
                app.settings.language.clear();
                crate::i18n::set_language("");
                app.settings_changed();
            }
            for l in LANGUAGES {
                if ui.selectable_label(selected == Some(l.code), l.name).clicked() {
                    app.settings.language = l.code.into();
                    crate::i18n::set_language(l.code);
                    app.settings_changed();
                }
            }
        });
    });
}

/// A path setting; `name` is what a screen reader calls it.
fn path_field(ui: &mut Ui, value: &mut Option<PathBuf>, name: &str, hint: &str) -> bool {
    // While typing, the text as typed: the path kept is trimmed, and showing that instead would
    // drop the space typed in "C:\Program Files" before the next letter.
    let typed = ui.next_auto_id().with("typed");
    let shown = value.as_ref().map(|p| p.display().to_string()).unwrap_or_default();
    let mut s = ui.data(|d| d.get_temp::<String>(typed)).unwrap_or(shown);
    let r = ui.add(
        egui::TextEdit::singleline(&mut s)
            .hint_text(tr(hint))
            // Up to 240, leaving room for a Choose… button after it, so a row never widens the window (#68).
            .desired_width((ui.available_width() - 34.0).clamp(120.0, 240.0))
            .font(theme::mono(theme::CAPTION))
            .frame(widgets::text_field_frame_compact(&Tokens::get(ui.ctx()))),
    );
    widgets::named(r.clone(), &tr(name));
    if r.has_focus() {
        ui.data_mut(|d| d.insert_temp(typed, s.clone()));
    } else {
        ui.data_mut(|d| d.remove::<String>(typed));
    }
    if r.changed() {
        *value = if s.trim().is_empty() { None } else { Some(PathBuf::from(s.trim())) };
        return true;
    }
    false
}

fn engines(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    row(
        ui,
        "Clean room / Deep",
        "Clean room never reads the original's code, files or page source: it works from what you can see. Deep reads them with every tool, and with Ghidra when it's installed. Every finding records which was used.",
        |ui| {
            let sel = usize::from(app.settings.deep_mode);
            if let Some(i) = widgets::segmented(ui, &["Clean room", "Deep"], sel, 30.0) {
                app.set_deep(i == 1);
            }
        },
    );
    ui.add_space(space::M);
    row(ui, "Ghidra", "Adds its decompiler to Deep mode. OMCA can download and set it up for you.", |ui| {
        crate::engine_ui::ghidra_status(app, ui);
    });
    row(
        ui,
        "Memory for Ghidra",
        "More memory makes Ghidra's first look at a big program faster; leave some for the rest of the computer. Used from the next time Ghidra starts.",
        |ui| crate::engine_ui::ghidra_memory_setting(app, ui),
    );
    ui.label(RichText::new(crate::engine_ui::ghidra_memory_guide()).color(t.text_faint).size(theme::CAPTION));
    ui.add_space(space::SM);
    row(ui, "Ghidra folder", "Leave empty to use the one OMCA sets up.", |ui| {
        if path_field(ui, &mut app.settings.engines.ghidra.install_dir, "Ghidra folder", "managed") {
            app.settings_changed();
        }
        if icons::button(ui, "folder-open", size::CONTROL, false, "Choose…").clicked() {
            app.pick(Pick::GhidraDir);
        }
    });
    row(ui, "Java runtime", "Only for Ghidra. Empty = managed.", |ui| {
        if path_field(ui, &mut app.settings.engines.ghidra.java_home, "Java runtime", "managed") {
            app.settings_changed();
        }
    });
    ui.add_space(space::M);
    crate::engine_ui::settings_section(app, ui);
    ui.add_space(space::M);
    widgets::caption(ui, "Your own tool installs (optional)");
    ui.label(RichText::new(tr("Leave a path empty to use the managed install. We never need environment variables.")).color(t.text_faint).size(theme::CAPTION));
    ui.add_space(space::S);
    let mut changed = false;
    egui::Grid::new("tool-paths").num_columns(2).spacing([space::M, space::S]).show(ui, |ui| {
        // Names and purposes come from the engine table; the override list matches it (tested).
        for (id, v) in app.settings.engines.tools.entries_mut() {
            let (name, needed_for) = craft_anything_engines::engine(id).map(|e| (e.name, e.needed_for)).unwrap_or((id, ""));
            ui.label(RichText::new(tr(name)).size(theme::CAPTION)).on_hover_text(format!("{}\n{id}", tr(needed_for)));
            changed |= path_field(ui, v, name, "managed");
            ui.end_row();
        }
    });
    if changed {
        app.settings_changed();
    }
}

fn ai(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.label(theme::heading(tr("Models & accounts"), theme::SECTION));
    ui.add_space(space::XS);
    ui.add(
        egui::Label::new(
            RichText::new(tr("Only accounts connected in OMCA are used. Other apps and installed agents are never imported."))
                .color(t.text_dim)
                .size(theme::SMALL),
        )
        .wrap(),
    );
    ui.add_space(space::XL);
    crate::provider_connect::show(app, ui);
}

/// A number field; true when it changed.
fn number<T: egui::emath::Numeric>(ui: &mut Ui, value: &mut T, range: std::ops::RangeInclusive<T>, prefix: &str, suffix: &str) -> bool {
    ui.add(egui::DragValue::new(value).range(range).prefix(prefix).suffix(suffix)).changed()
}

fn builds(app: &mut CraftApp, ui: &mut Ui) {
    let mut changed = false;
    let b = &mut app.settings.build;
    row(ui, "Good enough", "How alike a rebuild must look to the original before refining stops.", |ui| {
        changed |= number(ui, &mut b.good_enough, 50.0..=100.0, "", " %");
    });
    row(ui, "Refining rounds", "At most this many rounds of fixes after the first build.", |ui| {
        changed |= number(ui, &mut b.max_rounds, 0..=20, "", "");
    });
    row(ui, "Website pages", "Other pages of a website studied and compared besides the one you gave.", |ui| {
        changed |= number(ui, &mut b.max_pages, 0..=craft_anything_ai::rebuild::kinds::website::MAX_ROUTES, "", "");
    });
    row(ui, "Match my pictures", "Apps and games: keep refining until they look this alike to your pictures. Off, the likeness is only reported.", |ui| {
        let mut on = b.picture_good_enough.is_some();
        if widgets::toggle(ui, &mut on, "Use as a goal").changed() {
            b.picture_good_enough = on.then_some(80.0);
            changed = true;
        }
        if let Some(v) = &mut b.picture_good_enough {
            changed |= number(ui, v, 30.0..=100.0, "", " %");
        }
    });
    row(ui, "Builds at once", "How many projects may build at the same time. More wait their turn.", |ui| {
        changed |= number(ui, &mut b.parallel, 1..=8, "", "");
    });
    ui.add_space(space::SM);
    if let Some(bytes) = app.pinned_copy_bytes {
        row(ui, "Pinned builds", "Size", |ui| {
            ui.label(craft_anything_engine::evidence::human_bytes(bytes));
        });
    }
    widgets::caption(ui, "Generated art (games)");
    let a = &mut app.settings.assets;
    row(
        ui,
        "Budget for a new project",
        "What a new game may spend on generated pictures and 3D models before you're asked. Each project keeps its own.",
        |ui| {
            changed |= number(ui, &mut a.budget_usd, 0.0..=10_000.0, "$", "");
        },
    );
    let table = craft_anything_engines::assets::Catalogue::built_in();
    row(ui, "Picture model", "What pictures and textures are made with.", |ui| {
        let current = table.image_model(a.image_model.as_deref()).or_else(|_| table.image_model(None)).map(|m| m.name.clone()).unwrap_or_default();
        egui::ComboBox::from_id_salt("image-model").icon(widgets::combo_icon).selected_text(current).show_ui(ui, |ui| {
            for m in table.images() {
                let price = crate::usage::asset_cost_words(m.cost(craft_anything_engines::assets::Shape::Square));
                let label = format!("{} · {} {}", m.name, price, tr("a picture"));
                let on = a.image_model.as_deref() == Some(m.id.as_str());
                if ui.selectable_label(on, label).on_hover_text(&m.licence).clicked() && !on {
                    a.image_model = Some(m.id.clone());
                    changed = true;
                }
            }
        });
    });
    if changed {
        app.settings_changed();
    }
    ui.add_space(space::XL);
    crate::engine_ui::assets_settings(app, ui);
}

/// Plugins: external MCP servers (`[[plugins.mcp]]`) and WASM components (`[[plugins.wasm]]`),
/// listed, turned on or off, added and removed.
fn plugins(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.add(
        egui::Label::new(
            RichText::new(tr(
                "Plugins add tools OMCA offers as its own: MCP servers (a command or an address; read their tools after adding) or WebAssembly components (a .wasm file, which can't touch your files or the network).",
            ))
            .color(t.text_faint)
            .size(theme::CAPTION),
        )
        .wrap(),
    );
    ui.add_space(space::M);
    let plugins = app.settings.plugins.mcp.clone();
    let wasm = app.settings.plugins.wasm.clone();
    if plugins.is_empty() && wasm.is_empty() {
        ui.label(RichText::new(tr("No plugins yet.")).color(t.text_faint));
    }
    let mut changed = false;
    for (i, p) in plugins.iter().enumerate() {
        widgets::panel(&t).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(300.0);
                    ui.label(RichText::new(p.display_name()).font(theme::medium(theme::SMALL)));
                    ui.label(RichText::new(p.place()).font(theme::mono(theme::CAPTION)).color(t.text_faint));
                    if let Some(problem) = p.problem() {
                        ui.label(RichText::new(problem).color(t.warning).size(theme::CAPTION));
                    }
                    if !p.consent {
                        ui.label(RichText::new(tr("Runs its tools without asking")).color(t.warning).size(theme::CAPTION));
                    }
                });
                if widgets::toggle(ui, &mut app.settings.plugins.mcp[i].enabled, "On").changed() {
                    changed = true;
                }
                if icons::button(ui, "refresh-cw", size::CONTROL, false, "Read its tools").clicked() {
                    app.act("plugins_refresh", serde_json::json!({"id": p.id}), tr("Read the plugin's tools"));
                }
                if icons::button(ui, "trash-2", size::CONTROL, false, "Remove").clicked() {
                    app.act("plugins_remove", serde_json::json!({"id": p.id}), tr("Remove a plugin"));
                }
            });
        });
        ui.add_space(space::S);
    }
    for (i, p) in wasm.iter().enumerate() {
        widgets::panel(&t).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(300.0);
                    ui.label(RichText::new(p.display_name()).font(theme::medium(theme::SMALL)));
                    ui.label(RichText::new(p.path.display().to_string()).font(theme::mono(theme::CAPTION)).color(t.text_faint));
                    if let Some(problem) = p.problem() {
                        ui.label(RichText::new(problem).color(t.warning).size(theme::CAPTION));
                    }
                });
                if widgets::toggle(ui, &mut app.settings.plugins.wasm[i].enabled, "On").changed() {
                    changed = true;
                }
                if icons::button(ui, "trash-2", size::CONTROL, false, "Remove").clicked() {
                    app.act("plugins_remove", serde_json::json!({"id": p.id}), tr("Remove a plugin"));
                }
            });
        });
        ui.add_space(space::S);
    }
    if changed {
        app.settings_changed();
    }
    if !plugins.is_empty() || !wasm.is_empty() {
        ui.label(RichText::new(tr("Turning a plugin on or off takes effect from the next start.")).color(t.text_faint).size(theme::CAPTION));
    }
    ui.add_space(space::L);
    widgets::caption(ui, "Add a plugin");
    ui.add_space(space::XS);
    let form = &mut app.state.plugin_form;
    ui.horizontal(|ui| {
        let id =
            ui.add(egui::TextEdit::singleline(&mut form.id).hint_text(tr("id, e.g. docs")).desired_width(120.0).frame(widgets::text_field_frame_compact(&t)));
        widgets::named(id, "Plugin id");
        let place = ui.add(
            egui::TextEdit::singleline(&mut form.place)
                .hint_text(tr("command, https:// address or .wasm file"))
                .desired_width(200.0)
                .frame(widgets::text_field_frame_compact(&t)),
        );
        widgets::named(place, "Plugin command or address");
        let args = ui.add(
            egui::TextEdit::singleline(&mut form.args).hint_text(tr("arguments (optional)")).desired_width(160.0).frame(widgets::text_field_frame_compact(&t)),
        );
        widgets::named(args, "Plugin arguments");
    });
    ui.add_space(space::S);
    let params = app.state.plugin_form.params();
    if widgets::primary_icon(ui, Some("plus"), "Add", params.is_some()).clicked()
        && let Some(params) = params
    {
        app.act("plugins_add", params, tr("Add a plugin"));
        app.state.plugin_form = Default::default();
    }
    ui.label(RichText::new(tr("Plugins added here always ask before each call.")).color(t.text_faint).size(theme::CAPTION));
}

fn safety(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    row(
        ui,
        "Ask before running anything",
        "Tools that run code from a target or change a phone always ask first, unless you allowed them for a project.",
        |ui| {
            if widgets::toggle(ui, &mut app.settings.ask_before_running, "Ask first").changed() {
                app.settings_changed();
            }
        },
    );
    if !app.settings.ask_before_running {
        ui.horizontal(|ui| {
            icons::show(ui, "triangle-alert", size::ICON, t.warning);
            ui.label(RichText::new(tr("Tools may now run code from what you open without asking. Turn this back on unless you're sure.")).color(t.warning));
        });
    }
    row(
        ui,
        "Auto mode",
        "Builds keep going without stopping to ask you before an action. The builder still never deletes outside its project, changes system settings or publishes anything, and stays within each project's asset budget.",
        |ui| {
            if widgets::toggle(ui, &mut app.settings.auto_mode, "Keep going without asking").changed() {
                app.settings_changed();
            }
        },
    );
    row(
        ui,
        "The builder's questions",
        "When the builder needs a choice made: show it to you and wait, or let it decide and say what it chose. Each project can choose for itself under its prompt box.",
        |ui| {
            use craft_anything_engine::workspace::Questions;
            let sel = usize::from(app.settings.questions() == Questions::Decide);
            if let Some(i) = widgets::segmented(ui, &["Ask me", "Decide yourself"], sel, 30.0) {
                app.settings.questions = Some(if i == 0 { Questions::Ask } else { Questions::Decide });
                app.settings_changed();
            }
        },
    );
    row(
        ui,
        "Study apps by running them",
        "When you rebuild an app or game, start it once to picture its window and compare the rebuild with it. It runs that program.",
        |ui| {
            if widgets::toggle(ui, &mut app.settings.runtime_study, "Run to study").changed() {
                app.settings_changed();
            }
        },
    );
    if let Some(p) = app.view.project.clone() {
        ui.add_space(space::SM);
        widgets::caption(ui, &format!("{} {}", tr("Allowed in"), p));
        if app.view.project_allows.is_empty() {
            ui.label(RichText::new(tr("Nothing allowed yet.")).color(t.text_faint));
        }
        for tool in app.view.project_allows.clone() {
            ui.label(RichText::new(tool).font(theme::mono(theme::CAPTION)));
        }
    }
}

/// What to tell the person at startup when their settings couldn't be loaded: `why` names the
/// file, and the rest says what that means until it's fixed. `None` when they loaded. English for
/// now: the wording needs a native speaker's check before it joins the catalogs.
pub(crate) fn not_loaded_warning(settings: &craft_anything_engine::Settings) -> Option<String> {
    let why = settings.not_loaded.as_deref()?;
    Some(format!("{why}. Until that's fixed, OMCA uses its default settings and won't save over it."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Done, demo, jobs::Finished};

    /// Settings that couldn't be loaded say so where the person is looking, with the file's name.
    #[test]
    fn settings_that_didnt_load_are_said_at_startup() {
        let loaded = craft_anything_engine::Settings::default();
        assert_eq!(not_loaded_warning(&loaded), None);
        let why = r"C:\Users\me\AppData\Roaming\Craft Anything\config.toml couldn't be read (access denied)";
        let standing_in = craft_anything_engine::Settings { not_loaded: Some(why.into()), ..Default::default() };
        let said = not_loaded_warning(&standing_in).unwrap();
        assert!(said.contains("config.toml") && said.contains("access denied"), "{said}");
        assert!(said.contains("won't save over it"), "{said}");
    }

    /// A key check that started before a save or a newer check must not put the old state back.
    #[test]
    fn a_stale_backend_key_check_does_not_replace_a_newer_one() {
        use std::collections::BTreeMap;

        let mut app = demo::app(demo::engine(craft_anything_engine::Mode::Experienced, true));
        app.backend_keys_request = 2;
        app.backend_keys = Some(BTreeMap::from([("OpenAI".to_string(), true)]));
        app.handle_done(Finished { job: 1, result: Ok(Done::BackendKeys { request: 1, keys: BTreeMap::from([("OpenAI".to_string(), false)]) }) });
        assert_eq!(app.backend_keys.as_ref().and_then(|keys| keys.get("OpenAI")), Some(&true), "the newer state stays");
        app.handle_done(Finished { job: 2, result: Ok(Done::BackendKeys { request: 2, keys: BTreeMap::from([("OpenAI".to_string(), false)]) }) });
        assert_eq!(app.backend_keys.as_ref().and_then(|keys| keys.get("OpenAI")), Some(&false), "the current check applies");
    }

    /// A failed key save keeps what was typed; a success clears it only when it is still the same
    /// text, so an edit made while the save was running is not lost.
    #[test]
    fn a_key_draft_survives_a_failed_save_and_a_newer_edit() {
        use std::collections::BTreeMap;

        let mut app = demo::app(demo::engine(craft_anything_engine::Mode::Experienced, true));
        app.backend_keys_request = 1;
        app.backend_key_inputs.insert("OpenAI".into(), "sk-draft".into());
        app.handle_done(Finished {
            job: 1,
            result: Ok(Done::BackendKeyResult {
                request: 1,
                name: "OpenAI".into(),
                keys: BTreeMap::from([("OpenAI".to_string(), false)]),
                error: Some("keychain is locked".into()),
                saved_key: None,
                forgot: false,
            }),
        });
        assert_eq!(app.backend_key_inputs.get("OpenAI").map(String::as_str), Some("sk-draft"), "a failure keeps the draft");
        assert_eq!(app.backend_keys.as_ref().and_then(|keys| keys.get("OpenAI")), Some(&false), "a failure still refreshes the shown status");

        app.backend_key_inputs.insert("OpenAI".into(), "sk-edited".into());
        app.handle_done(Finished {
            job: 2,
            result: Ok(Done::BackendKeyResult {
                request: 1,
                name: "OpenAI".into(),
                keys: BTreeMap::from([("OpenAI".to_string(), true)]),
                error: None,
                saved_key: Some("sk-saved".into()),
                forgot: false,
            }),
        });
        assert_eq!(app.backend_key_inputs.get("OpenAI").map(String::as_str), Some("sk-edited"), "a newer edit stays");

        app.backend_key_inputs.insert("OpenAI".into(), "sk-saved ".into());
        app.handle_done(Finished {
            job: 3,
            result: Ok(Done::BackendKeyResult {
                request: 1,
                name: "OpenAI".into(),
                keys: BTreeMap::from([("OpenAI".to_string(), true)]),
                error: None,
                saved_key: Some("sk-saved".into()),
                forgot: false,
            }),
        });
        assert!(!app.backend_key_inputs.contains_key("OpenAI"), "the saved draft is cleared");

        app.backend_key_inputs.insert("OpenAI".into(), "sk-next".into());
        app.handle_done(Finished {
            job: 4,
            result: Ok(Done::BackendKeyResult {
                request: 1,
                name: "OpenAI".into(),
                keys: BTreeMap::from([("OpenAI".to_string(), true)]),
                error: Some("keychain is locked".into()),
                saved_key: None,
                forgot: true,
            }),
        });
        assert_eq!(app.backend_key_inputs.get("OpenAI").map(String::as_str), Some("sk-next"), "a failed forget keeps the draft");

        app.handle_done(Finished {
            job: 5,
            result: Ok(Done::BackendKeyResult {
                request: 1,
                name: "OpenAI".into(),
                keys: BTreeMap::from([("OpenAI".to_string(), false)]),
                error: None,
                saved_key: None,
                forgot: true,
            }),
        });
        assert!(!app.backend_key_inputs.contains_key("OpenAI"), "a successful forget clears it");
    }
}
