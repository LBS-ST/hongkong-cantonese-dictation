//! Deterministic user correction rules shared by every UI host.
//!
//! Correction rules are separate from vocabulary hints. They are applied after
//! transcription/polishing and intentionally support only the conservative
//! `{num}` wildcard instead of arbitrary regular expressions.

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Utc;

use crate::errors::{BackendError, BackendErrorCode};
use crate::persistence::{atomic_write, persistence_error};
use crate::types::{CorrectionRule, RuleSource};

const NUM_TOKEN: &str = "{num}";

/// Persistent correction-rule repository with framework-independent paths.
pub struct CorrectionRuleStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl CorrectionRuleStore {
    pub fn at_data_dir(data_dir: impl AsRef<Path>) -> Self {
        Self::at_path(data_dir.as_ref().join("correction-rules.json"))
    }

    pub fn at_path(path: PathBuf) -> Self {
        Self {
            path,
            lock: Mutex::new(()),
        }
    }

    pub fn list(&self) -> Result<Vec<CorrectionRule>, BackendError> {
        let _guard = self.lock_store()?;
        self.read_locked()
    }

    pub(crate) fn sync_snapshot(
        &self,
        permit: &crate::cloud_sync_e2ee_store::gate::ExclusivePermit,
    ) -> Result<Vec<CorrectionRule>, BackendError> {
        crate::cloud_sync_e2ee_store::gate::require_exclusive(&self.path, permit)?;
        let _guard = self.lock_store()?;
        crate::persistence::read_lossless_rows(&self.path, &[])
    }

    pub(crate) fn sync_replace_all(
        &self,
        records: &[CorrectionRule],
        permit: &crate::cloud_sync_e2ee_store::gate::ExclusivePermit,
    ) -> Result<(), BackendError> {
        crate::cloud_sync_e2ee_store::gate::require_exclusive(&self.path, permit)?;
        let _guard = self.lock_store()?;
        let bytes = serde_json::to_vec_pretty(records)
            .map_err(|_| persistence_error("encode restored corrections"))?;
        crate::persistence::atomic_write_for_sync(&self.path, &bytes, permit)
    }

    pub(crate) fn cloud_sync_access(
        &self,
    ) -> Result<(std::sync::MutexGuard<'_, ()>, &Path), BackendError> {
        Ok((self.lock_store()?, &self.path))
    }

    pub fn add(
        &self,
        pattern: String,
        replacement: String,
    ) -> Result<CorrectionRule, BackendError> {
        crate::cloud_sync_e2ee_store::gate::with_registered_mutation(
            &self.path,
            crate::cloud_sync_e2ee_store::gate::ChangeOrigin::User,
            || self.add_with_source(pattern, replacement, RuleSource::Manual),
        )
    }

    pub fn add_with_source(
        &self,
        pattern: String,
        replacement: String,
        source: RuleSource,
    ) -> Result<CorrectionRule, BackendError> {
        crate::cloud_sync_e2ee_store::gate::with_registered_mutation(
            &self.path,
            crate::cloud_sync_e2ee_store::gate::ChangeOrigin::User,
            || {
                let pattern = pattern.trim().to_string();
                let replacement = replacement.trim().to_string();
                validate_correction_rule_syntax(&pattern, &replacement)?;
                let _guard = self.lock_store()?;
                let mut rules = self.read_locked()?;
                let rule = CorrectionRule {
                    id: uuid::Uuid::new_v4().to_string(),
                    pattern,
                    replacement,
                    enabled: true,
                    created_at: Utc::now().to_rfc3339(),
                    source,
                    exceptions: Vec::new(),
                };
                rules.insert(0, rule.clone());
                self.write_locked(&rules)?;
                Ok(rule)
            },
        )
    }

