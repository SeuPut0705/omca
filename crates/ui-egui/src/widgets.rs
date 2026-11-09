//! Small shared widgets drawn from theme tokens: cards, buttons, pills, badges, empty states.
//!
//! Icons on actions follow three rules, so a person learns an icon once:
//! - Icon and words: an action whose meaning needs words (Use this, Build with this, Always for
//!   this project, Add pictures of the original) and every main button. The icon comes from
//!   [`ACTION_ICONS`], so the same action always has the same icon.
//! - Icon only ([`icons::button`]): a well-known action repeated in a dense row (close, copy,
//!   rename, delete, open the folder, pin). Its words are its tooltip and its accessible name,
//!   so it is never a mystery and a screen reader still says it.
//! - Words only: a dialog's answer pair (Cancel, Back, Close, Not now, Dismiss), where an icon
//!   would compete with the main action beside it.

/// The icon for each action that appears on more than one screen, so it never changes meaning.
pub const ACTION_ICONS: &[(&str, &str)] = &[
    ("Open folder", "folder-open"),
    ("Rename", "pencil"),
    ("Duplicate", "copy"),
    ("Delete", "trash-2"),
    ("Remove", "trash-2"),
    ("Pin this build", "pin"),
    ("Open in the workbench", "microscope"),
    ("Open log", "file-text"),
    ("Restart", "rotate-ccw"),
    ("Use", "check"),
    ("Use this", "check"),
    ("Build with this", "hammer"),
    ("Extract with this", "package"),
    ("Always for this project", "shield-check"),
    ("Don't allow", "ban"),
];

/// The icon [`ACTION_ICONS`] gives `action` (the square when it isn't listed, so a missing row
/// shows in review rather than silently).
pub fn action_icon(action: &str) -> &'static str {
    ACTION_ICONS.iter().find(|(a, _)| *a == action).map_or("square", |(_, i)| i)
}

/// [`secondary`] with the icon [`ACTION_ICONS`] gives its words.
pub fn action(ui: &mut Ui, label: &str) -> Response {
    secondary(ui, Some(action_icon(label)), label)
}

/// An icon-only action for a dense row: the icon [`ACTION_ICONS`] gives `label`, and `label` as its
/// tooltip and accessible name.
pub fn icon_action(ui: &mut Ui, label: &str) -> Response {
    icons::button(ui, action_icon(label), size::CONTROL, false, label)
}

use crate::i18n::tr;
use crate::icons;
use crate::theme::{self, Tokens, confidence_color, confidence_label, size, space};
use craft_anything_engine::evidence::Confidence;
use egui::{Color32, Frame, Margin, Response, RichText, Sense, Stroke, Ui, Vec2};

/// Lay out `text` in the label style ([`theme::label_text`]) for painting.
pub fn label_galley(ui: &Ui, text: &str, size: f32, color: Color32) -> std::sync::Arc<egui::Galley> {
    egui::WidgetText::from(theme::label_text(text, size).color(color)).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::FontSelection::Default,
    )
}

/// 0..1 as the pointer rests on `resp` (a short ease, so hover states don't snap).
pub fn hover_t(ui: &Ui, resp: &Response) -> f32 {
    ui.ctx().animate_bool_with_time(resp.id.with("hover-t"), resp.hovered() || resp.has_focus(), 0.12)
}

/// The frame every text field uses: a field-coloured box with a 3:1 border and roomy padding.
pub fn text_field_frame(t: &Tokens) -> Frame {
    Frame::new().fill(t.field).stroke(Stroke::new(1.0, t.field_border)).corner_radius(t.r()).inner_margin(Margin::symmetric(12, 10))
}

/// The text-field frame for dense panels (filters, offsets, forms).
pub fn text_field_frame_compact(t: &Tokens) -> Frame {
    text_field_frame(t).inner_margin(Margin::symmetric(8, 6))
}

