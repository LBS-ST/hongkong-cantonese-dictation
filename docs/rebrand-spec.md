# Rebrand Spec — 唔打啦 (Mudala)

## 0. 定位
- 呢份係 rebrand 嘅唯一權威 spec。讀者 = coding agent（Claude Code）。
- 將 app 由上游名「OpenLess」改名做「唔打啦」，同時保留 AGPL-3.0 合規。

## 1. 已定決定（勿改）
- 中文名：**唔打啦**
- 拉丁/英文名：**Mudala**（粵拼 m4 daa2 laa3）
- Bundle ID：**`com.lbs-smarttech.mudala`**（取代 `com.openless.app`）
- License：保留 AGPL-3.0 + OpenLess 原版 attribution / 版權聲明（強制，見 SPEC.md §7）

## 2. 範圍（要改）

### 2.1 tauri.conf.json（`openless-all/app/src-tauri/tauri.conf.json`）
- `productName`: `OpenLess` → `唔打啦`
- `identifier`: `com.openless.app` → `com.lbs-smarttech.mudala`
- 所有 window `title`: `OpenLess` → `唔打啦`（包括 `OpenLess Capsule` → `唔打啦 Capsule`）

### 2.2 Icon（`openless-all/app/src-tauri/icons/`）
- 換新 icon（「唔打啦」概念：咪高峰 + 粵語元素，簡潔扁平）
- 生成／替換全套：`icon.icns`（macOS）、`icon.ico`（Windows）、`icon.png`、`32x32` / `64x64` / `128x128` / `128x128@2x` / 各 Square 尺寸
- Android / iOS 子資料夾 icon 呢輪唔使碰（non-goal，M1 唔做 mobile）

### 2.3 用戶見到嘅「OpenLess」字
- `README.md` / `README.zh.md`：標題、app 名改做「唔打啦」/ Mudala；**保留**「fork 自 OpenLess（AGPL-3.0）」attribution + 上游連結
- 前端 UI 有 user-facing「OpenLess」字嘅地方（onboarding / about / 視窗標題）→ 改做「唔打啦」

## 3. 唔改（內部，唔好碰）
- Rust crate 名：`openless` / `openless_lib` / `openless_core` / `openless-linux-egui` 等（改咗會大範圍 refactor，零用戶價值）
- 資料夾名：`openless-all` / `src-tauri` 等
- module 名、`use` import path、`crate::` 路徑
- `docs/` 上游文件（fork 歷史，唔好改）
- `.github/` workflows（除非 build 實際需要）
- `CLAUDE.md` 講 codebase 結構嘅部分（agent 自己會讀，唔好因為改名而破壞）

## 4. 驗收
1. macOS build 成功，.app 名同視窗標題顯示「唔打啦」
2. bundle ID = `com.lbs-smarttech.mudala`（About / System Info 睇到）
3. 新 icon 喺 Dock / Finder 顯示
4. README 有「唔打啦」名 + AGPL/OpenLess attribution 齊全
5. core tests + frontend tests 照舊全過（rebrand 唔應該郁 test）
