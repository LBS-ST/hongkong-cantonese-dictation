# 唔打啦（Mudala）— Tray 選單 Rebrand 遺留 Fix Spec

## 問題
macOS 選單欄（menu bar）個 app icon 撳出嚟嘅選單，最後一項仲寫「退出 OpenLess」（8 個語言全部係）。位置喺 Rust code，唔喺 frontend i18n，所以之前 A6 grep 漏咗。

## Fix

### 1. 改 tray 選單「退出」標籤
- **檔案**：`openless-all/app/src-tauri/src/lib.rs`（約行 1014–1105，`TrayMenuLocale` struct 嘅 `quit` 欄位）
- 8 個語言，將「OpenLess」改做「唔打啦」：
  - en：`"Quit OpenLess"` → `"Quit 唔打啦"`
  - es：`"Salir de OpenLess"` → `"Salir de 唔打啦"`
  - fr：`"Quitter OpenLess"` → `"Quitter 唔打啦"`
  - de：`"OpenLess beenden"` → `"唔打啦 beenden"`
  - zh-TW：`"退出 OpenLess"` → `"退出 唔打啦"`
  - ja：`"OpenLessを終了"` → `"唔打啦を終了"`
  - ko：`"OpenLess 종료"` → `"唔打啦 종료"`
  - zh-CN（default）：`"退出 OpenLess"` → `"退出 唔打啦"`

### 2. 全域 grep 防漏（Rust 層）
- 喺 `openless-all/app/src-tauri/src/` grep 用戶會見到嘅 `"OpenLess"` / `"openless"` 字串 literal，清走。
- ⚠️ 唔好改以下（唔係用戶可見，或 deferred）：
  - code comment（`// allow OpenLess to access...` 呢類）
  - `openless_core` / `OpenLessBackend` 類型名
  - thread name（`"openless-dictation-key"` 等）
  - Android package（`com.openless.app`，M1 唔做 Android）
  - log 路徑 `~/Library/Logs/OpenLess`（deferred）
  - data folder `~/Library/Application Support/OpenLess`（deferred，改咗要重下載模型）

## 驗收
1. 選單欄選單最後一項顯示「退出 唔打啦」（唔再係「退出 OpenLess」）。
2. `cargo check` 通過。

## 交付
- push 去 `beta` branch。
