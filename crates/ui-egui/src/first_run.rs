//! First run: Welcome → Who builds (and answers) → Purpose → Done. Four short screens, all
//! skippable. Making something needs a builder, so that's what the AI step sets up first.

use crate::i18n::tr;
use crate::state::FirstRunStep;
use crate::theme::{self, Tokens, size, space};
use crate::{CraftApp, icons, widgets};
use craft_anything_engine::Mode;
use egui::{Align2, RichText, Ui, Vec2};

pub fn show(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let full = ui.max_rect();
    let width = 640.0_f32.min(full.width() - 48.0);
    // A step taller than the window scrolls above the footer, so Continue stays reachable (#231).
    egui::ScrollArea::vertical().max_height(full.height() - 52.0).show(ui, |ui| {
        ui.add_space(((full.height() - 560.0) / 2.0).clamp(24.0, 120.0));
        ui.vertical_centered(|ui| {
            ui.set_max_width(width);
            match app.state.first_run {
                FirstRunStep::Welcome => welcome(app, ui),
                FirstRunStep::Brain => brain(app, ui, width),
                FirstRunStep::Purpose => purpose(app, ui, width),
                FirstRunStep::Done => done(app, ui),
            }
        });
        ui.add_space(space::M);
    });
    // Where you are, the ArtCraft way: "02 / 04  AI", plus a progress hairline.
    let step = app.state.first_run.index();
    let y = full.bottom() - 34.0;
    let names = ["Welcome", "Models & accounts", "Purpose", "Ready"];
    let label = format!("{:02} / 04   {}", step + 1, tr(names.get(step).copied().unwrap_or_default()));
    let g = widgets::label_galley(ui, &label, theme::SMALL, t.text_dim);
    ui.painter().galley_with_override_text_color(egui::pos2(full.center().x - g.size().x / 2.0, y - g.size().y / 2.0), g, t.text_dim);
    let track = egui::Rect::from_min_size(egui::pos2(full.left(), full.bottom() - 2.0), Vec2::new(full.width(), 2.0));
    ui.painter().rect_filled(track, 0.0, t.separator);
    ui.painter().rect_filled(egui::Rect::from_min_size(track.min, Vec2::new(track.width() * (step + 1) as f32 / 4.0, 2.0)), 0.0, t.accent);
    if step > 0 && step < 3 {
        let r = egui::Rect::from_min_size(egui::pos2(full.left() + 24.0, y - 14.0), Vec2::new(120.0, 28.0));
        let resp = ui.interact(r, ui.id().with("fr-back"), egui::Sense::click());
        let g = widgets::label_galley(ui, &tr("← Back"), theme::SMALL, t.text);
        let col = if resp.hovered() { t.text } else { t.text_dim };
        ui.painter().galley_with_override_text_color(egui::pos2(r.left(), r.center().y - g.size().y / 2.0), g, col);
        widgets::focus_ring(ui, &resp, r);
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr("Back")));
        if resp.clicked() {
            app.state.first_run = match app.state.first_run {
                FirstRunStep::Purpose => FirstRunStep::Brain,
                _ => FirstRunStep::Welcome,
            };
        }
    }
}

fn welcome(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    ui.add(crate::app_icon(132.0));
    ui.add_space(space::XL);
    ui.label(theme::heading(tr("OMCA"), theme::TITLE));
    ui.add_space(space::S);
    ui.label(RichText::new(tr("Find out how any software works, then craft your own.")).size(theme::BODY).color(t.text_dim));
    ui.add_space(space::XXXL);
    let w = 270.0;
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - (w * 2.0 + 16.0)) / 2.0).max(0.0));
        if choice_card(ui, w, "graduation-cap", "I'm new to this", "Plain words, big buttons, and I'll explain everything.").clicked() {
            app.settings.mode = Mode::Beginner;
            app.state.first_run = FirstRunStep::Brain;
        }
        ui.add_space(space::L);
        if choice_card(ui, w, "terminal", "Show me everything", "Every tool as well: a workbench with functions, bytes, references and evidence.").clicked() {
            app.settings.mode = Mode::Experienced;
            app.state.first_run = FirstRunStep::Brain;
        }
    });
    ui.add_space(space::XL);
    if widgets::link(ui, "Skip setup").on_hover_text(tr("Start with Simple mode; you can set up AI later in Settings.")).clicked() {
        app.finish_first_run();
    }
}

fn choice_card(ui: &mut Ui, w: f32, icon: &str, title: &str, blurb: &str) -> egui::Response {
    widgets::big_card(ui, Vec2::new(w, 200.0), icon, title, blurb, None)
}

fn header(ui: &mut Ui, title: &str, sub: &str) {
    let t = Tokens::get(ui.ctx());
    ui.label(theme::heading(tr(title), theme::TITLE));
    ui.add_space(space::XS);
    ui.label(RichText::new(tr(sub)).size(theme::BODY).color(t.text_dim));
    ui.add_space(space::XL);
}

