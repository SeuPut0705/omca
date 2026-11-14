//! Design system: two themes (Studio dark, Studio Light), colour tokens, radii and type.
//!
//! A calm desktop look: neutral graphite (or near-white) surfaces, a slightly darker sidebar,
//! soft rounded cards and controls, sentence-case Pretendard throughout, and app-colour main buttons.
//! Indigo is the app colour for highlights, progress, links and selection.
//! Widgets read [`Tokens::get`] instead of hard-coding colours, so a theme switch applies everywhere.
//! Confidence has its own fixed palette so "how sure are we" reads the same in both themes.

use craft_anything_engine::evidence::Confidence;
use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle, Visuals};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeKind {
    #[default]
    Studio,
    StudioLight,
}

impl ThemeKind {
    pub const ALL: [ThemeKind; 2] = [ThemeKind::Studio, ThemeKind::StudioLight];

    pub fn label(self) -> &'static str {
        match self {
            ThemeKind::Studio => "Studio (dark)",
            ThemeKind::StudioLight => "Studio Light",
        }
    }

    /// The value stored in `settings.theme`.
    pub fn id(self) -> &'static str {
        match self {
            ThemeKind::Studio => "studio",
            ThemeKind::StudioLight => "studio-light",
        }
    }

    pub fn from_id(s: &str) -> ThemeKind {
        match s.trim().to_ascii_lowercase().replace(['_', ' '], "-").as_str() {
            "studio-light" | "light" => ThemeKind::StudioLight,
            _ => ThemeKind::Studio,
        }
    }
}

/// Colour and shape tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tokens {
    pub kind: ThemeKind,
    /// Title bar, rails, panels.
    pub chrome: Color32,
    /// The main reading surface behind cards.
    pub canvas: Color32,
    pub card: Color32,
    pub card_border: Color32,
    /// A card a step above `card` (hovered cards, chips).
    pub raised: Color32,
    pub field: Color32,
    pub field_border: Color32,
    pub hover: Color32,
    pub pressed: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_faint: Color32,
    pub icon: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    /// Indigo that reads as text on the theme's surfaces.
    pub accent_ink: Color32,
    /// Indigo wash for soft highlights.
    pub accent_soft: Color32,
    /// Text drawn on an `accent` fill.
    pub on_accent: Color32,
    pub separator: Color32,
    pub shadow: Color32,
    pub danger: Color32,
    pub warning: Color32,
    pub success: Color32,
    pub code_bg: Color32,
    pub radius_sm: f32,
    pub radius: f32,
    pub radius_lg: f32,
}

/// The app colour: OMCA's indigo, from the app icon's #272b6e lifted until it reads on screen.
/// White on it is 6.2:1 (WCAG AA); `accent_hover` is a step lighter.
pub const BRAND: Color32 = Color32::from_rgb(0x4b, 0x4f, 0xd6);

pub const SEEN_IT: Color32 = Color32::from_rgb(0x3f, 0xb2, 0x7f);
pub const WORKED_OUT: Color32 = Color32::from_rgb(0x5b, 0x8d, 0xef);
pub const BEST_GUESS: Color32 = Color32::from_rgb(0xe0, 0xa2, 0x3a);
pub const UNKNOWN: Color32 = Color32::from_rgb(0x8a, 0x8a, 0x93);

/// The colour for a confidence level (`None` = unknown).
pub fn confidence_color(c: Option<Confidence>) -> Color32 {
    match c {
        Some(Confidence::Observed) => SEEN_IT,
        Some(Confidence::Derived) => WORKED_OUT,
        Some(Confidence::Inferred) => BEST_GUESS,
        None => UNKNOWN,
    }
}

pub fn confidence_label(c: Option<Confidence>) -> &'static str {
    c.map(Confidence::plain_label).unwrap_or("Unknown")
}