/// A switch with a label, in place of a checkbox. Flips `on` when clicked.
pub fn toggle(ui: &mut Ui, on: &mut bool, label: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    // The label wraps to the room left, so a long one can't push a narrow panel wider.
    let wrap = (ui.available_width() - 42.0).max(80.0);
    let g = (!label.is_empty()).then(|| ui.painter().layout(tr(label), egui::FontId::proportional(theme::SMALL), t.text, wrap));
    let w = 32.0 + g.as_ref().map_or(0.0, |g| g.size().x + 10.0);
    let h = g.as_ref().map_or(24.0, |g| g.size().y.max(24.0));
    let (rect, mut resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
    if resp.clicked() {
        *on = !*on;
        resp.mark_changed();
    }
    let how = ui.ctx().animate_bool_with_time(resp.id, *on, 0.12);
    let sw = egui::Rect::from_min_size(egui::pos2(rect.left(), rect.top() + 3.0), Vec2::new(32.0, 18.0));
    ui.painter().rect(sw, 9.0, theme::mix(t.field, t.accent, how), Stroke::new(1.0, theme::mix(t.field_border, t.accent, how)), egui::StrokeKind::Inside);
    let knob = egui::pos2(egui::lerp(sw.left() + 9.0..=sw.right() - 9.0, how), sw.center().y);
    ui.painter().circle_filled(knob, 6.0, theme::mix(t.text_dim, Color32::WHITE, how));
    if let Some(g) = g {
        ui.painter().galley(egui::pos2(sw.right() + 10.0, rect.center().y - g.size().y / 2.0), g, t.text);
    }
    focus_ring(ui, &resp, sw);
    let state = *on;
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, state, tr(label)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The dropdown arrow for every combo box: a Lucide chevron instead of egui's filled triangle.
pub fn combo_icon(ui: &Ui, rect: egui::Rect, _visuals: &egui::style::WidgetVisuals, _open: bool) {
    icons::paint(ui, rect.expand(2.0), "chevron-down", 14.0, Tokens::get(ui.ctx()).icon);
}

/// The disclosure arrow for every collapsing section: a Lucide chevron.
pub fn disclosure_icon(ui: &mut Ui, openness: f32, resp: &Response) {
    let t = Tokens::get(ui.ctx());
    let r = egui::Rect::from_center_size(resp.rect.center(), Vec2::splat(12.0));
    icons::paint(ui, r, if openness > 0.5 { "chevron-down" } else { "chevron-right" }, 12.0, t.text_faint);
}

/// Give a field without a visible label a name for screen readers (WCAG 4.1.2); egui doesn't
/// use the hint text for that.
pub fn named(resp: Response, name: &str) -> Response {
    resp.ctx.accesskit_node_builder(resp.id, |b| b.set_label(tr(name)));
    resp
}

/// A suggestion: a sentence in a hairline box that fills in a field when clicked. Wraps when
/// it's wider than the space it has; fills the width in a justified layout (a tidy list).
pub fn suggestion(ui: &mut Ui, text: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let max_w = ui.available_width().max(60.0);
    let g = ui.painter().layout(tr(text), egui::FontId::proportional(theme::SMALL), t.text, max_w - 26.0);
    let w = if ui.layout().horizontal_justify() { max_w } else { (g.size().x + 26.0).min(max_w) };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, (g.size().y + 14.0).max(32.0)), Sense::click());
    let h = hover_t(ui, &resp);
    ui.painter().rect(
        rect,
        t.radius,
        theme::mix(Color32::TRANSPARENT, t.hover, h),
        Stroke::new(1.0, theme::mix(t.card_border, t.field_border, h)),
        egui::StrokeKind::Inside,
    );
    let pos = egui::pos2(rect.left() + 13.0, rect.center().y - g.size().y / 2.0);
    ui.painter().galley_with_override_text_color(pos, g, theme::mix(t.text_dim, t.text, h));
    focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(text)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A big, radio-like choice with an icon and plain words (and an optional line under them,
/// empty for none); the chosen one is in the app colour with a tick (so it doesn't rely on
/// colour alone). It grows taller when the words need it.
pub fn choice_tile(ui: &mut Ui, size: Vec2, icon: &str, label: &str, blurb: &str, on: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let wrap = size.x - 84.0;
    let g = ui.painter().layout(tr(label), theme::medium(theme::BODY), t.text, wrap);
    let sub = (!blurb.is_empty()).then(|| ui.painter().layout(tr(blurb), egui::FontId::proportional(theme::SMALL), t.text_dim, wrap));
    let text_h = g.size().y + sub.as_ref().map_or(0.0, |s| s.size().y + 3.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(size.x, size.y.max(text_h + 24.0)), Sense::click());
    let h = hover_t(ui, &resp);
    let (fill, stroke) = if on {
        (t.accent_soft.gamma_multiply(0.6), theme::line(theme::EMPHASIS, t.accent))
    } else {
        (theme::mix(Color32::TRANSPARENT, t.hover, h), theme::hairline(theme::mix(t.field_border, t.text_faint, h)))
    };
    ui.painter().rect(rect, t.radius, fill, stroke, egui::StrokeKind::Inside);
    let icon_rect = egui::Rect::from_center_size(egui::pos2(rect.left() + 28.0, rect.center().y), Vec2::splat(24.0));
    icons::paint(ui, icon_rect, icon, 22.0, if on { t.accent_ink } else { t.icon });
    let top = egui::pos2(icon_rect.right() + 12.0, rect.center().y - text_h / 2.0);
    let title_h = g.size().y;
    ui.painter().galley(top, g, t.text);
    if let Some(s) = sub {
        ui.painter().galley(top + Vec2::new(0.0, title_h + 3.0), s, t.text_dim);
    }
    if on {
        icons::paint(ui, egui::Rect::from_center_size(egui::pos2(rect.right() - 20.0, rect.center().y), Vec2::splat(18.0)), "check", 18.0, t.accent_ink);
    }
    focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, on, tr(label)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A visible keyboard-focus ring around `rect` when `resp` has focus (2 px accent, outside).
pub fn focus_ring(ui: &Ui, resp: &Response, rect: egui::Rect) {
    if resp.has_focus() {
        let t = Tokens::get(ui.ctx());
        ui.painter().rect_stroke(rect.expand(2.0), t.radius + 2.0, Stroke::new(2.0, t.accent), egui::StrokeKind::Outside);
    }
}

/// A card frame: a soft rounded box with a hairline edge.
pub fn card(t: &Tokens) -> Frame {
    Frame::new().fill(t.card).stroke(Stroke::new(1.0, t.card_border)).corner_radius(t.r_lg()).inner_margin(Margin::same(16))
}

/// A quieter, denser panel frame for the workbench.
pub fn panel(t: &Tokens) -> Frame {
    Frame::new().fill(t.card).stroke(Stroke::new(1.0, t.card_border)).corner_radius(t.r()).inner_margin(Margin::same(10))
}

/// A panel's background: `fill` (a surface token) with `margin` (from the spacing scale) inside.
/// For side panels, bars and the central pane.
pub fn pane(fill: Color32, margin: Margin) -> Frame {
    Frame::new().fill(fill).inner_margin(margin)
}

/// Room around content with no background of its own.
pub fn inset(margin: Margin) -> Frame {
    Frame::new().inner_margin(margin)
}

/// A dialog or a floating sheet: a card with roomier padding.
pub fn dialog(t: &Tokens) -> Frame {
    card(t).inner_margin(theme::pad(space::XL))
}

/// A card that needs the eye (a question, a warning, the step being worked on): an emphasis
/// edge in `edge`.
pub fn callout(t: &Tokens, edge: Color32) -> Frame {
    Frame::new().fill(t.card).stroke(theme::line(theme::EMPHASIS, edge)).corner_radius(t.r_lg()).inner_margin(theme::pad(space::L))
}

/// A soft, edgeless note on the accent wash.
pub fn callout_soft(t: &Tokens) -> Frame {
    Frame::new().fill(t.accent_soft).corner_radius(t.r_lg()).inner_margin(theme::pad(space::L))
}

/// A tag or a small chip: the raised surface, tight padding.
pub fn chip(t: &Tokens) -> Frame {
    Frame::new().fill(t.raised).corner_radius(t.r_sm()).inner_margin(theme::pad_xy(space::SM, space::XXS))
}

/// A block of code or raw text.
pub fn code_block(t: &Tokens) -> Frame {
    Frame::new().fill(t.code_bg).corner_radius(t.r()).inner_margin(theme::pad(space::SM))
}

/// A section of a side panel or a sheet: a hairline above (except the first), then its title
/// in the caption style. Sections replace a stack of separate cards, so a panel reads as one list.
pub fn section(ui: &mut Ui, title: &str, first: bool) {
    let t = Tokens::get(ui.ctx());
    if !first {
        ui.add_space(space::M);
        let y = ui.cursor().top();
        ui.painter().hline(ui.max_rect().x_range(), y, theme::hairline(t.separator));
        ui.add_space(space::M);
    }
    caption(ui, title);
    ui.add_space(space::XS);
}

/// How wide a property's label column is, so values line up down a panel.
pub const PROPERTY_LABEL_W: f32 = 88.0;

/// One property: a faint label in a fixed column and its value beside it, one [`size::ROW`]
/// high (taller when the value wraps). `value` draws the value; it gets the rest of the row.
pub fn property<R>(ui: &mut Ui, label: &str, value: impl FnOnce(&mut Ui) -> R) -> R {
    let t = Tokens::get(ui.ctx());
    ui.horizontal_top(|ui| {
        ui.set_min_height(size::ROW);
        // A real label (screen readers read it), in its own column so values line up.
        ui.allocate_ui_with_layout(Vec2::new(PROPERTY_LABEL_W, size::ROW), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.set_width(PROPERTY_LABEL_W);
            ui.add(egui::Label::new(RichText::new(tr(label)).size(theme::SMALL).color(t.text_faint)).wrap());
        });
        ui.vertical(|ui| {
            ui.add_space(((size::ROW - ui.text_style_height(&egui::TextStyle::Body)) / 2.0).max(0.0));
            value(ui)
        })
        .inner
    })
    .inner
}

/// A property whose value is plain text, cut to one line with the full text on hover.
pub fn property_text(ui: &mut Ui, label: &str, value: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    property(ui, label, |ui| ui.add(egui::Label::new(RichText::new(value).size(theme::SMALL).color(t.text)).truncate()))
}

/// A feature icon on its tinted square (a project's kind, a target's kind).
pub fn icon_tile(ui: &mut Ui, icon: &str, side: f32) -> Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
    ui.painter().rect_filled(rect, t.radius, t.accent_soft);
    let glyph = if side >= size::TILE_LG { size::ICON_XL } else { size::ICON_LG };
    icons::paint(ui, rect, icon, glyph, t.accent_ink);
    resp
}

/// A compact radio-like chip with an icon: the chosen one has the app-colour edge, a wash and a
/// tick. One [`size::CONTROL_LG`] high and as wide as its words plus the tick's slot, which every
/// chip keeps so choosing one never moves the others or changes how the row wraps.
pub fn choice_chip(ui: &mut Ui, icon: &str, label: &str, on: bool) -> Response {
    let t = Tokens::get(ui.ctx());
    let g = label_galley(ui, &tr(label), theme::SMALL, t.text);
    let tick = size::ICON_SM + space::S;
    let w = space::M + size::ICON + space::S + g.size().x + tick + space::M;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, size::CONTROL_LG), Sense::click());
    let h = hover_t(ui, &resp);
    let (fill, stroke) = if on {
        (t.accent_soft.gamma_multiply(0.6), theme::line(theme::EMPHASIS, t.accent))
    } else {
        (theme::mix(Color32::TRANSPARENT, t.hover, h), theme::hairline(theme::mix(t.field_border, t.text_faint, h)))
    };
    ui.painter().rect(rect, t.radius, fill, stroke, egui::StrokeKind::Inside);
    let mut x = rect.left() + space::M;
    icons::paint(
        ui,
        egui::Rect::from_min_size(egui::pos2(x, rect.center().y - size::ICON / 2.0), Vec2::splat(size::ICON)),
        icon,
        size::ICON,
        if on { t.accent_ink } else { t.icon },
    );
    x += size::ICON + space::S;
    let gw = g.size().x;
    ui.painter().galley_with_override_text_color(egui::pos2(x, rect.center().y - g.size().y / 2.0), g, t.text);
    if on {
        let at = egui::pos2(x + gw + space::S, rect.center().y - size::ICON_SM / 2.0);
        icons::paint(ui, egui::Rect::from_min_size(at, Vec2::splat(size::ICON_SM)), "check", size::ICON_SM, t.accent_ink);
    }
    focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::RadioButton, true, on, tr(label)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// The main action: a button filled in the app colour.
