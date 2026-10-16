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
/// `csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2025-09-09`.
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

/// Written-Chinese words that really use 系 (system / series / lineage / department).
/// In Cantonese dictation every other 系 is the copula 係 ("is"): SenseVoice yue writes it
/// as the Simplified 系 and OpenCC `s2hk` only restores 係 inside words it knows (關係,
/// 就係), missing 係咪 / 唔係 / 定係 / 淨係 / 但係 / 佢係 ... The 繫 sense (聯繫, 繫上) is
/// already converted by OpenCC.
const WRITTEN_XI_WORDS: &[&str] = &[
    "系統",
    "系列",
    "系譜",
    "系主任",
    "體系",
    "派系",
    "星系",
    "銀河系",
    "太陽系",
    "直系",
    "旁系",
    "嫡系",
    "科系",
    "學系",
    "院系",
    "母系",
    "父系",
    "譜系",
    "世系",
    "菌系",
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
            output
        }
        _ => text.to_string(),
    }
}

/// Restore the Cantonese copula: every 系 becomes 係 except inside [`WRITTEN_XI_WORDS`].
pub fn restore_cantonese_copula(text: &str) -> String {
    if !text.contains('系') {
        return text.to_string();
    }
    let characters: Vec<char> = text.chars().collect();
    let mut keep = vec![false; characters.len()];
    for word in WRITTEN_XI_WORDS {
        let word: Vec<char> = word.chars().collect();
        if word.len() > characters.len() {
            continue;
        }
        for start in 0..=characters.len() - word.len() {
            if characters[start..start + word.len()] == word[..] {
                for (offset, character) in word.iter().enumerate() {
                    if *character == '系' {
                        keep[start + offset] = true;
                    }
                }
            }
        }
    }
    characters
        .iter()
        .zip(keep)
        .map(|(character, keep)| {
            if *character == '系' && !keep {
                '係'
            } else {
                *character
            }
        })
        .collect()
}

/// Text inserted by offline dictation: Hong Kong Traditional, plus the Cantonese copula
/// fix when recognising Cantonese (`yue`, the default). Mandarin (`zh`) keeps 系 as is.
pub fn localize_offline_transcript(text: &str, language: Option<&str>) -> String {
    let traditional = to_hong_kong_traditional(text);
    let cantonese = language
        .map(str::trim)
        .filter(|language| !language.is_empty())
        .is_none_or(|language| language.eq_ignore_ascii_case(OFFLINE_LOCK_DEFAULT_LANGUAGE));
    if cantonese {
        restore_cantonese_copula(&traditional)
    } else {
        traditional
    }
}

/// Colloquial Cantonese → written Chinese word table (M2 round 2 spec §2, 書面 mode).
/// Word substitution only — Cantonese word order is kept (我食先 → 我吃先).
const WRITTEN_CHINESE_WORDS: &[(&str, &str)] = &[
    // Phrases first (the longest match wins).
    ("而家", "現在"),
    ("宜家", "現在"),
    ("點解", "為什麼"),
    ("點樣", "怎樣"),
    ("咁樣", "這樣"),
    ("今日", "今天"),
    ("聽日", "明天"),
    ("尋日", "昨天"),
    ("琴日", "昨天"),
    ("邊度", "哪裏"),
    ("邊個", "誰"),
    ("乜嘢", "甚麼"),
    ("咩嘢", "甚麼"),
    ("唔係", "不是"),
    ("係咪", "是不是"),
    ("唔該晒", "非常感謝"),
    ("多謝晒", "非常感謝"),
    ("唔該", "謝謝"),
    ("我哋", "我們"),
    ("你哋", "你們"),
    ("佢哋", "他們"),
    ("呢個", "這個"),
    ("嗰個", "那個"),
    ("呢啲", "這些"),
    ("嗰啲", "那些"),
    ("呢度", "這裏"),
    ("嗰度", "那裏"),
    ("呢次", "這次"),
    ("呢排", "最近"),
    ("嗰陣", "那時"),
    ("一啲", "一點"),
    ("有啲", "有些"),
    ("好啲", "好一點"),
    ("試吓", "試一下"),
    ("即刻", "立刻"),
    // Single characters.
    ("唔", "不"),
    ("嘅", "的"),
    ("係", "是"),
    ("喺", "在"),
    ("嚟", "來"),
    ("返", "回"),
    ("畀", "給"),
    ("俾", "給"),
    ("攞", "拿"),
    ("睇", "看"),
    ("講", "說"),
    ("食", "吃"),
    ("飲", "喝"),
    ("瞓", "睡"),
    ("冇", "沒有"),
    ("咗", "了"),
    ("佢", "他"),
    ("啲", "些"),
    ("搵", "找"),
    ("嘢", "東西"),
    ("喇", "了"),
    ("咯", "了"),
    ("啦", "吧"),
];

