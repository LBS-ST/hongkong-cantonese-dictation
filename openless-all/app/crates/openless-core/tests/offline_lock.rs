//! M1 offline Cantonese lock, exercised against the real (non-`cfg(test)`) library.

use openless_core::shared_types::{PipelineMode, UserPreferences};
use openless_core::types::PolishMode;
use openless_core::PreferencesStore;

#[test]
fn lock_is_enforced_outside_unit_tests() {
    assert!(openless_core::offline_lock::ENFORCED);
}

#[test]
fn first_launch_defaults_need_no_cloud_credentials() {
    let preferences = UserPreferences::default();
    assert_eq!(preferences.active_asr_provider, "sherpa-onnx-local");
    assert_eq!(preferences.sherpa_onnx_model, "sense-voice-small-zh");
    assert_eq!(preferences.sherpa_onnx_language_hint, "yue");
    assert_eq!(preferences.default_mode, PolishMode::Raw);
    assert_eq!(preferences.active_style_pack_id, "builtin.raw");
    assert!(!preferences.multimodal_pipeline_enabled);
}

#[test]
fn stored_cloud_preferences_are_relocked_on_open_and_save() {
    let dir = std::env::temp_dir().join(format!(
        "openless-offline-lock-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("preferences.json");
    // Simulate preferences left behind by an upstream OpenLess install.
    std::fs::write(
        &path,
        serde_json::json!({
            "activeAsrProvider": "volcengine",
            "activeLlmProvider": "ark",
            "sherpaOnnxLanguageHint": "",
            "defaultMode": "structured",
            "multimodalPipelineEnabled": true,
            "pipelineMode": "multimodal",
            "translationTargetLanguage": "English",
        })
        .to_string(),
    )
    .unwrap();

    let store = PreferencesStore::open(&path).unwrap();
    let opened = store.get();
    assert_eq!(opened.active_asr_provider, "sherpa-onnx-local");
    assert_eq!(opened.sherpa_onnx_language_hint, "yue");
    assert_eq!(opened.default_mode, PolishMode::Raw);
    assert_eq!(opened.pipeline_mode, PipelineMode::Traditional);
    assert!(opened.translation_target_language.is_empty());

    // A save cannot switch back to a cloud engine or auto language detection,
    // but the manual zh / en switch is kept.
    let mut edited = opened;
    edited.active_asr_provider = "tencent-cloud".into();
    edited.sherpa_onnx_language_hint = "zh".into();
    store.set(edited).unwrap();
    let saved = store.get();
    assert_eq!(saved.active_asr_provider, "sherpa-onnx-local");
    assert_eq!(saved.sherpa_onnx_language_hint, "zh");

    let mut auto = saved;
    auto.sherpa_onnx_language_hint = "auto".into();
    store.set(auto).unwrap();
    assert_eq!(store.get().sherpa_onnx_language_hint, "yue");
    let _ = std::fs::remove_dir_all(&dir);
}
