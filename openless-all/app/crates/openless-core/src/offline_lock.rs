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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