impl Tokens {
    pub fn for_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Studio => Tokens {
                kind,
                chrome: Color32::from_rgb(0x19, 0x19, 0x1c),
                canvas: Color32::from_rgb(0x19, 0x19, 0x1c),
                card: Color32::from_rgb(0x23, 0x23, 0x27),
                card_border: Color32::from_rgb(0x3a, 0x3a, 0x41),
                raised: Color32::from_rgb(0x23, 0x23, 0x27),
                field: Color32::from_rgb(0x1f, 0x1f, 0x23),
                field_border: Color32::from_rgb(0x68, 0x68, 0x72),
                hover: Color32::from_rgb(0x2c, 0x2c, 0x31),
                pressed: Color32::from_rgb(0x34, 0x34, 0x3b),
                text: Color32::from_rgb(0xed, 0xed, 0xf0),
                text_dim: Color32::from_rgb(0xb4, 0xb4, 0xbe),
                text_faint: Color32::from_rgb(0xa1, 0xa1, 0xaa),
                icon: Color32::from_rgb(0xb4, 0xb4, 0xbe),
                accent: BRAND,
                accent_hover: Color32::from_rgb(0x5a, 0x5e, 0xe4),
                accent_ink: Color32::from_rgb(0xa9, 0xab, 0xff),
                accent_soft: Color32::from_rgba_unmultiplied(0x63, 0x67, 0xe8, 40),
                on_accent: Color32::WHITE,
                separator: Color32::from_rgb(0x32, 0x32, 0x38),
                shadow: Color32::from_black_alpha(120),
                danger: Color32::from_rgb(0xff, 0x6b, 0x63),
                warning: Color32::from_rgb(0xf0, 0xb2, 0x52),
                success: Color32::from_rgb(0x5c, 0xc2, 0x93),
                code_bg: Color32::from_rgb(0x09, 0x0a, 0x0e),
                radius_sm: 8.0,
                radius: 12.0,
                radius_lg: 16.0,
            },
            ThemeKind::StudioLight => Tokens {
                kind,
                chrome: Color32::from_rgb(0xf4, 0xf4, 0xf6),
                canvas: Color32::from_rgb(0xff, 0xff, 0xff),
                card: Color32::from_rgb(0xff, 0xff, 0xff),
                card_border: Color32::from_rgb(0xdc, 0xdc, 0xe2),
                raised: Color32::from_rgb(0xf4, 0xf4, 0xf6),
                field: Color32::from_rgb(0xff, 0xff, 0xff),
                field_border: Color32::from_rgb(0x85, 0x85, 0x90),
                hover: Color32::from_rgb(0xee, 0xee, 0xf1),
                pressed: Color32::from_rgb(0xe4, 0xe4, 0xe9),
                text: Color32::from_rgb(0x1b, 0x1b, 0x1f),
                text_dim: Color32::from_rgb(0x52, 0x52, 0x5c),
                text_faint: Color32::from_rgb(0x5f, 0x5f, 0x68),
                icon: Color32::from_rgb(0x5f, 0x5f, 0x68),
                accent: BRAND,
                accent_hover: Color32::from_rgb(0x5a, 0x5e, 0xe4),
                accent_ink: BRAND,
                accent_soft: Color32::from_rgba_unmultiplied(0x4b, 0x4f, 0xd6, 26),
                on_accent: Color32::WHITE,
                separator: Color32::from_rgb(0xdc, 0xdc, 0xe2),
                shadow: Color32::from_black_alpha(22),
                danger: Color32::from_rgb(0xb3, 0x26, 0x1e),
                warning: Color32::from_rgb(0x8f, 0x5e, 0x00),
                success: Color32::from_rgb(0x12, 0x75, 0x36),
                code_bg: Color32::from_rgb(0xee, 0xee, 0xf3),
                radius_sm: 8.0,
                radius: 12.0,
                radius_lg: 16.0,
            },
        }
    }

    /// The tokens of the active theme.
    pub fn get(ctx: &egui::Context) -> Tokens {
        ctx.data(|d| d.get_temp::<Tokens>(tokens_id())).unwrap_or_else(|| Tokens::for_kind(ThemeKind::Studio))
    }

    pub fn dark(&self) -> bool {
        self.kind == ThemeKind::Studio
    }

    pub fn r_sm(&self) -> CornerRadius {
        CornerRadius::same(self.radius_sm as u8)
    }
    pub fn r(&self) -> CornerRadius {
        CornerRadius::same(self.radius as u8)
    }
    pub fn r_lg(&self) -> CornerRadius {
        CornerRadius::same(self.radius_lg as u8)
    }
}

fn tokens_id() -> egui::Id {
    egui::Id::new("craft-anything-theme")
}

// ---- fonts -------------------------------------------------------------------------------------

/// Pretendard: one design for Hangul and Latin (its Latin is Inter's), made for screens, so a
/// sentence mixing Korean and English has one height, weight and rhythm. The TrueType builds.
pub static PRETENDARD_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/Pretendard-Regular.ttf");
pub static PRETENDARD_MEDIUM: &[u8] = include_bytes!("../../../assets/fonts/Pretendard-Medium.ttf");
pub static PRETENDARD_SEMIBOLD: &[u8] = include_bytes!("../../../assets/fonts/Pretendard-SemiBold.ttf");
pub static JETBRAINS_MONO: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.ttf");

/// A font from the optional craft-fonts build input (empty unless built with `CRAFT_FONTS_DIR`).
pub struct CraftFont {
    pub family: &'static str,
    pub style: &'static str,
    /// ISO 15924 scripts the font is for, e.g. `"Jpan"`.
    pub scripts: &'static [&'static str],
    pub bytes: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/craft_fonts.rs"));

/// Install Pretendard (three weights: body, labels and buttons, headings), JetBrains Mono (code,
/// with Pretendard behind it so Hangul in code reads in the same face as the rest), plus any
/// craft-fonts as further fallbacks.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let mut add = |name: &str, bytes: &'static [u8]| {
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    };
    add("Pretendard", PRETENDARD_REGULAR);
    add("Pretendard-Medium", PRETENDARD_MEDIUM);
    add("Pretendard-SemiBold", PRETENDARD_SEMIBOLD);
    add("JetBrainsMono", JETBRAINS_MONO);
    let mut fallbacks = Vec::new();
    let mut bold_fallbacks = Vec::new();
    for f in CRAFT_FONTS {
        let name = format!("craft:{}:{}", f.family, f.style);
        fonts.font_data.insert(name.clone(), Arc::new(FontData::from_static(f.bytes)));
        if f.style.eq_ignore_ascii_case("regular") {
            fallbacks.push(name);
        } else if f.style.eq_ignore_ascii_case("bold") {
            bold_fallbacks.push(name);
        }
    }
    // Sans first: a mincho/serif fallback makes Japanese and Chinese UI text look like prose.
    let fallbacks = sans_first(fallbacks);
    let bold_fallbacks = sans_first(bold_fallbacks);
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "Pretendard".to_owned());
    let mono = fonts.families.entry(FontFamily::Monospace).or_default();
    mono.insert(0, "JetBrainsMono".to_owned());
    mono.insert(1, "Pretendard".to_owned());
    for fam in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts.families.entry(fam).or_default().extend(fallbacks.iter().cloned());
    }
    let base: Vec<String> = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for (fam, stack) in weighted_families(&base, &bold_fallbacks) {
        fonts.families.insert(fam, stack);
    }
    ctx.set_fonts(fonts);
}