/// Written words that contain a character from [`WRITTEN_CHINESE_WORDS`] and must stay.
const WRITTEN_CHINESE_KEEP: &[&str] = &[
    "關係", "係數", "食物", "食品", "食堂", "飲食", "糧食", "零食", "美食", "食譜", "食材", "食肆",
    "素食", "主食", "飲品", "飲料", "冷飲", "熱飲", "餐飲", "講座", "講師", "演講", "講解", "講究",
    "講義", "講台", "返回", "往返", "返還",
];

/// 書面 mode: colloquial Cantonese → written Chinese by word substitution. 咁 depends on
/// position: sentence-initial → 那麼, before punctuation / end → 這樣, otherwise 這麼.
pub fn to_written_chinese(text: &str) -> String {
    let words = crate::correction::replace_words(text, WRITTEN_CHINESE_WORDS, WRITTEN_CHINESE_KEEP);
    let characters: Vec<char> = words.chars().collect();
    let is_break = |character: Option<&char>| {
        character.is_none_or(|character| {
            character.is_whitespace() || "，。！？、；：,.!?;:…".contains(*character)
        })
    };
    let mut output = String::with_capacity(words.len());
    for (index, character) in characters.iter().enumerate() {
        if *character != '咁' {
            output.push(*character);
        } else if index == 0 || is_break(characters.get(index - 1)) {
            output.push_str("那麼");
        } else if is_break(characters.get(index + 1)) {
            output.push_str("這樣");
        } else {
            output.push_str("這麼");
        }
    }
    output
}

