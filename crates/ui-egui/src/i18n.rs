//! UI localisation: English strings in code are the lookup keys; a per-language
//! TSV catalog (`key<TAB>translation` per line, `#` comments) maps them to display text at render
//! time. A string with no translation is shown in English. Tool ids, the CLI and MCP never see
//! translated text.
//!
//! Supported: English (built in) and Korean (`i18n/ko.tsv`). Adding a language: put `xx.tsv` in
//! `i18n/`, add one row to [`LANGUAGES`] and its code to `xtask/src/i18n_coverage.rs`.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// One supported UI language.
pub struct LangInfo {
    /// BCP 47 code, lowercase. Also the `settings.language` value.
    pub code: &'static str,
    /// The language's name in itself.
    pub name: &'static str,
    /// Catalog contents (empty for the built-in English).
    pub source: &'static str,
}

pub static LANGUAGES: &[LangInfo] =
    &[LangInfo { code: "en", name: "English", source: "" }, LangInfo { code: "ko", name: "한국어", source: include_str!("i18n/ko.tsv") }];

/// Match a locale (`ko-KR`, `ko_KR.UTF-8`, `en-US`…) to a registered UI language.
pub fn language_from_tag(tag: &str) -> Option<&'static str> {
    let tag = tag.trim().split(['.', '@']).next().unwrap_or_default().replace('_', "-").to_ascii_lowercase();
    let primary = tag.split('-').next()?;
    LANGUAGES.iter().find(|l| l.code == primary).map(|l| l.code)
}

fn preferred_language(tags: impl IntoIterator<Item = impl AsRef<str>>) -> &'static str {
    tags.into_iter().find_map(|tag| language_from_tag(tag.as_ref())).unwrap_or("en")
}

fn system_language() -> &'static str {
    #[cfg(not(target_arch = "wasm32"))]
    {
        static SYSTEM: OnceLock<&'static str> = OnceLock::new();
        SYSTEM.get_or_init(|| preferred_language(sys_locale::get_locales()))
    }
    #[cfg(target_arch = "wasm32")]
    {
        preferred_language(std::iter::empty::<&str>())
    }
}

fn current() -> &'static RwLock<(String, HashMap<String, String>)> {
    static CUR: OnceLock<RwLock<(String, HashMap<String, String>)>> = OnceLock::new();
    CUR.get_or_init(|| RwLock::new(("en".into(), HashMap::new())))
}

fn parse(source: &str) -> HashMap<String, String> {
    source
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .filter_map(|l| l.split_once('\t'))
        .filter(|(k, v)| !k.is_empty() && !v.trim().is_empty())
        .map(|(k, v)| (k.replace("\\n", "\n"), v.trim_end().replace("\\n", "\n")))
        .collect()
}

/// Switch language. Empty means the system's preferred language; unsupported codes mean English.
pub fn set_language(code: &str) {
    let code = if code.trim().is_empty() { system_language() } else { language_from_tag(code).unwrap_or("en") };
    let lang = LANGUAGES.iter().find(|l| l.code == code);
    let map = lang.map(|l| parse(l.source)).unwrap_or_default();
    if let Ok(mut g) = current().write() {
        *g = (lang.map(|l| l.code).unwrap_or("en").to_string(), map);
    }
}

pub fn language() -> String {
    current().read().map(|g| g.0.clone()).unwrap_or_else(|_| "en".into())
}

/// Translate a UI string.
pub fn tr(s: &str) -> String {
    match current().read() {
        Ok(g) => g.1.get(s).cloned().unwrap_or_else(|| s.to_string()),
        Err(_) => s.to_string(),
    }
}

/// Translate, then fill `{name}` placeholders.
pub fn trf(s: &str, args: &[(&str, &str)]) -> String {
    let mut out = tr(s);
    for (k, v) in args {
        out = out.replace(&format!("{{{k}}}"), v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_parse_and_fallback() {
        let m = parse("# comment\nHello\tHola\nEmpty\t \nbad line\n");
        assert_eq!(m.get("Hello").map(String::as_str), Some("Hola"));
        assert!(!m.contains_key("Empty"));
        assert_eq!(trf("Open {name}", &[("name", "a.exe")]), "Open a.exe");
        assert_eq!(tr("Untranslated"), "Untranslated");
    }

    /// A locale resolves to its language from any region or encoding; anything else is English.
    #[test]
    fn locales_resolve_to_korean_or_english() {
        for tag in ["ko", "ko-KR", "ko_KR.UTF-8", "KO-kr"] {
            assert_eq!(language_from_tag(tag), Some("ko"), "{tag}");
        }
        assert_eq!(language_from_tag("en-US"), Some("en"));
        assert_eq!(language_from_tag("ja-JP"), None);
        assert_eq!(preferred_language(["ja-JP", "ko-KR", "en-US"]), "ko");
        assert_eq!(preferred_language(["en-US", "ko-KR"]), "en");
        assert_eq!(preferred_language(["zh-CN"]), "en");
        assert_eq!(preferred_language(std::iter::empty::<&str>()), "en");
    }

    /// The Korean catalogue parses cleanly: one key and one translation per line, no duplicates,
    /// and placeholders and trailing ellipses kept.
    #[test]
    fn korean_catalog_is_valid() {
        fn placeholders(s: &str) -> Vec<&str> {
            let mut out: Vec<_> = s.split('{').skip(1).filter_map(|s| s.split_once('}').map(|(p, _)| p)).collect();
            out.sort_unstable();
            out
        }
        let lang = LANGUAGES.iter().find(|l| l.code == "ko").unwrap();
        let mut keys = std::collections::BTreeSet::new();
        for (i, line) in lang.source.lines().enumerate() {
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line.split_once('\t').unwrap_or_else(|| panic!("ko:{}: missing tab", i + 1));
            assert!(!key.is_empty() && !value.trim().is_empty(), "ko:{}: empty column", i + 1);
            assert!(!value.contains('\t'), "ko:{}: extra column", i + 1);
            assert!(keys.insert(key.to_string()), "ko:{}: duplicate {key}", i + 1);
            assert_eq!(placeholders(key), placeholders(value), "ko: {key}");
            assert_eq!(key.ends_with('…'), value.ends_with('…'), "ko: {key}");
        }
        assert_eq!(parse(lang.source).len(), keys.len());
        assert!(keys.len() > 500, "ko: incomplete catalog");
    }

    /// Every translation keeps its key's `{placeholders}`, or `trf` would leave them unfilled.
    #[test]
    fn catalogs_keep_placeholders() {
        let holes = |s: &str| {
            let mut v: Vec<String> = s.split('{').skip(1).filter_map(|p| p.split_once('}')).map(|(h, _)| h.to_string()).collect();
            v.sort();
            v
        };
        for l in LANGUAGES {
            let m = parse(l.source);
            assert!(l.source.is_empty() || m.len() > 100, "{} catalog barely parsed", l.code);
            for (k, v) in &m {
                assert_eq!(holes(k), holes(v), "{}: {k:?}", l.code);
            }
        }
    }
}