    /// Removing an unknown id is deliberately idempotent.
    pub fn remove(&self, id: &str) -> Result<(), BackendError> {
        crate::cloud_sync_e2ee_store::gate::with_registered_mutation(
            &self.path,
            crate::cloud_sync_e2ee_store::gate::ChangeOrigin::User,
            || {
                let _guard = self.lock_store()?;
                let mut rules = self.read_locked()?;
                let before = rules.len();
                rules.retain(|rule| rule.id != id);
                if rules.len() != before {
                    self.write_locked(&rules)?;
                }
                Ok(())
            },
        )
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> Result<(), BackendError> {
        crate::cloud_sync_e2ee_store::gate::with_registered_mutation(
            &self.path,
            crate::cloud_sync_e2ee_store::gate::ChangeOrigin::User,
            || {
                let _guard = self.lock_store()?;
                let mut rules = self.read_locked()?;
                let rule = rules.iter_mut().find(|rule| rule.id == id).ok_or_else(|| {
                    BackendError::new(
                        BackendErrorCode::InvalidArgument,
                        "correction rule not found",
                    )
                })?;
                if rule.enabled != enabled {
                    rule.enabled = enabled;
                    self.write_locked(&rules)?;
                }
                Ok(())
            },
        )
    }

    /// Add shipped rules (e.g. the Hong Kong Cantonese corrections) to the user's list.
    ///
    /// Each rule id is seeded at most once: the ids already offered are recorded in a
    /// marker file next to the store, so a built-in the user deleted or edited is never
    /// re-added, while built-ins introduced by a later release still are. Returns how
    /// many rules were added.
    pub fn seed_builtin_rules(&self, builtins: &[CorrectionRule]) -> Result<usize, BackendError> {
        let marker = self.path.with_extension("builtin-seed.json");
        // Fast path: every built-in was already offered → no gate mutation, no file write.
        let already_offered: Vec<String> = std::fs::read(&marker)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        if builtins
            .iter()
            .all(|builtin| already_offered.contains(&builtin.id))
        {
            return Ok(0);
        }
        crate::cloud_sync_e2ee_store::gate::with_registered_mutation(
            &self.path,
            crate::cloud_sync_e2ee_store::gate::ChangeOrigin::User,
            || {
                let _guard = self.lock_store()?;
                let mut offered: Vec<String> = match std::fs::read(&marker) {
                    Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_default(),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
                    Err(_) => return Err(persistence_error("read builtin correction marker")),
                };
                let mut rules = self.read_locked()?;
                let mut added = 0;
                let mut newly_offered = 0;
                for builtin in builtins {
                    if offered.contains(&builtin.id) {
                        continue;
                    }
                    offered.push(builtin.id.clone());
                    newly_offered += 1;
                    if !rules.iter().any(|rule| rule.id == builtin.id) {
                        rules.push(builtin.clone());
                        added += 1;
                    }
                }
                if added > 0 {
                    self.write_locked(&rules)?;
                }
                // The marker only changes when an id is offered for the first time.
                if newly_offered > 0 {
                    let json = serde_json::to_vec_pretty(&offered)
                        .map_err(|_| persistence_error("encode builtin correction marker"))?;
                    atomic_write(&marker, &json)?;
                }
                Ok(added)
            },
        )
    }

    fn lock_store(&self) -> Result<std::sync::MutexGuard<'_, ()>, BackendError> {
        self.lock.lock().map_err(|_| {
            BackendError::new(
                BackendErrorCode::Internal,
                "correction rule store lock poisoned",
            )
        })
    }

    fn read_locked(&self) -> Result<Vec<CorrectionRule>, BackendError> {
        crate::persistence::read_lossless_rows(&self.path, &[])
    }

    fn write_locked(&self, rules: &[CorrectionRule]) -> Result<(), BackendError> {
        let json = serde_json::to_vec_pretty(rules)
            .map_err(|_| persistence_error("encode correction rules"))?;
        atomic_write(&self.path, &json)
    }
}

pub fn validate_correction_rule_syntax(
    pattern: &str,
    replacement: &str,
) -> Result<(), BackendError> {
    if pattern.is_empty() {
        return Err(BackendError::new(
            BackendErrorCode::InvalidArgument,
            "correction rule pattern is empty",
        ));
    }
    let pattern_token_count = pattern.matches(NUM_TOKEN).count();
    let invalid = pattern_token_count > 1
        || (replacement.contains(NUM_TOKEN) && pattern_token_count == 0)
        || (pattern_token_count == 1
            && pattern
                .split_once(NUM_TOKEN)
                .is_none_or(|(prefix, suffix)| prefix.is_empty() && suffix.is_empty()));
    if invalid {
        return Err(BackendError::new(
            BackendErrorCode::InvalidArgument,
            "unsupported correction rule syntax",
        ));
    }
    Ok(())
}