/// Built-in Hong Kong correction rules for fixed SenseVoice yue misrecognitions
/// (M2 round 2 spec §1). Seeded once into the user's correction-rule list, so they are
/// editable, can be disabled or deleted, and run alongside user rules — after the
/// HK Traditional conversion and the 係 copula fix.
pub fn hong_kong_correction_rules() -> Vec<crate::types::CorrectionRule> {
    let rule = |id: &str, pattern: &str, replacement: &str, exceptions: &[&str]| {
        crate::types::CorrectionRule {
            id: format!("builtin.hk.{id}"),
            pattern: pattern.into(),
            replacement: replacement.into(),
            enabled: true,
            created_at: String::new(),
            source: crate::types::RuleSource::Builtin,
            exceptions: exceptions.iter().map(|word| (*word).to_string()).collect(),
        }
    };
    vec![
        rule("jik-hak", "知家", "即刻", &[]),
        rule("si-haa", "試下", "試吓", &[]),
        rule("laa", "啦", "喇", &[]),
        // 搵 (find) heard as 穩; 穩 inside 穩定 / 穩陣 / 穩妥 / 穩步 / 穩健 ... is real.
        rule(
            "wan",
            "穩",
            "搵",
            &[
                "穩定",
                "穩陣",
                "穩妥",
                "穩步",
                "穩健",
                "穩固",
                "穩重",
                "穩當",
                "穩紮穩打",
                "安穩",
                "平穩",
                "企穩",
                "坐穩",
                "站穩",
            ],
        ),
    ]
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

    /// Full offline pipeline for one utterance: HK conversion + 係 fix → correction
    /// rules (built-in + user) → optional 書面 mode. Mirrors dictation_engine.
    fn pipeline(asr: &str, user_rules: &[crate::types::CorrectionRule], written: bool) -> String {
        let mut rules = hong_kong_correction_rules();
        rules.extend_from_slice(user_rules);
        let text = localize_offline_transcript(asr, Some("yue"));
        let text = crate::correction::apply_correction_rules(&text, &rules);
        if written {
            to_written_chinese(&text)
        } else {
            text
        }
    }

    #[test]
    fn spec_acceptance_hk_corrections_and_written_mode() {
        // §1: 「我而家就即刻試」 heard as 知家.
        assert_eq!(pipeline("我而家就知家试", &[], false), "我而家就即刻試");
        // §2 colloquial (default) and written.
        assert_eq!(pipeline("我今日唔去观塘啦", &[], false), "我今日唔去觀塘喇");
        assert_eq!(pipeline("我今日唔去观塘啦", &[], true), "我今天不去觀塘了");
        // Word substitution only: Cantonese word order stays (spec limitation).
        assert_eq!(pipeline("我食先", &[], true), "我吃先");
    }

    #[test]
    fn hong_kong_corrections_cover_spec_cases_and_respect_context() {
        assert_eq!(pipeline("你试下啦", &[], false), "你試吓喇");
        assert_eq!(pipeline("我去稳嘢", &[], false), "我去搵嘢");
        for kept in [
            "个系统好稳定",
            "好稳阵",
            "好稳妥",
            "稳步上升",
            "稳健",
            "好安稳",
        ] {
            let colloquial = pipeline(kept, &[], false);
            assert!(colloquial.contains('穩'), "{kept} → {colloquial}");
        }
        // User rules run alongside the built-ins, and a disabled built-in is skipped.
        let user = crate::types::CorrectionRule {
            id: "user".into(),
            pattern: "觀塘".into(),
            replacement: "觀塘區".into(),
            enabled: true,
            created_at: String::new(),
            source: crate::types::RuleSource::Manual,
            exceptions: Vec::new(),
        };
        assert_eq!(
            pipeline("我今日去观塘啦", &[user.clone()], false),
            "我今日去觀塘區喇"
        );
        let mut rules = hong_kong_correction_rules();
        rules
            .iter_mut()
            .for_each(|rule| rule.enabled = rule.pattern != "啦");
        let text = localize_offline_transcript("去啦", Some("yue"));
        assert_eq!(
            crate::correction::apply_correction_rules(&text, &rules),
            "去啦"
        );
    }

    #[test]
    fn written_mode_substitutes_words_and_keeps_written_vocabulary() {
        for (colloquial, written) in [
            ("點解你唔嚟", "為什麼你不來"),
            ("佢喺屋企睇電視", "他在屋企看電視"),
            ("我哋而家食飯，飲咗水", "我們現在吃飯，喝了水"),
            ("呢個係我嘅，冇問題", "這個是我的，沒有問題"),
            ("咁樣唔得", "這樣不得"),
            ("咁，我返屋企喇", "那麼，我回屋企了"),
            ("好咁好", "好這麼好"),
            ("就係咁", "就是這樣"),
            ("我哋嘅關係", "我們的關係"),
            ("食物同飲品", "食物同飲品"),
            ("佢講咗個講座", "他說了個講座"),
        ] {
            assert_eq!(to_written_chinese(colloquial), written, "{colloquial}");
        }
    }

    #[test]
    fn cantonese_copula_is_restored_in_live_dictation_output() {
        // Verbatim SenseVoice yue output from live dictation on macOS (asrTranscript).
        for (asr, inserted) in [
            ("而家系咪净系出繁体字今晚。", "而家係咪淨係出繁體字今晚。"),
            ("现在测试中应该，系净系出繁体字。", "現在測試中應該，係淨係出繁體字。"),
            (
                "我现在就是用这个程式同你沟通，你看看里边有没有我们想做的事是否达成系咪搞掂咗定系还是要执。",
                "我現在就是用這個程式同你溝通，你看看裏邊有沒有我們想做的事是否達成係咪搞掂咗定係還是要執。",
            ),
            (
                "你啱，啱讲话仲有三个残留，但系我睇翻，我自己 send 个 message 系啊繁体中文嚟嘅。",
                "你啱，啱講話仲有三個殘留，但係我睇翻，我自己 send 個 message 係啊繁體中文嚟嘅。",
            ),
            (
                "我想问多次，我哋依个模型系决定用依个噶啦嘛定定系还是会用其他。",
                "我想問多次，我哋依個模型係決定用依個噶啦嘛定定係還是會用其他。",
            ),
            ("梗系啦，即系咁，都系佢", "梗係啦，即係咁，都係佢"),
        ] {
            assert_eq!(localize_offline_transcript(asr, Some("yue")), inserted, "{asr}");
            assert_eq!(localize_offline_transcript(asr, None), inserted, "{asr} (default)");
        }
        // Written words keep 系; the 繫 sense is OpenCC's job.
        assert_eq!(
            localize_offline_transcript("呢个系统系列同太阳系", Some("yue")),
            "呢個系統系列同太陽系"
        );
        assert_eq!(localize_offline_transcript("联系佢", Some("yue")), "聯繫佢");
        // Mandarin keeps 系 untouched.
        assert_eq!(
            localize_offline_transcript("他是中文系的", Some("zh")),
            "他是中文系的"
        );
    }

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
            localize_offline_transcript("呢个系咪你嘅，咁，就系咁先。", Some("yue")),
            "呢個係咪你嘅，咁，就係咁先。"
        );
        assert_eq!(
            to_hong_kong_traditional("唔该晒，我今日唔去观塘啦。"),
            "唔該晒，我今日唔去觀塘啦。"
        );
        assert_eq!(
            localize_offline_transcript("佢系唔系真系咁讲", Some("yue")),
            "佢係唔係真係咁講"
        );
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
