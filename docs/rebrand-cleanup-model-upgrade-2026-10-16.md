# 唔打啦（Mudala）— Rebrand 遺留清理 + SenseVoice 廣東話模型升級 Spec

## 0. 定位
- 讀者 = Claude Code Desktop（Opus 4.8）
- 呢份係一次性 spec，兩部分：
  - **Part A**：rebrand 遺留清理（用戶見到嘅 OpenLess / Typeless / Siri / Beta 字眼，同多餘嘅設定項）
  - **Part B**：SenseVoice 廣東話模型升級（由 2024-07-17 舊版 → 2025-09-09 廣東話專用新版）
- 所有技術決定已由 CTO 落定，照做，**唔好自行更改**。
- 改完 push 去 `beta` branch。

---

## Part A — Rebrand 遺留清理

### A1. 錄音設定「風格」選單 → 只留兩個
- **檔案**：`openless-all/app/src/pages/settings/RecordingInputSection.tsx`（約行 473–488）
- **現狀**：3 個選項
  - `siri` = 「流光 Siri 風格」（目前預設值 `prefs.capsuleStyle ?? 'siri'`）
  - `classic` = 「Openless 預設風格」← 有「Openless」字
  - `typeless` = 「Typeless 傳統風格」← 有「Typeless」字
- **改成**：只留 2 個
  - `classic` → 標籤「預設風格」
  - `typeless` → 標籤「傳統風格」
  - 刪走 `siri` 選項
- **預設值**：由 `siri` 改做 `classic`（即「預設風格」= 預設）
- **i18n key**（`openless-all/app/src/i18n/*.ts`，8 個語系都要改）：
  - `settings.recording.capsuleStyleClassic` → 「預設風格」（zh-TW）
  - `settings.recording.capsuleStyleTypeless` → 「傳統風格」
  - `settings.recording.capsuleStyleSiri` → 可刪或留空（選項已移除）
- **⚠️ 注意**：`CapsuleStylePreview`（`TypelessCapsule.tsx`）同 `Capsule.tsx` 有 `siri` 分支（Siri orb），刪 `siri` 選項時要確保冇 render crash；`siri` 分支 code 可以保留（只係 UI 唔再畀揀），但預設值必須改做 `classic`。

### A2. 刪走 Settings「服務」section
- **檔案**：`openless-all/app/src/pages/settings/navigation.ts`（約行 51 `{ id: 'services', icon: 'cloud' }`）
- 由 `SETTINGS_SECTIONS` 移除 `services`，SettingsModal 就唔會再出呢個 section。
- 留意：`SettingsModal.tsx` 行 696 有 `{section === 'services' && <ServicesTab />}`，同行 619-620 有 `section === 'services'` 嘅 padding 特判，一併清。
- 唔需要刪 `ServicesTab` 底層檔案（離線版已隱藏雲端 provider），只係唔好喺 nav 出現。

### A3. 刪走「幫助中心」+「釋出日記」
兩個位置：
1. **`openless-all/app/src/components/SettingsModal.tsx`**：
   - 行 49–52 `LINKS` 陣列（`helpCenter` / `releaseNotes`）
   - 行 484 `{!mobile && <HelpLinks />}`、行 707 `{mobile && !searching && <HelpLinks />}`
   - 行 719–747 `HelpLinks` 組件（可成個刪）
2. **`openless-all/app/src/pages/settings/AboutSection.tsx`**：
   - 行 112–121（`helpCenter` + `releaseNotes` 兩個 `Row`）

### A4. About 頁清走 OpenLess 字眼 / 連結
- **檔案**：`openless-all/app/src/pages/settings/AboutSection.tsx`
- 刪走以下所有指去上游 OpenLess 嘅連結：
  - 行 15–16 `HELP_URL` / `RELEASE_NOTES_URL`（`https://github.com/Open-Less/openless...`）
  - 行 99–106 「原始碼 GitHub」按鈕（`openExternal('https://github.com/Open-Less/openless')`）
  - 行 107–111 「文檔」按鈕（`openExternal(HELP_URL)`，zh-TW 標籤係 `openless.app/docs ↗`）
  - 行 112–116 「幫助中心」（已喺 A3 刪）
  - 行 117–121 「釋出日記」（已喺 A3 刪）
  - 行 122–129 「意見反饋」（`openExternal('https://github.com/Open-Less/openless/issues')`）
  - 行 130–155 QQ 群（`1078960553`，OpenLess 官方群）
- **結果**：About 頁只保留 app 名「唔打啦」＋ tagline ＋ 版本號（`APP_VERSION_LABEL`）。刪走「檔案連結」整張 Card（`linksTitle`）如果入面已經冇嘢。