pub fn primary(ui: &mut Ui, label: &str) -> Response {
    primary_icon(ui, None, label, true)
}

pub fn primary_icon(ui: &mut Ui, icon: Option<&str>, label: &str, enabled: bool) -> Response {
    primary_sized(ui, icon, label, enabled, size::CONTROL, theme::SMALL)
}

/// The main button at a given height and label size (Simple mode's Start is a big one): filled
/// in the app colour, the one place on a screen it is used as a fill.
pub fn primary_sized(ui: &mut Ui, icon: Option<&str>, label: &str, enabled: bool, height: f32, text: f32) -> Response {
    let t = Tokens::get(ui.ctx());
    let galley = label_galley(ui, &tr(label), text, t.on_accent);
    let icon_w = if icon.is_some() { BUTTON_ICON_W } else { 0.0 };
    let pad = ui.spacing().button_padding;
    let size =
        Vec2::new(galley.size().x + icon_w + pad.x * 2.0 + 8.0 + (height - size::CONTROL_LG).max(0.0), (galley.size().y + pad.y * 2.0 + 4.0).max(height));
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(size, sense);
    let fill = if !enabled {
        t.pressed
    } else if resp.is_pointer_button_down_on() {
        t.accent_hover
    } else {
        theme::mix(t.accent, t.accent_hover, hover_t(ui, &resp))
    };
    ui.painter().rect_filled(rect, t.radius, fill);
    let fg = if enabled { t.on_accent } else { t.text_faint };
    button_content(ui, rect, icon, galley, fg, fg);
    focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, enabled, tr(label)));
    if enabled { resp.on_hover_cursor(egui::CursorIcon::PointingHand) } else { resp }
}