/// The Pretendard weights first, then bold script fallbacks and the proportional stack.
fn weighted_families(base: &[String], bold: &[String]) -> Vec<(FontFamily, Vec<String>)> {
    [("medium", "Pretendard-Medium"), ("semibold", "Pretendard-SemiBold")]
        .into_iter()
        .map(|(fam, primary)| {
            let mut stack = vec![primary.to_owned()];
            stack.extend(bold.iter().cloned());
            stack.extend(base.iter().cloned());
            (FontFamily::Name(fam.into()), stack)
        })
        .collect()
}

/// Fallback faces, most wanted first. A stable sort keeps the manifest's order inside a rank.
fn sans_first(fallbacks: Vec<String>) -> Vec<String> {
    let mut ranked: Vec<(FallbackRank, String)> = fallbacks.into_iter().map(|n| (fallback_rank(&n), n)).collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked.into_iter().map(|(_, name)| name).collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum FallbackRank {
    /// A face made for the interface's language (BIZ UDPGothic for Japanese).
    Local,
    /// One pan-CJK sans.
    Cjk,
    /// Any other sans or gothic.
    Sans,
    /// Nothing known about it.
    Other,
    /// A book face: last, because interface text set in one reads like prose.
    Serif,
}

/// Which face a craft-fonts name looks like, from the family name: a face made for the interface's
/// language first, a book face last.
fn fallback_rank(name: &str) -> FallbackRank {
    let n = name.to_ascii_lowercase();
    if n.contains("udpgothic") {
        FallbackRank::Local
    } else if n.contains("sans cjk") {
        FallbackRank::Cjk
    } else if n.contains("gothic") || n.contains("sans") {
        FallbackRank::Sans
    } else if n.contains("mincho") || n.contains("serif") {
        FallbackRank::Serif
    } else {
        FallbackRank::Other
    }
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}
/// The type scale: every text size in the interface is one of these, chosen by role.
///
/// Below the headings the steps are 1 pt apart, because at Korean UI sizes a point is already a
/// visible change; the headings grow by about 1.2 each, so a level reads as a level. BODY is the
/// smallest size Hangul stays comfortable for a paragraph in Pretendard; CAPTION is the smallest
/// at which its strokes stay apart; TINY is only for the workbench's dense lists.
///
/// Headings (`heading`, Pretendard SemiBold): the one title on a screen, a dialog's or a
/// section's title, a card's or a list item's title.
pub const TITLE: f32 = 24.0;
pub const SECTION: f32 = 18.0;
pub const CARD: f32 = 15.0;
/// Text: reading text (Simple mode's body), secondary text, list rows, buttons and the
/// Advanced body, fine print (paths, counts, captions), and the workbench's dense rows.
pub const BODY: f32 = 14.0;
pub const SMALL: f32 = 13.0;
pub const CAPTION: f32 = 12.0;
pub const TINY: f32 = 11.0;

/// A heading: Pretendard SemiBold, at one of the heading sizes above.
pub fn heading(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(semibold(size))
}

/// `heading` laid out for painting, wrapped at `wrap`.
pub fn heading_galley(painter: &egui::Painter, text: impl Into<String>, size: f32, color: Color32, wrap: f32) -> Arc<egui::Galley> {
    painter.layout(text.into(), semibold(size), color, wrap)
}
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

/// `a` blended toward `b` by `t` (0..=1), channel by channel (alpha included).
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgba_premultiplied(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()), m(a.a(), b.a()))
}

/// A short label (captions, buttons, the title bar) in sentence case, in the medium weight, at
/// one of the sizes above.
pub fn label_text(text: &str, size: f32) -> egui::RichText {
    egui::RichText::new(text).font(medium(size))
}

/// The spacing scale: every gap, padding and inset in the interface is one of these (a 4 pt grid
/// with the half steps dense rows need). Screens use these names, never a number.
pub mod space {
    pub const XXS: f32 = 2.0;
    pub const XS: f32 = 4.0;
    pub const S: f32 = 6.0;
    pub const SM: f32 = 8.0;
    pub const M: f32 = 12.0;
    pub const L: f32 = 16.0;
    pub const XL: f32 = 24.0;
    pub const XXL: f32 = 32.0;
    pub const XXXL: f32 = 40.0;
}

/// Padding of `all` on every side, from the spacing scale.
pub fn pad(all: f32) -> egui::Margin {
    egui::Margin::same(all as i8)
}

/// Padding of `x` left and right and `y` top and bottom, from the spacing scale.
pub fn pad_xy(x: f32, y: f32) -> egui::Margin {
    egui::Margin::symmetric(x as i8, y as i8)
}