/// Apply enabled correction rules sequentially.
pub fn apply_correction_rules(text: &str, rules: &[CorrectionRule]) -> String {
    let mut current = text.to_string();
    for rule in rules {
        if !rule.enabled {
            continue;
        }
        let pattern = rule.pattern.trim();
        if pattern.is_empty() {
            continue;
        }
        current = if rule.exceptions.is_empty() {
            apply_rule(&current, pattern, &rule.replacement)
        } else {
            replace_words(
                &current,
                &[(pattern, rule.replacement.as_str())],
                &rule
                    .exceptions
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )
        };
    }
    current
}

/// Literal word replacement that leaves exception words untouched.
///
/// Scans left to right: an exception word starting at the cursor is copied verbatim
/// (so `安穩` / `穩定` protect their 穩), otherwise the longest matching `from` is replaced.
/// Used by rules with exceptions and by the offline written-Chinese table.
pub fn replace_words(text: &str, replacements: &[(&str, &str)], exceptions: &[&str]) -> String {
    let mut output = String::with_capacity(text.len());
    let mut cursor = 0;
    'scan: while cursor < text.len() {
        let rest = &text[cursor..];
        if let Some(word) = exceptions
            .iter()
            .filter(|word| !word.is_empty() && rest.starts_with(**word))
            .max_by_key(|word| word.len())
        {
            output.push_str(word);
            cursor += word.len();
            continue 'scan;
        }
        if let Some((from, to)) = replacements
            .iter()
            .filter(|(from, _)| !from.is_empty() && rest.starts_with(*from))
            .max_by_key(|(from, _)| from.len())
        {
            output.push_str(to);
            cursor += from.len();
            continue 'scan;
        }
        let character = rest.chars().next().expect("cursor is on a char boundary");
        output.push(character);
        cursor += character.len_utf8();
    }
    output
}

/// Apply one correction pattern.
///
/// This lower-level operation remains public for the selection edit-plan
/// compatibility adapter. UI hosts should normally use
/// [`apply_correction_rules`].
pub fn apply_rule(text: &str, pattern: &str, replacement: &str) -> String {
    let token_count = pattern.matches(NUM_TOKEN).count();
    if token_count == 0 {
        if replacement.contains(NUM_TOKEN) {
            return text.to_string();
        }
        return text.replace(pattern, replacement);
    }
    if token_count != 1 {
        return text.to_string();
    }
    apply_num_rule(text, pattern, replacement)
}

fn apply_num_rule(text: &str, pattern: &str, replacement: &str) -> String {
    let Some((prefix, suffix)) = pattern.split_once(NUM_TOKEN) else {
        return text.to_string();
    };
    if prefix.is_empty() && suffix.is_empty() {
        return text.to_string();
    }

    let mut output = String::with_capacity(text.len());
    let mut cursor = 0usize;
    while cursor < text.len() {
        let Some((match_start, token_start)) = next_prefix_match(text, cursor, prefix) else {
            break;
        };
        let Some(token_end) = consume_number_token(text, token_start) else {
            output.push_str(&text[cursor..next_char_boundary(text, match_start)]);
            cursor = next_char_boundary(text, match_start);
            continue;
        };
        let after_number = &text[token_end..];
        if !after_number.starts_with(suffix) {
            output.push_str(&text[cursor..next_char_boundary(text, match_start)]);
            cursor = next_char_boundary(text, match_start);
            continue;
        }

        let match_end = token_end + suffix.len();
        output.push_str(&text[cursor..match_start]);
        output.push_str(&replacement.replace(NUM_TOKEN, &text[token_start..token_end]));
        cursor = match_end;
    }
    output.push_str(&text[cursor..]);
    output
}