/// Secondary button: a card-coloured box with a soft edge and an optional icon.
pub fn secondary(ui: &mut Ui, icon: Option<&str>, label: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let galley = label_galley(ui, &tr(label), theme::SMALL, t.text);
    let icon_w = if icon.is_some() { BUTTON_ICON_W } else { 0.0 };
    let pad = ui.spacing().button_padding;
    let size = Vec2::new(galley.size().x + icon_w + pad.x * 2.0 + 6.0, (galley.size().y + pad.y * 2.0 + 4.0).max(size::CONTROL));
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let h = hover_t(ui, &resp);
    let fill = if resp.is_pointer_button_down_on() { t.pressed } else { theme::mix(t.card, t.hover, h) };
    let stroke = theme::mix(t.card_border, t.field_border, h);
    ui.painter().rect(rect, t.radius, fill, Stroke::new(1.0, stroke), egui::StrokeKind::Inside);
    button_content(ui, rect, icon, galley, t.text, t.icon);
    focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(label)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A button's icon and label, centred together in `rect`.
fn button_content(ui: &Ui, rect: egui::Rect, icon: Option<&str>, galley: std::sync::Arc<egui::Galley>, fg: Color32, icon_tint: Color32) {
    let icon_w = if icon.is_some() { BUTTON_ICON_W } else { 0.0 };
    let mut x = (rect.center().x - (icon_w + galley.size().x) / 2.0).round();
    if let Some(i) = icon {
        icons::paint(ui, egui::Rect::from_min_size(egui::pos2(x, rect.center().y - size::ICON / 2.0), Vec2::splat(size::ICON)), i, size::ICON, icon_tint);
        x += icon_w;
    }
    ui.painter().galley_with_override_text_color(egui::pos2(x, rect.center().y - galley.size().y / 2.0), galley, fg);
}

/// A button icon (16 px) and the gap after it.
const BUTTON_ICON_W: f32 = 22.0;

/// A borderless text button in the accent colour.
pub fn link(ui: &mut Ui, label: &str) -> Response {
    let t = Tokens::get(ui.ctx());
    let r = ui.add(egui::Label::new(RichText::new(tr(label)).color(t.accent_ink)).sense(Sense::click()));
    let r = r.on_hover_cursor(egui::CursorIcon::PointingHand);
    if r.hovered() {
        let y = r.rect.bottom();
        ui.painter().hline(r.rect.x_range(), y, Stroke::new(1.0, t.accent_ink));
    }
    r
}

/// A segmented switch; the selected segment is a raised card on a sunken track. Returns the newly
/// picked index when it changes.
pub fn segmented(ui: &mut Ui, options: &[&str], selected: usize, height: f32) -> Option<usize> {
    segmented_sized(ui, options, selected, height, theme::SMALL)
}

/// [`segmented`] with a given label size (one of the type scale's), for a bigger switch.
pub fn segmented_sized(ui: &mut Ui, options: &[&str], selected: usize, height: f32, text: f32) -> Option<usize> {
    segmented_scaled(ui, options, selected, height, text, 1.0)
}

/// [`segmented_sized`] with every size in it, padding too, multiplied by `scale` (the macOS
/// title bar's, which keeps its size when the app zooms).
pub fn segmented_scaled(ui: &mut Ui, options: &[&str], selected: usize, height: f32, text: f32, scale: f32) -> Option<usize> {
    let t = Tokens::get(ui.ctx());
    let label = text * scale;
    let (height, pad) = (height * scale, space::XXS * scale);
    let galleys: Vec<_> = options.iter().map(|o| label_galley(ui, &tr(o), label, t.text)).collect();
    let widths: Vec<f32> = galleys.iter().map(|g| segment_width(g.size().x, scale)).collect();
    let total: f32 = widths.iter().sum::<f32>() + 2.0 * pad;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total, height), Sense::hover());
    ui.painter().rect(rect, t.radius, t.raised, Stroke::new(scale, t.card_border), egui::StrokeKind::Inside);
    let mut x = rect.left() + pad;
    let mut picked = None;
    for (i, (g, w)) in galleys.into_iter().zip(widths).enumerate() {
        let seg = egui::Rect::from_min_size(egui::pos2(x, rect.top() + pad), Vec2::new(w, height - 2.0 * pad));
        let resp = ui.interact(seg, ui.id().with(("seg", options.get(i).copied().unwrap_or_default())), Sense::click());
        let on = i == selected;
        resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, on, tr(options.get(i).copied().unwrap_or_default())));
        if on {
            ui.painter().rect(seg, t.radius_sm, t.card, Stroke::new(scale, t.card_border), egui::StrokeKind::Inside);
        } else {
            ui.painter().rect_filled(seg, t.radius_sm, theme::mix(Color32::TRANSPARENT, t.hover, hover_t(ui, &resp)));
        }
        let color = if on { t.text } else { t.text_dim };
        ui.painter().galley_with_override_text_color(seg.center() - g.size() / 2.0, g, color);
        focus_ring(ui, &resp, seg);
        let resp = if on { resp } else { resp.on_hover_cursor(egui::CursorIcon::PointingHand) };
        if resp.clicked() && !on {
            picked = Some(i);
        }
        x += w;
    }
    picked
}