/// Padding set per side, from the spacing scale.
pub fn pad_sides(left: f32, right: f32, top: f32, bottom: f32) -> egui::Margin {
    egui::Margin { left: left as i8, right: right as i8, top: top as i8, bottom: bottom as i8 }
}

/// Line widths: hairlines for edges and dividers, an emphasis edge for a card that needs the
/// eye (a question, a warning), and the focus/selection width.
pub const HAIRLINE: f32 = 1.0;
pub const EMPHASIS: f32 = 1.5;
pub const FOCUS: f32 = 2.0;

/// A line `width` wide (one of [`HAIRLINE`], [`EMPHASIS`], [`FOCUS`]) in `color`.
pub fn line(width: f32, color: Color32) -> Stroke {
    Stroke::new(width, color)
}

/// A hairline in `color`.
pub fn hairline(color: Color32) -> Stroke {
    line(HAIRLINE, color)
}

/// The size scale: every icon, control and row height is one of these. Icons sit on a 4 pt
/// grid; controls and rows are the hit targets they sit in.
pub mod size {
    /// Icons: inline with caption text, with body text, the default, in a heading, a feature
    /// icon, and the large one in an empty state or a project tile.
    pub const ICON_XS: f32 = 12.0;
    pub const ICON_SM: f32 = 14.0;
    pub const ICON: f32 = 16.0;
    pub const ICON_LG: f32 = 20.0;
    pub const ICON_XL: f32 = 24.0;
    pub const ICON_HERO: f32 = 32.0;
    /// Control heights (icon buttons, segmented switches, buttons, fields): a tight toolbar,
    /// a compact row, the default, a prominent one, and the single main action of a screen.
    pub const CONTROL_XS: f32 = 20.0;
    pub const CONTROL_SM: f32 = 24.0;
    pub const CONTROL: f32 = 28.0;
    pub const CONTROL_LG: f32 = 32.0;
    pub const CONTROL_XL: f32 = 40.0;
    /// List and property rows: dense (workbench, ledgers), the default, and a roomy one.
    pub const ROW_DENSE: f32 = 22.0;
    pub const ROW: f32 = 28.0;
    pub const ROW_LG: f32 = 36.0;
    /// The tinted square behind a feature icon (a project's or a target's).
    pub const TILE: f32 = 32.0;
    pub const TILE_LG: f32 = 40.0;
}

/// Scroll bars that can always be seen and grabbed. egui's thin preset paints the thumb in the
/// button fill and its track in the field colour, which here are the same, and narrows it to 2 px
/// at rest, so it showed only on hover. The thumb is the text colour, see-through enough to stay
/// quiet but 3:1 against the surfaces it sits on (WCAG 1.4.11), 6 px at rest and 10 px on hover.
pub fn scroll_style() -> egui::style::ScrollStyle {
    egui::style::ScrollStyle {
        floating: true,
        bar_width: 10.0,
        floating_width: 6.0,
        floating_allocated_width: 8.0,
        foreground_color: true,
        dormant_background_opacity: 0.0,
        active_background_opacity: 0.5,
        interact_background_opacity: 0.8,
        dormant_handle_opacity: SCROLL_THUMB_AT_REST,
        active_handle_opacity: 0.65,
        interact_handle_opacity: 0.85,
        ..egui::style::ScrollStyle::solid()
    }
}

/// How opaque a scroll bar's thumb is at rest (over the text colour).
const SCROLL_THUMB_AT_REST: f32 = 0.5;