fn next_prefix_match(text: &str, cursor: usize, prefix: &str) -> Option<(usize, usize)> {
    if prefix.is_empty() {
        let match_start = next_number_start(text, cursor)?;
        return Some((match_start, match_start));
    }
    let relative = text[cursor..].find(prefix)?;
    let match_start = cursor + relative;
    Some((match_start, match_start + prefix.len()))
}

fn next_number_start(text: &str, cursor: usize) -> Option<usize> {
    text[cursor..]
        .char_indices()
        .find_map(|(offset, ch)| is_number_char(ch).then_some(cursor + offset))
}

fn consume_number_token(text: &str, start: usize) -> Option<usize> {
    let mut end = start;
    let mut consumed = false;
    for (offset, ch) in text[start..].char_indices() {
        if !is_number_char(ch) {
            break;
        }
        consumed = true;
        end = start + offset + ch.len_utf8();
    }
    consumed.then_some(end)
}

fn is_number_char(ch: char) -> bool {
    ch.is_ascii_digit()
        || matches!(
            ch,
            '零' | '〇'
                | '一'
                | '二'
                | '两'
                | '兩'
                | '三'
                | '四'
                | '五'
                | '六'
                | '七'
                | '八'
                | '九'
                | '十'
                | '百'
                | '千'
                | '万'
                | '萬'
                | '亿'
                | '億'
                | '几'
                | '幾'
        )
}

