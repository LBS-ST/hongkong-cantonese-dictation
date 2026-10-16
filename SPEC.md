# 香港廣東話離線聽寫 App — SPEC

## 0. 文件定位
- 呢份係 **M1（v0.1 個人可用）嘅唯一權威 spec**。
- 讀者 = coding agent（Codex / Claude Code）。
- 所有技術決定已由 CTO 落定，coding agent 照做，**唔好自行更改決定**。

## 1. 項目定義
- **產品**：香港廣東話離線聽寫 App（macOS + Windows）
- **一句話**：開源、跨 Mac/Windows、全程離線、專為香港廣東話口語嘅語音輸入工具
- **基底**：fork 自 OpenLess（AGPL-3.0），本 repo = `LBS-ST/cantonese-dictation`

## 2. Non-goals（明確唔做，唔好自作主張加）
- ❌ 唔做 iOS / Android
- ❌ 唔做任何伺服器 / 雲端 API 依賴（全程 on-device）
- ❌ 唔做訂閱 / 收費機制
- ❌ 唔 reverse engineer Wispr Flow 或任何閉源 app
- ❌ M1 唔加任何新功能（只做「令現有 app 離線跑廣東話」）

## 3. 技術基底（已確認，勿改）
- 框架：Tauri 2 + Rust + React/TypeScript
- 本地 ASR：SenseVoice Small（sherpa-onnx，yue）
- 目標平台：M1 = macOS（Apple Silicon 優先）；Windows 留 M3

## 4. 引擎決定（CTO 已定，勿改）
- M1 默認引擎 = **SenseVoice Small（sherpa-onnx，yue）**
- 模型 = `csukuangfj/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-int8-2025-09-09`（OpenLess 已整合，model alias = `sense-voice-small-zh`）
- 語言 = 鎖死 **yue**，唔做自動語言偵測
- Qwen3-ASR 唔喺 M1 用（留 M2 A/B 測試）

## 5. M1 範圍（v0.1 個人可用）

### 5.1 全程離線（核心）
1. 開 app **唔需要任何雲端憑證**，首次啟動即可用
2. 本地 ASR（SenseVoice yue）= 默認引擎，無需配置
3. 雲端 ASR provider（Volcengine / Tencent / iFlytek / Alibaba 百煉 / StepFun / 智譜 GLM / 小米 MiMo / ElevenLabs / OpenAI 兼容 / Apple Speech 等）全部 **禁用 + 喺 UI 隱藏**，令用戶完全見唔到、用唔到
4. AI 潤飾（LLM polish，見 `openless-core/src/polish.rs`）= **默認關閉**（口語粵字輸出需要，潤飾會將口語轉書面語）

> 注意：M1 係「禁用 + 隱藏」雲端，**唔做物理刪除**雲端 code（後期清理另開 task）。目標係用戶完全見唔到、用唔到、唔需要雲端。

### 5.2 鎖定 yue + 口語粵字
1. 語言鎖死 yue，唔自動偵測
2. 輸出必須保留口語字「呢 / 唔 / 嘅 / 咁 / 係咁先」等，**唔得轉書面語**
3. 中 / EN / Yue 手動切換可以保留（但默認 yue）

### 5.3 模型一鍵下載
- 用現有 model_store 邏輯，內建 SenseVoice yue 模型一鍵下載
- 下載完成後，斷網都能跑

### 5.4 macOS build
- Apple Silicon 優先
- hotkey 按住講 → 鬆手 → 文字出喺 cursor

## 6. M1 驗收標準（全部要過）
1. ✅ Alex Mac 上 hotkey 按住講嘢 → 鬆手 → 廣東話文字出喺 cursor
2. ✅ 引擎鎖 yue，口語粵字保留（「我今日唔去觀塘喇」唔會變「我今天不去觀塘了」）
3. ✅ 斷網測試通過
4. ✅ 內建模型一鍵下載

## 7. License 合規（強制）
- 保留 AGPL-3.0 license + 原版 attribution / 版權聲明
- README 加 SenseVoice 模型（FunASR Model License）attribution + license 連結
- 禁止抄 Wispr Flow 任何閉源資源（名、icon、文案、binary）

## 8. 香港化（M2 先做，M1 唔好碰）
- hotword 詞典 + 港式詞典（18 區地名 / 公司名 / 俗語）
- 口語 / 書面輸出切換
- 中英夾雜優化
- 廣東話測試集 CER ≤ 10%

## 9. 開發環境
- 開發位置：Alex Mac local（Codex / Claude Code 寫 code）
- 唔喺 VPS 跑 dev build
- 每個 milestone 完成後：Alex 實機測試確認先過下一關

## 10. 重要 code 位置（coding agent 起手參考）
- 本地 ASR catalog：`openless-all/app/crates/openless-core/src/local_asr_catalog.rs`
- 模型下載：`openless-all/app/crates/openless-core/src/model_store.rs`（`sense-voice-small-zh` alias → yue 模型）
- sherpa-onnx runtime：`openless-all/app/src-tauri/src/asr/local/sherpa_runtime.rs`
- Qwen3-ASR engine：`openless-all/app/src-tauri/src/asr/local/qwen_engine.rs`
- 本地 ASR service：`openless-all/app/crates/openless-core/src/local_asr_service.rs`
- 繁體 UI：`openless-all/app/src/i18n/zh-TW.ts`