fn brain(app: &mut CraftApp, ui: &mut Ui, width: f32) {
    let t = Tokens::get(ui.ctx());
    header(ui, "Who builds what you make", "Connect an account in OMCA to choose a model for building and questions.");
    // Whether something can build yet, in one line: the same rule Make and Start use.
    match app.default_builder() {
        Some(b) => {
            ui.horizontal(|ui| {
                icons::show(ui, "check", size::ICON, t.success);
                ui.label(RichText::new(crate::i18n::trf("{name} builds what you make.", &[("name", &b)])).color(t.text_dim));
            });
        }
        None => {
            ui.horizontal_wrapped(|ui| {
                icons::show(ui, "triangle-alert", size::ICON, t.warning);
                ui.label(RichText::new(tr("Choose model")).color(t.text_dim));
            });
        }
    }
    ui.add_space(space::M);
    ui.set_width(width);
    crate::provider_connect::composer(app, ui);
    // "No AI for now"
    let none_on = app.state.backend.is_none();
    let r = option_row(ui, "circle-stop", "No AI for now", "The built-in engines still show you everything they find.", none_on);
    if r.clicked() {
        app.choose_backend(None);
        app.state.first_run = FirstRunStep::Purpose;
    }

    ui.add_space(space::L);
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - 240.0) / 2.0).max(0.0));
        if widgets::primary(ui, "Continue").clicked() {
            app.state.first_run = FirstRunStep::Purpose;
        }
    });
    ui.add_space(space::M);
    crate::provider_connect::show(app, ui);
}

fn option_row(ui: &mut Ui, icon: &str, title: &str, sub: &str, on: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 64.0), egui::Sense::click());
    let h = widgets::hover_t(ui, &resp);
    let fill = if on { t.accent_soft } else { theme::mix(t.card, t.raised, h) };
    let stroke = if on { theme::hairline(t.accent) } else { theme::hairline(theme::mix(t.card_border, t.field_border, h)) };
    ui.painter().rect(rect, t.radius_lg, fill, stroke, egui::StrokeKind::Inside);
    let ir = egui::Rect::from_min_size(rect.min + Vec2::new(14.0, 12.0), Vec2::splat(size::TILE_LG));
    ui.painter().rect_filled(ir, t.radius, t.raised);
    icons::paint(ui, ir, icon, size::ICON_LG, if on { t.accent_ink } else { t.icon });
    ui.painter().text(egui::pos2(ir.right() + 14.0, rect.top() + 14.0), Align2::LEFT_TOP, tr(title), theme::semibold(theme::BODY), t.text);
    ui.painter().text(egui::pos2(ir.right() + 14.0, rect.top() + 35.0), Align2::LEFT_TOP, tr(sub), egui::FontId::proportional(theme::SMALL), t.text_dim);
    if on {
        icons::paint(
            ui,
            egui::Rect::from_center_size(egui::pos2(rect.right() - 26.0, rect.center().y), Vec2::splat(size::ICON_LG)),
            "check",
            size::ICON_LG,
            t.accent,
        );
    }
    widgets::focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, tr(title)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn purpose(app: &mut CraftApp, ui: &mut Ui, width: f32) {
    let t = Tokens::get(ui.ctx());
    header(ui, "What brings you here?", "One line, in plain words. It stays on this computer and is never sent anywhere.");
    ui.allocate_ui(Vec2::new(width, 400.0), |ui| {
        for (p, title, sub) in crate::state::purposes() {
            let on = app.state.purpose == p;
            if option_row(ui, purpose_icon(p), title, sub, on).clicked() {
                app.state.purpose = p;
            }
            ui.add_space(space::S);
        }
    });
    ui.add_space(space::XS);
    // Shown, not a hover tip in link colour: a click on it did nothing and read as broken (#71).
    ui.scope(|ui| {
        ui.set_max_width(width);
        ui.label(RichText::new(tr("Why we ask")).size(theme::SMALL).color(t.text_dim).strong());
        ui.label(RichText::new(tr(
            "Looking inside software you own or are allowed to examine is normal and often protected (learning, compatibility, security). Saying why helps you keep to it; it stays on this computer.",
        ))
        .size(theme::SMALL)
        .color(t.text_dim));
    });
    ui.add_space(space::L);
    if widgets::primary(ui, "Continue").clicked() {
        app.state.first_run = FirstRunStep::Done;
    }
}

fn purpose_icon(p: craft_anything_engine::project::Purpose) -> &'static str {
    use craft_anything_engine::project::Purpose::*;
    match p {
        OwnOrAuthorized => "user",
        Education => "graduation-cap",
        Interoperability => "link",
        Security => "shield-check",
        Preservation => "history",
    }
}

fn done(app: &mut CraftApp, ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size::TILE_LG + size::ICON_HERO), egui::Sense::hover());
    ui.painter().rect(rect, rect.width() / 2.0, t.accent_soft, theme::hairline(t.accent), egui::StrokeKind::Inside);
    icons::paint(ui, rect, "check", size::ICON_HERO, t.accent);
    ui.add_space(space::L);
    ui.label(theme::heading(tr("You're all set"), theme::TITLE));
    ui.add_space(space::S);
    let brain = app.state.backend.clone().unwrap_or_else(|| tr("the built-in engines"));
    let builds = match app.default_builder() {
        Some(b) => crate::i18n::trf("{name} builds what you make; answers come from {brain}.", &[("name", &b), ("brain", &brain)]),
        None => crate::i18n::trf("Answers come from {brain}. To build, set up a coding agent in Settings → Models & accounts.", &[("brain", &brain)]),
    };
    ui.label(RichText::new(builds).size(theme::BODY).color(t.text_dim));
    ui.add_space(space::SM);
    ui.label(
        RichText::new(tr(
            "Say what you want to make, or drop an app or file on the window to look inside it. Everything runs in Clean room mode unless you switch on Deep.",
        ))
        .size(theme::BODY)
        .color(t.text_dim),
    );
    ui.add_space(space::XXL);
    if widgets::primary_icon(ui, Some("rocket"), "Let's go", true).clicked() {
        app.finish_first_run();
    }
}
