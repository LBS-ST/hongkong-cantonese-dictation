# 唔打啦（Mudala）— 鑰匙串 Rebrand 遺留 Fix Spec

## 問題
macOS 鑰匙串提示框顯示「openless 想使用你在鑰匙圈的「com.openless.app」裏儲存的機密資料」。
兩個 rebrand 遺留：
1. 鑰匙串 service 名 = `com.openless.app`
2. binary 名 = `openless`

## Fix

### 1. 改鑰匙串 service 名
- **檔案**：`openless-all/app/src-tauri/src/persistence/credentials.rs`（約行 3525）
- 由 `pub const SERVICE_NAME: &'static str = "com.openless.app";` → `"com.lbs-smarttech.mudala"`
- 呢個 constant 係唯一 source（`macos_keychain.rs` + `keyring_entry_for` 都用佢），改一處即全局生效。
- ⚠️ 改咗之後，舊 `com.openless.app` 存嘅 credential 唔會再讀返。離線版唔用 sync，所以冇影響（app 會喺新名之下重新 create empty credential）。

### 2. 改 binary 名（令鑰匙串提示顯示新 app 名）
- **檔案**：`openless-all/app/src-tauri/Cargo.toml`（行 2 `name = "openless"`）
- 目標：macOS 鑰匙串提示框嘅 app 名由「openless」→ 新名（`mudala`）。
- 做法（你揀最乾淨、build 唔壞嘅）：
  - (a) 改 Cargo package `name` → `mudala`，並喺 `tauri.conf.json` 加 `"mainBinaryName": "mudala"`；或
  - (b) 其他 Tauri 2 正確做法。
- ⚠️ 唔好改 `[lib] name = "openless_lib"`（內部 library 名，唔影響顯示）。
- ⚠️ 順手清埋 `Cargo.toml` 行 5-6 嘅 `description` / `authors`（仍寫「OpenLess」），改成「唔打啦」相關。

## 驗收
1. macOS 鑰匙串提示框唔再顯示「openless」或「com.openless.app」（改顯示新名）。
2. `npm run tauri dev` build 通過 + app 正常啟動。
3. `cargo check` 通過。

## 交付
- push 去 `beta` branch。