/// Apply a theme. `roomy` = Beginner mode (bigger type, more air).
pub fn apply(ctx: &egui::Context, kind: ThemeKind, roomy: bool) {
    let t = Tokens::for_kind(kind);
    ctx.data_mut(|d| d.insert_temp(tokens_id(), t));
    ctx.set_theme(if t.dark() { egui::Theme::Dark } else { egui::Theme::Light });
    let mut v = if t.dark() { Visuals::dark() } else { Visuals::light() };
    v.panel_fill = t.chrome;
    v.window_fill = t.card;
    v.window_stroke = Stroke::new(1.0, t.card_border);
    v.extreme_bg_color = t.field;
    v.faint_bg_color = t.raised;
    v.code_bg_color = t.code_bg;
    v.override_text_color = Some(t.text);
    v.hyperlink_color = t.accent_ink;
    v.warn_fg_color = t.warning;
    v.error_fg_color = t.danger;
    v.window_corner_radius = t.r_lg();
    v.menu_corner_radius = t.r();
    v.window_shadow = egui::Shadow { offset: [0, 10], blur: 28, spread: 0, color: t.shadow };
    v.popup_shadow = egui::Shadow { offset: [0, 4], blur: 14, spread: 0, color: t.shadow };
    v.text_edit_bg_color = Some(t.field);
    v.selection.bg_fill = t.accent_soft;
    v.selection.stroke = Stroke::new(1.0, t.accent);
    v.handle_shape = egui::style::HandleShape::Circle;
    v.striped = false;
    v.indent_has_left_vline = false;
    let r = t.r_sm();
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = t.card;
    w.noninteractive.weak_bg_fill = t.card;
    w.noninteractive.bg_stroke = Stroke::new(1.0, t.separator);
    w.noninteractive.fg_stroke = Stroke::new(1.0, t.text_dim);
    w.noninteractive.corner_radius = r;
    // Buttons and combo boxes are hairline boxes on the surface (weak fill transparent at rest).
    for (wv, bg, weak, stroke) in [
        (&mut w.inactive, t.field, Color32::TRANSPARENT, t.field_border),
        (&mut w.hovered, t.hover, t.hover, t.text_faint),
        (&mut w.active, t.pressed, t.pressed, t.accent),
        (&mut w.open, t.hover, t.hover, t.field_border),
    ] {
        wv.bg_fill = bg;
        wv.weak_bg_fill = weak;
        wv.bg_stroke = Stroke::new(1.0, stroke);
        wv.fg_stroke = Stroke::new(1.0, t.text);
        wv.corner_radius = r;
        wv.expansion = 0.0;
    }
    ctx.set_visuals(v);
    // One body size per mode, from the scale: Simple mode reads at BODY, Advanced (a dense tool)
    // at SMALL; everything else steps from there, so `ui.label` and `theme::BODY` never disagree.
    let (body, small, mono_size, heading_size) = if roomy { (BODY, CAPTION, SMALL, SECTION) } else { (SMALL, TINY, CAPTION, CARD) };
    ctx.global_style_mut(|s| {
        s.text_styles = [
            (TextStyle::Small, FontId::proportional(small)),
            (TextStyle::Body, FontId::proportional(body)),
            (TextStyle::Button, FontId::proportional(body)),
            (TextStyle::Heading, semibold(heading_size)),
            (TextStyle::Monospace, FontId::monospace(mono_size)),
        ]
        .into();
        s.spacing.item_spacing = if roomy { egui::vec2(space::SM, space::S) } else { egui::vec2(space::SM, space::XS) };
        s.spacing.button_padding = if roomy { egui::vec2(space::M, space::XS) } else { egui::vec2(space::SM, space::XS) };
        s.spacing.interact_size = egui::vec2(size::CONTROL_SM, if roomy { size::CONTROL } else { size::CONTROL_SM });
        s.spacing.menu_margin = egui::Margin::same(6);
        s.spacing.window_margin = egui::Margin::same(18);
        s.spacing.icon_width = 14.0;
        s.spacing.scroll = scroll_style();
        s.spacing.tooltip_width = 320.0;
        s.interaction.tooltip_delay = 0.3;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The kit (`theme.rs` and `widgets.rs`) is the only place a colour, a line, a corner, a
    /// padding, a gap or a frame is made. Screens use its tokens ([`space`], [`pad`], [`line`],
    /// [`Tokens`]) and its frames (`widgets::card`, `pane`, `callout`…), so a redesign happens
    /// in one place. Tests inside a screen's file are not scanned.
    #[test]
    fn no_style_literals_outside_the_kit() {
        const KIT: [&str; 2] = ["theme.rs", "widgets.rs"];
        fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    rust_files(&path, out);
                } else if path.extension().is_some_and(|x| x == "rs") {
                    out.push(path);
                }
            }
        }
        let mut files = Vec::new();
        rust_files(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut files);
        assert!(files.len() > 10, "the source scan found only {} files", files.len());
        let mut found = Vec::new();
        for path in files {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if KIT.contains(&name) {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            found.extend(style_literals(&text).into_iter().map(|(n, code)| format!("{name}:{n}: {code}")));
        }
        assert!(found.is_empty(), "{} style literals outside the kit; use theme/widgets tokens:\n{}", found.len(), found.join("\n"));
    }

    /// What the kit scan refuses, and what it lets through.
    #[test]
    fn the_kit_scan_catches_style_literals_and_spares_their_names() {
        let refused = [
            "let c = Color32::from_rgb(1, 2, 3);",
            "let c = egui::Color32::WHITE;",
            "ui.painter().line(p, Stroke::new(1.0, c));",
            "let r = CornerRadius::same(4);",
            "let m = Margin::same(8);",
            "let m = egui::Margin::symmetric(8, 4);",
            "let m = Margin { left: 4, right: 4, top: 0, bottom: 0 };",
            "let m = egui::Margin { top: 2, ..Default::default() };",
            "let m = Margin {\n    left: 4,\n    right: 4,\n    top: 0,\n    bottom: 0,\n};",
            "let m = Margin\n{ left: 4, right: 4, top: 0, bottom: 0 };",
            "egui::Frame::new().fill(c)",
            "ui.add_space(12.0);",
            "ui.add_space(if dense { 4.0 } else { 6.0 });",
            "ui.spacing_mut().item_spacing.x = 10.0;",
            "egui::Grid::new(\"g\").spacing([12.0, 6.0])",
            "icons::show(ui, \"info\", 14.0, t.text_dim);",
            "icons::paint(ui, rect, \"check\", 18.0, t.accent);",
            "icons::button(ui, \"x\", 24.0, false, \"Close\")",
            "icons::paint(\n    ui,\n    rect,\n    icon,\n    16.0,\n    t.icon,\n);",
            "icons::show(ui, \"cpu\", 14.0 * s, t.icon);",
            "let (r, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());",
            "ui.label(RichText::new(t).size(12.5));",
            "RichText::new(n).font(theme::mono(12.0))",
            "ui.label(theme::label_text(\"OMCA\", 11.0));",
            "let g = widgets::label_galley(ui, &n, theme::TINY - 1.5, t.text);",
            "theme::heading(\n    title,\n    20.0,\n)",
            "widgets::primary_sized(ui, None, \"Start\", true, size::CONTROL_XL, 14.0)",
        ];
        for code in refused {
            assert!(!style_literals(code).is_empty(), "missed: {code}");
        }
        let spared = [
            "fn margin(dense: bool) -> Margin {\n    theme::pad(space::M)\n}",
            "pub fn pane_margin() -> egui::Margin {",
            "fn edges() ->\n    Margin {\n    theme::pad(space::S)\n}",
            "let m = egui::Margin::ZERO;",
            "let c = Color32::TRANSPARENT;",
            "ui.add_space(space::M);",
            "ui.add_space(((w - 240.0) / 2.0).max(0.0));",
            "ui.spacing_mut().item_spacing.x = 0.0;",
            "let gaps = ui.spacing().item_spacing.x * 4.0;",
            "// let m = Margin::same(8);",
            "struct PaneMargin { left: f32 }",
            "#[cfg(test)]\nmod tests { fn t() { let m = Margin::same(8); } }",
            "icons::show(ui, \"info\", size::ICON_SM, t.text_dim);",
            "icons::paint(ui, rect, \"check\", size::ICON * s, t.accent);",
            "icons::button(ui, \"x\", size::CONTROL_SM, false, \"Close (24 px)\")",
            "let (r, _) = ui.allocate_exact_size(Vec2::splat(size::TILE), Sense::hover());",
            "ui.label(RichText::new(t).size(theme::CAPTION));",
            "RichText::new(n).font(theme::mono(theme::SMALL * s))",
            "ui.label(theme::label_text(\"v1.2.0\", theme::TINY));",
            "widgets::primary_sized(ui, None, \"Start\", true, size::CONTROL_XL, theme::BODY)",
        ];
        for code in spared {
            assert!(style_literals(code).is_empty(), "flagged: {code}: {:?}", style_literals(code));
        }
        // A multi-line literal is reported at the line it starts on.
        assert_eq!(style_literals("fn f() {\n    let m = Margin {\n        left: 1,\n    };\n}").first().map(|(n, _)| *n), Some(2));
    }

    /// Style values made outside the kit in a screen's source, as (line number, code). Lines after
    /// `#[cfg(test)]` and comment lines are skipped.
    fn style_literals(text: &str) -> Vec<(usize, String)> {
        const MADE_HERE_ONLY: [&str; 17] = [
            "Color32::from_rgb",
            "Color32::from_gray",
            "Color32::from_black_alpha",
            "Color32::from_white_alpha",
            "Color32::WHITE",
            "Color32::BLACK",
            "Color32::RED",
            "Color32::GREEN",
            "Color32::BLUE",
            "Color32::YELLOW",
            "Color32::GRAY",
            "Stroke::new(",
            "CornerRadius::",
            "Margin::same(",
            "Margin::symmetric(",
            "Frame::new(",
            "Frame::default(",
        ];
        // The code lines, with their numbers, and the same lines joined so a literal that spans
        // lines can be found; `starts` maps a byte offset in `joined` back to its line.
        let mut lines = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("#[cfg(test)]") {
                break;
            }
            if !code.starts_with("//") {
                lines.push((n + 1, code));
            }
        }
        let mut joined = String::new();
        let mut starts = Vec::new();
        for (n, code) in &lines {
            starts.push((joined.len(), *n, *code));
            joined.push_str(code);
            joined.push('\n');
        }
        let mut found = std::collections::BTreeMap::new();
        for (n, code) in &lines {
            let numeric_gap = code.match_indices("add_space(").any(|(at, m)| code[at + m.len()..].starts_with(|c: char| c.is_ascii_digit()));
            // A gap chosen by a condition or set as item or grid spacing is a gap too; `0.0`
            // ("no gap") and centring arithmetic (`(width - w) / 2.0`) are not.
            let sets_item_spacing = ["item_spacing =", "item_spacing.x =", "item_spacing.y ="].iter().any(|m| code.contains(m));
            let gap_setting = code.contains("add_space(if ") || sets_item_spacing || code.contains(".spacing([");
            let gap_literal = gap_setting && floats_outside_strings(code).iter().any(|v| *v != 0.0);
            if numeric_gap || gap_literal || MADE_HERE_ONLY.iter().any(|m| code.contains(m)) {
                found.insert(*n, (*code).to_string());
            }
        }
        // A `Margin { … }` struct literal, whatever its first field and however it's wrapped;
        // `-> Margin {` is a function returning one, not a literal.
        for (at, m) in joined.match_indices("Margin") {
            let before = &joined[..at];
            if before.ends_with(|c: char| c.is_alphanumeric() || c == '_') || !joined[at + m.len()..].trim_start().starts_with('{') {
                continue;
            }
            if before.strip_suffix("egui::").unwrap_or(before).trim_end().ends_with("->") {
                continue;
            }
            if let Some((_, n, code)) = starts.iter().rev().find(|(start, _, _)| *start <= at) {
                found.entry(*n).or_insert_with(|| (*code).to_string());
            }
        }
        // Sizes: an icon's size, an icon button's box and a square box come from `size::*`
        // (a number scaled by the title bar's zoom is still a number).
        const SIZED: [(&str, usize); 4] = [("icons::show(", 2), ("icons::paint(", 3), ("icons::button(", 2), ("Vec2::splat(", 0)];
        for (marker, arg) in SIZED {
            for (at, _) in joined.match_indices(marker) {
                let open = at + marker.len() - 1;
                let Some(call) = call_text(&joined, open) else { continue };
                let numeric = top_level_parts(call).get(arg).is_some_and(|a| a.trim().starts_with(|c: char| c.is_ascii_digit()));
                if numeric && let Some((_, n, code)) = starts.iter().rev().find(|(start, _, _)| *start <= at) {
                    found.entry(*n).or_insert_with(|| (*code).to_string());
                }
            }
        }
        // Text sizes: every size argument of a text function is a name from the type scale (a
        // number in it, even `CAPTION - 1.5`, is a size the scale doesn't have).
        const TEXT_SIZED: [(&str, usize); 14] = [
            (".size(", 0),
            ("mono(", 0),
            ("medium(", 0),
            ("semibold(", 0),
            ("FontId::proportional(", 0),
            ("FontId::monospace(", 0),
            ("label_text(", 1),
            ("label_galley(", 2),
            ("heading(", 1),
            ("heading_galley(", 2),
            ("primary_sized(", 5),
            ("segmented_sized(", 4),
            ("segmented_scaled(", 4),
            ("pill_sized(", 4),
        ];
        for (marker, arg) in TEXT_SIZED {
            for (at, _) in joined.match_indices(marker) {
                let open = at + marker.len() - 1;
                let Some(call) = call_text(&joined, open) else { continue };
                let numeric = top_level_parts(call).get(arg).is_some_and(|a| !floats_outside_strings(a).is_empty());
                if numeric && let Some((_, n, code)) = starts.iter().rev().find(|(start, _, _)| *start <= at) {
                    found.entry(*n).or_insert_with(|| (*code).to_string());
                }
            }
        }
        found.into_iter().collect()
    }

    /// The text between `open`'s parenthesis and its match, or `None` when it isn't closed on
    /// this line.
    fn call_text(line: &str, open: usize) -> Option<&str> {
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escaped = false;
        for (i, c) in line.get(open..)?.char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }
            match c {
                '"' => in_string = true,
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        return line.get(open + 1..open + i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Split at commas that aren't inside brackets, braces or a string.
    fn top_level_parts(s: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut start = 0;
        let mut depth = 0i32;
        let mut in_string = false;
        let mut escaped = false;
        for (i, c) in s.char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }
            match c {
                '"' => in_string = true,
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    out.push(&s[start..i]);
                    start = i + 1;
                }
                _ => {}
            }
        }
        out.push(&s[start..]);
        out
    }

    /// Float literals on a line, ignoring digits inside string literals (a version or a count).
    fn floats_outside_strings(line: &str) -> Vec<f64> {
        let mut out = Vec::new();
        let mut in_string = false;
        let mut escaped = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_string = false;
                }
                continue;
            }
            if c == '"' {
                in_string = true;
                continue;
            }
            if c.is_ascii_digit() {
                let mut number = String::from(c);
                while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                    if let Some(d) = chars.next() {
                        number.push(d);
                    }
                }
                if chars.peek() == Some(&'.') {
                    number.push('.');
                    chars.next();
                    while chars.peek().is_some_and(|c| c.is_ascii_digit()) {
                        if let Some(d) = chars.next() {
                            number.push(d);
                        }
                    }
                    if let Ok(v) = number.parse::<f64>() {
                        out.push(v);
                    }
                }
            }
        }
        out
    }

    /// WCAG relative luminance of an opaque colour.
    fn luminance(c: Color32) -> f64 {
        let ch = |v: u8| {
            let s = f64::from(v) / 255.0;
            if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
    }

    /// A Japanese sans comes before a serif, so interface text is not set like a book page.
    #[test]
    fn fallback_fonts_are_sans_first() {
        let names = |list: &[&str]| list.iter().map(|f| format!("craft:{f}:Regular")).collect::<Vec<_>>();
        let ordered = sans_first(names(&["Shippori Mincho", "Noto Sans CJK SC", "BIZ UDPGothic", "NanumGothic", "Unknown Face"]));
        assert_eq!(ordered[0], "craft:BIZ UDPGothic:Regular");
        assert_eq!(ordered[1], "craft:Noto Sans CJK SC:Regular");
        assert_eq!(ordered[2], "craft:NanumGothic:Regular");
        assert_eq!(ordered[3], "craft:Unknown Face:Regular");
        assert_eq!(ordered[4], "craft:Shippori Mincho:Regular");
        assert_eq!(fallback_rank("craft:Unknown Face:Regular"), FallbackRank::Other);
        assert_eq!(fallback_rank("craft:BIZ UDMincho:Regular"), FallbackRank::Serif);
        assert_eq!(fallback_rank("craft:BIZ UDPGothic:Regular"), FallbackRank::Local);
    }

    /// Releases are built with craft-fonts' Korean and symbol fonts (`packaging/env.sh` keeps only
    /// those: OMCA's interface is Korean and English); a build without them shows Hangul as boxes.
    /// Latin, Cyrillic, Greek and the arrows come from the bundled faces.
    #[test]
    fn with_craft_fonts_the_scripts_omca_ships_show() {
        if CRAFT_FONTS.is_empty() {
            return;
        }
        let ctx = egui::Context::default();
        install_fonts(&ctx);
        ctx.run_ui(egui::RawInput::default(), |_| {}).textures_delta.clear();
        let texts = [("Korean", "한국어 사용자"), ("Russian", "Привет"), ("Greek", "Γειά σου"), ("arrows", "Settings → AI ← ↑ ↓")];
        for (script, text) in texts {
            for font in [FontId::proportional(14.0), FontId::monospace(14.0), medium(14.0), semibold(14.0)] {
                assert!(ctx.fonts_mut(|f| f.has_glyphs(&font, text)), "{script} ({text}) shows as boxes in {font:?}");
            }
        }
    }

    /// #1808: a heading's family takes a script's own bold face before its regular one, so
    /// Japanese or Chinese headings aren't lighter than the Latin beside them.
    #[test]
    fn a_heading_takes_a_scripts_bold_face_before_its_regular_one() {
        let base = vec!["Pretendard".to_string(), "craft:japanese:regular".to_string()];
        let bold = vec!["craft:japanese:bold".to_string()];
        let families = weighted_families(&base, &bold);
        let stack = |name: &str| {
            families.iter().find(|(f, _)| f == &FontFamily::Name(name.into())).map(|(_, s)| s.clone()).unwrap_or_else(|| panic!("{name} is installed"))
        };
        assert_eq!(stack("semibold"), ["Pretendard-SemiBold", "craft:japanese:bold", "Pretendard", "craft:japanese:regular"]);
        assert_eq!(stack("medium"), ["Pretendard-Medium", "craft:japanese:bold", "Pretendard", "craft:japanese:regular"]);
        // Without a bold face the stack is what it always was.
        let regular_only = weighted_families(&base, &[]);
        assert_eq!(regular_only[1].1, ["Pretendard-SemiBold", "Pretendard", "craft:japanese:regular"]);
    }

    #[test]
    fn a_scroll_thumb_at_rest_stands_out_from_every_surface_in_every_theme() {
        let style = scroll_style();
        assert!(style.floating_width >= 6.0 && style.foreground_color, "visible and wide enough to grab at rest");
        for k in ThemeKind::ALL {
            let t = Tokens::for_kind(k);
            for (name, surface) in [("chrome", t.chrome), ("canvas", t.canvas), ("card", t.card)] {
                // The thumb is the text colour at that opacity, over the surface.
                let a = f64::from(style.dormant_handle_opacity);
                let mix = |f: u8, b: u8| (a * f64::from(f) + (1.0 - a) * f64::from(b)).round() as u8;
                let thumb = Color32::from_rgb(mix(t.text.r(), surface.r()), mix(t.text.g(), surface.g()), mix(t.text.b(), surface.b()));
                let (l1, l2) = (luminance(thumb), luminance(surface));
                let ratio = (l1.max(l2) + 0.05) / (l1.min(l2) + 0.05);
                assert!(ratio >= 3.0, "{k:?} {name}: the thumb at rest is only {ratio:.2}:1");
            }
        }
    }

    #[test]
    fn theme_ids_round_trip() {
        for k in ThemeKind::ALL {
            assert_eq!(ThemeKind::from_id(k.id()), k);
        }
        assert_eq!(ThemeKind::from_id("nonsense"), ThemeKind::Studio);
    }

    /// WCAG 2.x contrast ratio between two opaque colours.
    fn contrast(a: Color32, b: Color32) -> f32 {
        let lum = |c: Color32| {
            let ch = |v: u8| {
                let s = v as f32 / 255.0;
                if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
            };
            0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
        };
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn contrast_meets_wcag_aa_in_both_themes() {
        for k in ThemeKind::ALL {
            let t = Tokens::for_kind(k);
            // `success` is text too: the Clean room and Ready pills.
            for (name, fg) in [("text", t.text), ("text_dim", t.text_dim), ("text_faint", t.text_faint), ("accent_ink", t.accent_ink), ("success", t.success)] {
                for bg in [t.canvas, t.card, t.chrome, t.raised] {
                    assert!(contrast(fg, bg) >= 4.5, "{k:?} {name} on {bg:?}: {:.2}", contrast(fg, bg));
                }
            }
            assert!(contrast(t.on_accent, t.accent) >= 4.5, "{k:?} text on the app colour: {:.2}", contrast(t.on_accent, t.accent));
            // Send's icon on its hover colour (an icon, so 3:1).
            assert!(contrast(t.on_accent, t.accent_hover) >= 3.0, "{k:?} icon on the hover colour: {:.2}", contrast(t.on_accent, t.accent_hover));
            assert!(contrast(t.field_border, t.canvas) >= 3.0, "{k:?} control borders: {:.2}", contrast(t.field_border, t.canvas));
            // Main buttons are filled in the app colour, a step lighter on hover; the label stays readable on both.
            assert!(contrast(t.on_accent, t.accent_hover) >= 4.5, "{k:?} main buttons on hover: {:.2}", contrast(t.on_accent, t.accent_hover));
            assert_eq!(t.accent, BRAND);
        }
    }

    #[test]
    fn confidence_has_four_distinct_colours() {
        let cs = [
            confidence_color(Some(Confidence::Observed)),
            confidence_color(Some(Confidence::Derived)),
            confidence_color(Some(Confidence::Inferred)),
            confidence_color(None),
        ];
        for (i, a) in cs.iter().enumerate() {
            for b in cs.iter().skip(i + 1) {
                assert_ne!(a, b);
            }
        }
        assert_eq!(confidence_label(None), "Unknown");
    }
}