/// A segment's width: its label and a fixed padding either side (scaled with the title bar).
fn segment_width(label: f32, scale: f32) -> f32 {
    label + (space::M * 2.0 + space::XXS) * scale
}

/// How wide [`segmented`] draws `options`, for a row that needs to know before drawing them.
pub fn segmented_width(ui: &Ui, options: &[&str]) -> f32 {
    let t = Tokens::get(ui.ctx());
    options.iter().map(|o| segment_width(label_galley(ui, &tr(o), theme::SMALL, t.text).size().x, 1.0)).sum::<f32>() + 2.0 * space::XXS
}

/// A small status label on a tint.
pub fn pill(ui: &mut Ui, text: &str, fg: Color32, bg: Color32) -> Response {
    pill_sized(ui, text, fg, bg, theme::CAPTION)
}

/// [`pill`] with a given label size, its padding scaled to match.
pub fn pill_sized(ui: &mut Ui, text: &str, fg: Color32, bg: Color32, text_size: f32) -> Response {
    let t = Tokens::get(ui.ctx());
    let label = tr(text);
    let g = label_galley(ui, &label, text_size, fg);
    let size = Vec2::new(g.size().x + text_size * 14.0 / 11.0, g.size().y + text_size * 8.0 / 11.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter().rect_filled(rect, t.r_sm(), bg);
    ui.painter().galley_with_override_text_color(rect.center() - g.size() / 2.0, g, fg);
    // Painted text: say it to screen readers too.
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, &label));
    resp
}

/// The "how sure are we" badge: a coloured dot and the plain label.
pub fn confidence_badge(ui: &mut Ui, c: Option<Confidence>) -> Response {
    let t = Tokens::get(ui.ctx());
    let col = confidence_color(c);
    let font = theme::medium(theme::CAPTION);
    let g = ui.painter().layout_no_wrap(tr(confidence_label(c)), font, col);
    let size = Vec2::new(g.size().x + 26.0, g.size().y + 7.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    let bg = Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), 34);
    ui.painter().rect(rect, t.r_sm(), bg, Stroke::new(1.0, col.gamma_multiply(0.45)), egui::StrokeKind::Inside);
    ui.painter().circle_filled(egui::pos2(rect.left() + 10.0, rect.center().y), 3.5, col);
    ui.painter().galley_with_override_text_color(egui::pos2(rect.left() + 18.0, rect.center().y - g.size().y / 2.0), g, col);
    let tip = match c {
        Some(Confidence::Observed) => "Seen it: read directly from the file, page or capture.",
        Some(Confidence::Derived) => "Worked it out: computed step by step from things we saw.",
        Some(Confidence::Inferred) => "Best guess: a reasonable reading, but not proven.",
        None => "Unknown: we couldn't establish this yet.",
    };
    resp.on_hover_text(tr(tip))
}