### A5. 刪走「Beta」字眼
- **版本號改做穩定版**：`2.0.0-Beta.4+build.20260930` → `2.0.0`（冇 prerelease 段）
- 版本號喺 **3 個檔案**，要一齊改保持一致：
  1. `openless-all/app/package.json`（`"version"`）
  2. `openless-all/app/src-tauri/tauri.conf.json`（`"version"`）
  3. `openless-all/app/src-tauri/Cargo.toml`（`version`）
- `openless-all/app/src/lib/appVersion.ts` 有 `IS_BETA_BUILD = APP_VERSION.includes('-')`；改完版本號 = `false` → FloatingShell 側欄 BETA badge（行 350–368）自動消失，唔使額外改。
- **全域 grep**：`Beta` / `beta` / `BETA` 喺 `openless-all/app/src/` + `src/i18n/`，清走**用戶見到嘅**殘留（例 `shell.betaTag: 'BETA'`、`shell.betaNote`）。⚠️ 唔好刪 code 邏輯變數（如 `isStableChannelSwitch`），只清 UI 顯示。注意 update 相關嘅 `betaChannel*` key 屬已剷嘅 auto-updater，如仍被引用一併清。

### A6. 全域 grep（防漏）
- 喺 `openless-all/app/src/`（含 `i18n/`、`components/`、`pages/`）grep 以下字眼，清走**用戶見到**嘅殘留：
  - `OpenLess` / `openless` / `Open-Less`（i18n `docsBtn: 'openless.app/docs ↗'` 呢類）
  - `Typeless` / `TypeLess`（i18n 標籤、`windowsShowOpenlessInKeyboardListLabel` 呢類 Windows-only 字）
  - `Siri`（i18n 標籤）
- 注意：`mockData.ts` / 測試檔 / 內部 event name（`openless:ui-preferences-changed`）/ Rust 檔名（`openless-core`）**唔使改**——只改用戶喺 UI 見到嘅字。

---

## Part B — SenseVoice 廣東話模型升級

### B1. 換模型 repo
- **檔案**：`openless-all/app/crates/openless-core/src/model_store.rs`（約行 326–336）
- **現 repo**：`csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17`
- **新 repo**：`csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2025-09-09`
- **檔案清單一樣**（已驗證 HF 有）：`model.int8.onnx` + `tokens.txt`，所以只改 repo URL 就得，`exact(files)` selector 唔使改。
- **alias 保持** `sense-voice-small-zh`（唔好改，避免連鎖改 `OFFLINE_ASR_MODEL`、`sherpa_family` 等）。
- 新模型轉自 `ASLP-lab/WSYue-ASR`（`sensevoice_small_yue`）= 廣東話專用訓練，對香港廣東話口語準確度明顯好過舊版。

### B2. ⚠️ 強制：已下載舊模型要重新下載
- 問題：alias 一樣 = 本地 model 資料夾路徑一樣（`sherpa-onnx/sense-voice-small-zh/`）。已裝舊版嘅用戶（例如 Alex 部 Mac）會因為本地已有 `model.int8.onnx` + `tokens.txt` 而**唔會重新下載**，靜靜繼續用舊 2024-07-17 模型。
- **要求**：令已下載舊模型嘅安裝喺更新後自動偵測「模型已過期」並重新下載新模型。做法由你揀（二選一）：
  - (a) 喺 `model_store.rs` 加 model version / 內容 hash 檢查，比對本地檔 hash 唔同就 re-download；或
  - (b) 其他你覺得最乾淨、唔會影響其他模型嘅機制。
- **驗收**：更新後，舊安裝下一次啟動會見到 model 重新下載（或至少唔會用返舊檔）。

### B3. 更新 SPEC.md 內嘅舊 repo 名
- `SPEC.md` 行 27 有舊 repo 名 `csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17`，同步改做新 repo 名。

---

## 驗收（Alex 實機測試清單）
1. 錄音設定「風格」只有「預設風格」「傳統風格」兩個，冇英文品牌字。
2. Settings 冇「服務」section。
3. Settings 側欄底部同 About 頁冇「幫助中心」「釋出日記」。
4. About 頁冇任何 OpenLess / openless 字眼同外部 GitHub 連結，只顯示「唔打啦」＋版本。
5. 側欄冇 BETA 徽章，版本號係穩定版（無 Beta 字）。
6. 模型重新下載新版，廣東話口語準確度有提升（Alex 口述測試）。

## 交付
- 全部改動 push 去 `beta` branch，commit message 清楚分 Part A / Part B。
