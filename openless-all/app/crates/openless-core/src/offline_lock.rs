//! M1 offline Cantonese lock (see repo-root SPEC.md §5).
//!
//! This fork ships as a fully offline Hong Kong Cantonese dictation app:
//! - the only ASR engine is the local sherpa-onnx SenseVoice model;
//! - the recognition language is locked to `yue` (no auto detection), with the
//!   manual zh / en switch kept;
//! - LLM polish is off so colloquial Cantonese (呢 / 唔 / 嘅) is never
//!   rewritten into written Mandarin.
//!
//! Cloud provider code is intentionally kept (M1 = disable + hide, not delete);
//! this module is the single place that pins every session to the offline path.

use crate::shared_types::{PipelineMode, UserPreferences};
use crate::types::PolishMode;

/// Provider id of the local sherpa-onnx runtime.
pub const OFFLINE_LOCK_ASR_PROVIDER_ID: &str = "sherpa-onnx-local";
/// `csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17`.
pub const OFFLINE_LOCK_ASR_MODEL: &str = crate::local_asr_catalog::SHERPA_DEFAULT_MODEL_ALIAS;
/// Default recognition language: Cantonese.
pub const OFFLINE_LOCK_DEFAULT_LANGUAGE: &str = "yue";
/// Languages the user may switch to manually (SPEC §5.2.3: 中 / EN / Yue).
pub const OFFLINE_LOCK_LANGUAGES: &[&str] = &["yue", "zh", "en"];

/// Map any hint outside the allowed set (including empty / `auto`) to `yue`.
pub fn lock_language_hint(value: &str) -> String {
    let normalized = value.trim().to_lowercase();
    if OFFLINE_LOCK_LANGUAGES.contains(&normalized.as_str()) {
        normalized
    } else {
        OFFLINE_LOCK_DEFAULT_LANGUAGE.to_string()
    }
}

/// Whether the lock is applied by the preference store, defaults and session capture.
///
/// Controlled by the `offline-lock` cargo feature, which the desktop app enables. Without
/// it (this crate's own test suites, other hosts) the upstream cloud / polish code paths
/// that M1 hides but does not delete keep their original behaviour.
/// `cargo test -p openless-core --features offline-lock --test offline_lock` verifies it.
pub const ENFORCED: bool = cfg!(feature = "offline-lock");

/// Apply [`apply_offline_lock`] when [`ENFORCED`].
pub fn enforce(preferences: &mut UserPreferences) {
    if ENFORCED {
        apply_offline_lock(preferences);
    }
}

/// Colloquial Cantonese characters that OpenCC `s2hk` would rewrite into a different
/// written-Chinese word: 晒 (all / completely, 唔該晒) → 曬 (sun-dry), 搵 (find) → 揾,
/// 吓 (試吓 / 吓?) → 嚇 (scare). They are kept exactly as recognised.
const CANTONESE_PROTECTED: &[char] = &['晒', '搵', '吓'];

/// Cantonese copula 係 ("is") phrases that `s2hk` leaves as the Simplified 系, because
/// OpenCC only knows 係 inside written-Chinese words (關係, 就係). Applied after conversion.
/// Bare 系 is not touched: 系統 / 系列 / 星系 / 體系 must keep it.
const CANTONESE_COPULA_PHRASES: &[(&str, &str)] = &[
    ("系唔系", "係唔係"),
    ("系咪", "係咪"),
    ("唔系", "唔係"),
    ("真系", "真係"),
    ("梗系", "梗係"),
    ("即系", "即係"),
    ("都系", "都係"),
    ("咪系", "咪係"),
    ("又系", "又係"),
    ("佢系", "佢係"),
    ("佢哋系", "佢哋係"),
    ("我哋系", "我哋係"),
    ("你哋系", "你哋係"),
    ("邊個系", "邊個係"),
    ("系咁", "係咁"),
    ("系呀", "係呀"),
    ("系啊", "係啊"),
    ("系喇", "係喇"),
    ("系啦", "係啦"),
    ("系囉", "係囉"),
    ("系咩", "係咩"),
];

/// Convert ASR output to Hong Kong Traditional Chinese (OpenCC `s2hk`).
///
/// SenseVoice is trained mostly on mainland data and emits Simplified forms for common
/// words (其实 / 觉得 / 现在 / 讲). Colloquial Cantonese characters (呢 / 唔 / 嘅 / 咁 / 係 /
/// 冇) are already Traditional and pass through unchanged; [`CANTONESE_PROTECTED`] covers
/// the few that `s2hk` would otherwise change. If the converter cannot be built the text
/// is returned as is.
pub fn to_hong_kong_traditional(text: &str) -> String {
    use ferrous_opencc::config::BuiltinConfig;
    use std::sync::OnceLock;

    static S2HK: OnceLock<Option<ferrous_opencc::OpenCC>> = OnceLock::new();
    match S2HK.get_or_init(|| {
        ferrous_opencc::OpenCC::from_config(BuiltinConfig::S2hk)
            .map_err(|error| log::warn!("[offline-lock] OpenCC s2hk unavailable: {error}"))
            .ok()
    }) {
        Some(converter) if !text.is_empty() => {
            let mut output = String::with_capacity(text.len());
            let mut segment = String::new();
            for character in text.chars() {
                if CANTONESE_PROTECTED.contains(&character) {
                    output.push_str(&converter.convert(&segment));
                    segment.clear();
                    output.push(character);
                } else {
                    segment.push(character);
                }
            }
            output.push_str(&converter.convert(&segment));
            for (simplified, cantonese) in CANTONESE_COPULA_PHRASES {
                if output.contains(simplified) {
                    output = output.replace(simplified, cantonese);
                }
            }
            output
        }
        _ => text.to_string(),
    }
}