/// A section caption in the label style, faint.
pub fn caption(ui: &mut Ui, text: &str) {
    let t = Tokens::get(ui.ctx());
    ui.label(theme::label_text(&tr(text), theme::CAPTION).color(t.text_faint));
}

/// A centred, calm empty state.
pub fn empty_state(ui: &mut Ui, icon: &str, title: &str, body: &str) {
    let t = Tokens::get(ui.ctx());
    ui.vertical_centered(|ui| {
        ui.add_space(18.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(44.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 22.0, t.raised);
        icons::paint(ui, rect, icon, 22.0, t.text_faint);
        ui.add_space(6.0);
        ui.label(RichText::new(tr(title)).font(theme::medium(theme::SMALL)).color(t.text_dim));
        if !body.is_empty() {
            ui.label(RichText::new(tr(body)).color(t.text_faint).size(theme::CAPTION));
        }
        ui.add_space(12.0);
    });
}

/// A big clickable card with an icon, title and blurb (Beginner Home).
pub fn big_card(ui: &mut Ui, size: Vec2, icon: &str, title: &str, blurb: &str, badge: Option<&str>) -> Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();
    let fill = t.card;
    let stroke = Stroke::new(1.0, if hovered { t.field_border } else { t.card_border });
    if hovered {
        ui.painter().add(egui::Shadow { offset: [0, 8], blur: 24, spread: 0, color: t.shadow }.as_shape(rect, t.r_lg()));
    }
    ui.painter().rect(rect, t.radius_lg, fill, stroke, egui::StrokeKind::Inside);
    let icon_rect = egui::Rect::from_min_size(rect.min + Vec2::new(22.0, 22.0), Vec2::splat(52.0));
    ui.painter().rect_filled(icon_rect, t.radius, t.accent_soft);
    icons::paint(ui, icon_rect, icon, 28.0, t.accent_ink);
    if let Some(b) = badge {
        let g = ui.painter().layout_no_wrap(tr(b), theme::medium(theme::TINY), t.text_dim);
        let br = egui::Rect::from_min_size(egui::pos2(rect.right() - g.size().x - 34.0, rect.top() + 22.0), g.size() + Vec2::new(16.0, 8.0));
        ui.painter().rect_filled(br, t.r_sm(), t.hover);
        ui.painter().galley_with_override_text_color(br.center() - g.size() / 2.0, g, t.text_dim);
    }
    let title_pos = egui::pos2(rect.left() + 22.0, icon_rect.bottom() + 18.0);
    // Titles wrap: the display face is wide, and translations are longer.
    let title_g = theme::heading_galley(ui.painter(), tr(title), theme::CARD, t.text, rect.width() - 44.0);
    let title_h = title_g.size().y;
    ui.painter().galley(title_pos, title_g, t.text);
    let wrap = ui.painter().layout(tr(blurb), egui::FontId::proportional(theme::SMALL), t.text_dim, rect.width() - 44.0);
    ui.painter().galley(title_pos + Vec2::new(0.0, title_h + 8.0), wrap, t.text_dim);
    focus_ring(ui, &resp, rect);
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tr(title)));
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A dotted underline under a hoverable glossary word.
pub fn glossary_word(ui: &mut Ui, word: &str, meaning: &str, font: egui::FontId) -> Response {
    let t = Tokens::get(ui.ctx());
    let resp = ui.add(egui::Label::new(RichText::new(word).font(font).color(t.text)).sense(Sense::hover()));
    let y = resp.rect.bottom() - 1.0;
    let mut x = resp.rect.left();
    let trimmed_right = resp.rect.right() - if word.ends_with(' ') { 4.0 } else { 0.0 };
    while x < trimmed_right {
        ui.painter().hline(x..=(x + 2.0).min(trimmed_right), y, Stroke::new(1.0, t.accent));
        x += 4.0;
    }
    resp.on_hover_ui(|ui| {
        ui.label(RichText::new(word.trim()).font(theme::semibold(theme::SMALL)));
        ui.label(tr(meaning));
    })
}

/// Draw an animation's next frame in `every` (about 15 a second is smooth enough for a spinner),
/// or once a second while the window is in the background: a build can run for an hour, and
/// redrawing the whole window 30 times a second for it costs the battery.
pub fn animate(ui: &Ui, every: std::time::Duration) {
    let focused = ui.input(|i| i.viewport().focused.unwrap_or(true));
    ui.ctx().request_repaint_after(if focused { every } else { std::time::Duration::from_secs(1) });
}

/// One frame of an animation at the usual pace.
pub const FRAME: std::time::Duration = std::time::Duration::from_millis(66);

/// A spinning arc for "working", drawn like egui's own `Spinner` but asking for its next frame at
/// [`animate`]'s pace. egui's asks to be drawn again at once, every frame, for as long as it's on
/// screen: a build or a page capture can run for an hour, and redrawing the whole window at the
/// display's rate for it kept a CPU core (or, drawing in software, all of them) busy (#579).
#[must_use = "put it in a ui with `ui.add(widget);`"]
#[derive(Default)]
pub struct Spinner {
    size: Option<f32>,
    color: Option<Color32>,
}

impl Spinner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}