fn next_char_boundary(text: &str, start: usize) -> usize {
    text[start..]
        .chars()
        .next()
        .map(|ch| start + ch.len_utf8())
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RuleSource;

    fn rule(pattern: &str, replacement: &str) -> CorrectionRule {
        CorrectionRule {
            id: "rule".into(),
            pattern: pattern.into(),
            replacement: replacement.into(),
            enabled: true,
            created_at: String::new(),
            source: RuleSource::Manual,
            exceptions: Vec::new(),
        }
    }

    #[test]
    fn applies_literal_replacement() {
        let rules = vec![rule("几粒", "几例")];
        assert_eq!(
            apply_correction_rules("这里有几粒样品", &rules),
            "这里有几例样品"
        );
    }

    #[test]
    fn applies_num_wildcard_for_arabic_digits() {
        let rules = vec![rule("{num}粒", "{num}例")];
        assert_eq!(
            apply_correction_rules("2粒样品和10粒对照", &rules),
            "2例样品和10例对照"
        );
    }

    #[test]
    fn applies_num_wildcard_for_chinese_numbers() {
        let rules = vec![rule("{num}粒", "{num}例")];
        assert_eq!(
            apply_correction_rules("两粒样品和幾粒对照", &rules),
            "两例样品和幾例对照"
        );
    }

    #[test]
    fn disabled_rules_are_ignored() {
        let mut disabled = rule("{num}粒", "{num}例");
        disabled.enabled = false;
        assert_eq!(apply_correction_rules("10粒样品", &[disabled]), "10粒样品");
    }

    #[test]
    fn malformed_rules_are_inert() {
        let rules = vec![
            rule("{num}到{num}粒", "{num}例"),
            rule("几粒", "{num}例"),
            rule("{num}", "{num}例"),
        ];
        assert_eq!(apply_correction_rules("几粒和10粒", &rules), "几粒和10粒");
    }

    #[test]
    fn applies_rules_sequentially() {
        let rules = vec![rule("{num}粒", "{num}例"), rule("样本", "样品")];
        assert_eq!(apply_correction_rules("10粒样本", &rules), "10例样品");
    }

    #[test]
    fn syntax_validation_rejects_silent_noops() {
        assert!(validate_correction_rule_syntax("{num}粒", "{num}例").is_ok());
        assert!(validate_correction_rule_syntax("几粒", "几例").is_ok());
        for (pattern, replacement) in [
            ("", "几例"),
            ("{num}", "{num}例"),
            ("{num}到{num}粒", "{num}例"),
            ("几粒", "{num}例"),
        ] {
            assert_eq!(
                validate_correction_rule_syntax(pattern, replacement)
                    .unwrap_err()
                    .code,
                BackendErrorCode::InvalidArgument
            );
        }
    }

    #[test]
    fn legacy_source_defaults_and_round_trips() {
        let json = r#"{"id":"1","pattern":"甲","replacement":"乙","enabled":true,"createdAt":""}"#;
        let rule: CorrectionRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.source, RuleSource::Manual);
        assert_eq!(
            serde_json::to_string(&RuleSource::Learned).unwrap(),
            "\"learned\""
        );
    }

    #[test]
    fn store_round_trips_mutations_and_keeps_remove_idempotent() {
        let path = std::env::temp_dir().join(format!(
            "openless-core-correction-{}.json",
            uuid::Uuid::new_v4().simple()
        ));
        let store = CorrectionRuleStore::at_path(path.clone());
        let rule = store.add(" {num}粒 ".into(), " {num}例 ".into()).unwrap();
        assert_eq!(rule.pattern, "{num}粒");
        assert_eq!(store.list().unwrap(), vec![rule.clone()]);

        store.set_enabled(&rule.id, false).unwrap();
        assert!(!store.list().unwrap()[0].enabled);
        store.remove(&rule.id).unwrap();
        store.remove(&rule.id).unwrap();
        assert!(store.list().unwrap().is_empty());

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn exceptions_protect_whole_words_around_the_match() {
        let mut stable = rule("穩", "搵");
        stable.exceptions = ["穩定", "穩陣", "穩妥", "穩步", "穩健", "安穩"]
            .map(String::from)
            .to_vec();
        assert_eq!(
            apply_correction_rules("我去穩嘢，個系統好穩定，好穩陣，好安穩", &[stable]),
            "我去搵嘢，個系統好穩定，好穩陣，好安穩"
        );
        assert_eq!(
            replace_words("呢個關係係咁", &[("係", "是")], &["關係"]),
            "呢個關係是咁"
        );
        // Longest replacement wins over a shorter overlapping one.
        assert_eq!(
            replace_words(
                "唔係唔去",
                &[("唔係", "不是"), ("唔", "不"), ("係", "是")],
                &[]
            ),
            "不是不去"
        );
    }

    #[test]
    fn builtin_rules_are_seeded_once_and_stay_deleted() {
        let dir =
            std::env::temp_dir().join(format!("openless-builtin-seed-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let store = CorrectionRuleStore::at_data_dir(&dir);
        let user = store.add("觀塘".into(), "觀塘區".into()).unwrap();
        let mut builtin = rule("知家", "即刻");
        builtin.id = "builtin.test".into();
        builtin.source = RuleSource::Builtin;

        assert_eq!(store.seed_builtin_rules(&[builtin.clone()]).unwrap(), 1);
        let ids: Vec<_> = store
            .list()
            .unwrap()
            .into_iter()
            .map(|rule| rule.id)
            .collect();
        assert_eq!(ids, vec![user.id.clone(), "builtin.test".to_string()]);
        // Next launch: nothing new, and neither the marker nor the rules are rewritten.
        let marker = dir.join("correction-rules.builtin-seed.json");
        let marker_mtime = std::fs::metadata(&marker).unwrap().modified().unwrap();
        let rules_mtime = std::fs::metadata(dir.join("correction-rules.json"))
            .unwrap()
            .modified()
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        assert_eq!(store.seed_builtin_rules(&[builtin.clone()]).unwrap(), 0);
        assert_eq!(
            std::fs::metadata(&marker).unwrap().modified().unwrap(),
            marker_mtime
        );
        assert_eq!(
            std::fs::metadata(dir.join("correction-rules.json"))
                .unwrap()
                .modified()
                .unwrap(),
            rules_mtime
        );
        // The user deletes the built-in; it must not come back.
        store.remove("builtin.test").unwrap();
        assert_eq!(store.seed_builtin_rules(&[builtin.clone()]).unwrap(), 0);
        assert_eq!(store.list().unwrap().len(), 1);
        // A built-in shipped by a later release is still offered.
        let mut newer = builtin;
        newer.id = "builtin.newer".into();
        assert_eq!(store.seed_builtin_rules(&[newer]).unwrap(), 1);
        let _ = std::fs::remove_dir_all(dir);
    }
}