/// Pin preferences to the offline Cantonese configuration.
pub fn apply_offline_lock(preferences: &mut UserPreferences) {
    preferences.active_asr_provider = OFFLINE_LOCK_ASR_PROVIDER_ID.to_string();
    preferences.sherpa_onnx_model = OFFLINE_LOCK_ASR_MODEL.to_string();
    preferences.sherpa_onnx_language_hint =
        lock_language_hint(&preferences.sherpa_onnx_language_hint);
    // Multimodal (Omni) and translation are cloud LLM paths.
    preferences.multimodal_pipeline_enabled = false;
    preferences.pipeline_mode = PipelineMode::Traditional;
    preferences.translation_target_language.clear();
    // Built-in Raw style = true passthrough, no LLM request.
    preferences.default_mode = PolishMode::Raw;
    preferences.active_style_pack_id = crate::style_packs::BUILTIN_STYLE_PACK_RAW_ID.to_string();
    // M2: cloud-only features are hidden in the UI; also make sure their global hotkeys and
    // switches can't trigger an LLM / agent session. Translation is already inert because
    // its target language is cleared above. Local features (dictation, quick note, cursor
    // context, correction learning) are untouched.
    preferences.qa_hotkey = None;
    preferences.selection_polish_hotkey = None;
    preferences.selection_voice_enabled = false;
    preferences.voice_edit_enabled = false;
    preferences.switch_style_hotkey = None;
    preferences.style_pack_hotkeys.clear();
    preferences.coding_agent_enabled = false;
    preferences.coding_agent_voice_hotkey = None;
    preferences.coding_agent_panel_hotkey = None;
    preferences.coding_agent_quick_hotkey = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hong_kong_traditional_converts_simplified_and_keeps_colloquial_cantonese() {
        assert_eq!(
            to_hong_kong_traditional("其实我觉得现在讲"),
            "其實我覺得現在講"
        );
        for colloquial in [
            "我今日唔去觀塘喇",
            "呢個係咪你嘅？",
            "咁就係咁先",
            "佢冇嚟，我哋走啦",
            "唔該晒，啲嘢喺邊度？",
            "食晒啲飯先，試吓搵佢",
        ] {
            assert_eq!(to_hong_kong_traditional(colloquial), colloquial);
        }
        // SenseVoice yue output observed on macOS (macOS "Sinji" voice → sherpa-onnx).
        assert_eq!(
            to_hong_kong_traditional("呢个系咪你嘅，咁，就系咁先。"),
            "呢個係咪你嘅，咁，就係咁先。"
        );
        assert_eq!(
            to_hong_kong_traditional("唔该晒，我今日唔去观塘啦。"),
            "唔該晒，我今日唔去觀塘啦。"
        );
        assert_eq!(
            to_hong_kong_traditional("佢系唔系真系咁讲"),
            "佢係唔係真係咁講"
        );
        // Written-Chinese 系 words are untouched.
        assert_eq!(to_hong_kong_traditional("系统同系列"), "系統同系列");
        // Real conversions around a protected character still happen.
        assert_eq!(to_hong_kong_traditional("现在食晒几多"), "現在食晒幾多");
        assert_eq!(to_hong_kong_traditional(""), "");
        assert_eq!(
            to_hong_kong_traditional("OK, 3點半 meeting"),
            "OK, 3點半 meeting"
        );
    }

    #[test]
    fn language_hint_defaults_to_yue_and_keeps_manual_switch() {
        assert_eq!(lock_language_hint(""), "yue");
        assert_eq!(lock_language_hint("auto"), "yue");
        assert_eq!(lock_language_hint("ja"), "yue");
        assert_eq!(lock_language_hint(" ZH "), "zh");
        assert_eq!(lock_language_hint("en"), "en");
        assert_eq!(lock_language_hint("yue"), "yue");
    }

    #[test]
    fn locked_defaults_are_offline_cantonese() {
        let mut preferences = UserPreferences::default();
        apply_offline_lock(&mut preferences);
        assert_eq!(
            preferences.active_asr_provider,
            OFFLINE_LOCK_ASR_PROVIDER_ID
        );
        assert_eq!(preferences.sherpa_onnx_model, OFFLINE_LOCK_ASR_MODEL);
        assert_eq!(preferences.sherpa_onnx_language_hint, "yue");
        assert_eq!(preferences.default_mode, PolishMode::Raw);
        assert_eq!(
            preferences.active_style_pack_id,
            crate::style_packs::BUILTIN_STYLE_PACK_RAW_ID
        );
        assert!(!preferences.multimodal_pipeline_enabled);
    }

    #[test]
    fn lock_overrides_cloud_preferences() {
        let mut preferences = UserPreferences {
            active_asr_provider: "volcengine".into(),
            sherpa_onnx_language_hint: String::new(),
            default_mode: PolishMode::Structured,
            multimodal_pipeline_enabled: true,
            translation_target_language: "English".into(),
            ..UserPreferences::default()
        };
        apply_offline_lock(&mut preferences);
        assert_eq!(
            preferences.active_asr_provider,
            OFFLINE_LOCK_ASR_PROVIDER_ID
        );
        assert_eq!(preferences.sherpa_onnx_language_hint, "yue");
        assert_eq!(preferences.default_mode, PolishMode::Raw);
        assert!(!preferences.multimodal_pipeline_enabled);
        assert!(preferences.translation_target_language.is_empty());
        assert!(preferences.qa_hotkey.is_none());
        assert!(preferences.switch_style_hotkey.is_none());
        assert!(!preferences.coding_agent_enabled);
        assert!(preferences.coding_agent_voice_hotkey.is_none());
    }
}