impl egui::Widget for Spinner {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = self.size.unwrap_or_else(|| ui.style().spacing.interact_size.y);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        response.widget_info(|| egui::WidgetInfo::new(egui::WidgetType::ProgressIndicator));
        if ui.is_rect_visible(rect) {
            let color = self.color.unwrap_or_else(|| ui.visuals().strong_text_color());
            let radius = (rect.height().min(rect.width()) / 2.0) - 2.0;
            let n_points = (radius.round() as u32).clamp(8, 128);
            let time = ui.input(|i| i.time);
            let start = time * std::f64::consts::TAU;
            let end = start + 240f64.to_radians() * time.sin();
            let points: Vec<egui::Pos2> = (0..n_points)
                .map(|i| {
                    let angle = start + (end - start) * (i as f64 / n_points as f64);
                    let (sin, cos) = angle.sin_cos();
                    rect.center() + radius * egui::vec2(cos as f32, sin as f32)
                })
                .collect();
            ui.painter().add(egui::Shape::line(points, Stroke::new(3.0, color)));
            animate(ui, FRAME);
        }
        response
    }
}

/// A spinner-ish dot row for "thinking".
pub fn thinking(ui: &mut Ui) {
    let t = Tokens::get(ui.ctx());
    let time = ui.input(|i| i.time);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(36.0, 14.0), Sense::hover());
    for i in 0..3 {
        let phase = ((time * 3.0 - i as f64 * 0.5).sin() * 0.5 + 0.5) as f32;
        let c = t.accent.gamma_multiply(0.35 + 0.65 * phase);
        ui.painter().circle_filled(egui::pos2(rect.left() + 6.0 + i as f32 * 12.0, rect.center().y), 3.5, c);
    }
    animate(ui, FRAME);
}

/// How long a run may say nothing before the note points it out.
const QUIET_NOTE_AFTER: std::time::Duration = std::time::Duration::from_secs(60);

fn wait_note(secs: u64, quiet: Option<std::time::Duration>) -> Option<String> {
    match quiet {
        Some(q) if q >= QUIET_NOTE_AFTER => Some(crate::i18n::trf(
            "Nothing new for {time}. Long steps can be quiet; if it stays like this, press Pause.",
            &[("time", &crate::usage::duration_words(q))],
        )),
        Some(_) => None,
        None => match secs {
            0..20 => None,
            20..90 => Some(tr("Still working. Bigger jobs can take a few minutes.")),
            _ => Some(tr("This is a big one. You can leave this open and come back later.")),
        },
    }
}

/// "Still working" for waits longer than a moment: a spinning arc, what's happening, how long
/// it has taken, and a calm note once it runs long (so a quiet wait never looks stuck).
/// `title` is translated here; `detail` arrives already in the user's words.
pub fn working(ui: &mut Ui, title: &str, detail: Option<&str>) {
    working_since(ui, title, detail, None, None);
}

/// [`working`] with its clock from `began` (a build's start), so it doesn't restart each time the
/// row comes back; without it, from when the row was last shown.
///
/// `quiet` is how long since the work last had anything to say, when that's known: then the note
/// says so plainly, because a calm note can't tell a long step from one that has stopped.
pub fn working_since(ui: &mut Ui, title: &str, detail: Option<&str>, began: Option<std::time::Instant>, quiet: Option<std::time::Duration>) {
    let t = Tokens::get(ui.ctx());
    let now = ui.input(|i| i.time);
    // The clock restarts when the row wasn't on screen a moment ago.
    let id = ui.id().with("working-since");
    let (since, seen) = ui.data(|d| d.get_temp::<(f64, f64)>(id)).unwrap_or((now, now));
    let since = if now - seen > 1.0 { now } else { since };
    ui.data_mut(|d| d.insert_temp(id, (since, now)));
    let secs = began.map_or((now - since) as u64, |b| b.elapsed().as_secs());
    let note = wait_note(secs, quiet);
    let sub = [detail.map(str::to_string), note].into_iter().flatten().collect::<Vec<_>>().join("  ·  ");
    card(&t).inner_margin(theme::pad_xy(space::L, space::M)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        // The clock is measured first and the words get only what's left, so they wrap before it.
        let clock = crate::usage::duration_words(std::time::Duration::from_secs(secs));
        let clock = ui.painter().layout_no_wrap(clock, theme::mono(theme::CAPTION), t.text_faint);
        ui.horizontal_top(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(size::ICON_XL), Sense::hover());
            let (c, radius) = (r.center(), size::ICON_XL / 2.0 - theme::FOCUS);
            ui.painter().circle_stroke(c, radius, theme::line(theme::FOCUS, t.separator));
            let start = (now * 4.0) as f32;
            let sweep = std::f32::consts::PI * (0.9 + 0.5 * (now * 1.7).sin() as f32);
            let arc = (0..=20).map(|i| c + radius * Vec2::angled(start + sweep * i as f32 / 20.0)).collect();
            ui.painter().add(egui::Shape::line(arc, theme::line(theme::FOCUS, t.accent)));
            ui.add_space(space::XS);
            let words_w = (ui.available_width() - clock.size().x - space::M).max(0.0);
            ui.allocate_ui_with_layout(Vec2::new(words_w, 0.0), egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.set_width(words_w);
                ui.label(RichText::new(tr(title)).font(theme::semibold(theme::BODY)).color(t.text));
                if !sub.is_empty() {
                    ui.add(egui::Label::new(RichText::new(sub).color(t.text_dim).size(theme::SMALL)).wrap());
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                ui.label(RichText::new(clock.text()).font(theme::mono(theme::CAPTION)).color(t.text_faint));
            });
        });
        // An indeterminate sweep along the bottom: movement even when nothing new has arrived.
        ui.add_space(space::M);
        let (bar, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 2.0), Sense::hover());
        ui.painter().rect_filled(bar, 1.0, t.separator);
        let seg = bar.width() * 0.25;
        let x = bar.left() - seg + (bar.width() + seg) * ((now * 0.6).fract() as f32);
        let lit = egui::Rect::from_x_y_ranges(x.max(bar.left())..=(x + seg).min(bar.right()), bar.y_range());
        ui.painter().rect_filled(lit, 1.0, t.accent);
    });
    animate(ui, FRAME);
}

/// The button that opens a failure's log; hovering shows where the log is. True when clicked.
pub fn open_log_button(ui: &mut Ui, path: &std::path::Path) -> bool {
    ui.add_space(space::S);
    action(ui, "Open log").on_hover_text(path.display().to_string()).clicked()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Every action's icon is bundled: a missing one would quietly draw as a square.
    #[test]
    fn every_action_icon_is_bundled() {
        for (action, icon) in ACTION_ICONS {
            assert!(icons::exists(icon), "{action}: no icon {icon}");
        }
    }

    /// Choosing another chip leaves every chip where it was: the tick's slot is kept on all of them.
    #[test]
    fn choosing_a_chip_moves_no_chip() {
        use egui_kittest::kittest::Queryable as _;
        let labels = ["A website", "An app", "A game", "Nothing — start fresh"];
        // (chosen chip, fonts installed): the kit's fonts are bound from the frame after install.
        let mut h = egui_kittest::Harness::new_ui_state(
            move |ui, (chosen, ready): &mut (usize, bool)| {
                if !*ready {
                    theme::install_fonts(ui.ctx());
                    *ready = true;
                    return;
                }
                ui.horizontal_wrapped(|ui| {
                    for (i, l) in labels.iter().enumerate() {
                        choice_chip(ui, "globe", l, i == *chosen);
                    }
                });
            },
            (0usize, false),
        );
        h.run();
        let rects = |h: &egui_kittest::Harness<'_, (usize, bool)>| labels.map(|l| h.get_by_label(l).rect());
        let first = rects(&h);
        for chosen in 1..labels.len() {
            h.state_mut().0 = chosen;
            h.run();
            assert_eq!(rects(&h), first, "choosing chip {chosen} moved a chip");
        }
    }

    /// #579: egui's `Spinner` asks to be redrawn at once, every frame, for as long as it shows;
    /// ours waits for the animation's own pace, so an hour-long build isn't drawn at the display's
    /// rate. Both are drawn here for comparison.
    #[test]
    fn a_spinner_on_screen_does_not_ask_for_a_frame_at_once() {
        let wait_after = |add: &dyn Fn(&mut Ui)| {
            let ctx = egui::Context::default();
            let mut raw = egui::RawInput::default();
            raw.viewports.entry(egui::ViewportId::ROOT).or_default().focused = Some(true);
            // The first frames always ask for another (layout settles); look at a settled one.
            let mut out = egui::FullOutput::default();
            for _ in 0..4 {
                out = ctx.run_ui(raw.clone(), |ui| add(ui));
                out.textures_delta.clear();
            }
            out.viewport_output.get(&egui::ViewportId::ROOT).map(|v| v.repaint_delay).unwrap_or_default()
        };
        let egui_spinner = wait_after(&|ui| {
            ui.add(egui::Spinner::new());
        });
        assert!(egui_spinner < Duration::from_millis(10), "egui's own asks for the next frame at once: {egui_spinner:?}");
        let ours = wait_after(&|ui| {
            ui.add(Spinner::new().size(16.0));
        });
        assert!(ours >= Duration::from_millis(30), "ours waits for the animation's pace: {ours:?}");
    }

    /// #579: every screen uses our spinner; egui's own would bring the redraw-every-frame back.
    #[test]
    fn no_screen_uses_eguis_own_spinner() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut used = Vec::new();
        for entry in std::fs::read_dir(&src).unwrap().flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "rs") && path.file_name().is_some_and(|n| n != "widgets.rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                if text.lines().any(|l| !l.trim_start().starts_with("//") && l.contains("egui::Spinner")) {
                    used.push(path.file_name().unwrap().to_string_lossy().into_owned());
                }
            }
        }
        assert!(used.is_empty(), "use `widgets::Spinner`, which doesn't ask for a frame every frame: {used:?}");
    }

    #[test]
    fn the_wait_note_says_when_nothing_is_happening() {
        let quiet = wait_note(4000, Some(Duration::from_secs(150))).unwrap();
        assert!(quiet.contains("Nothing new for") && !quiet.contains("big one"), "{quiet}");
        assert_eq!(wait_note(4000, Some(Duration::from_secs(5))), None, "it's visibly moving: nothing to add");
        assert!(wait_note(4000, None).unwrap().contains("big one"), "without news to go on, the old note");
    }
}
