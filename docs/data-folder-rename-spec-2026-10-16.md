# 唔打啦（Mudala）— 資料夾改名（OpenLess → Mudala）Spec

## 問題
app 內部仲用緊舊名「OpenLess」做資料夾名（用戶正常唔會直接見到，但係系統入面嘅路徑）：
- 資料夾：`~/Library/Application Support/OpenLess`（存 preferences / history / 模型）
- Log 資料夾：`~/Library/Logs/OpenLess`

## Fix
改「OpenLess」→「Mudala」，兩處：

### 1. `data_dir()`
- **檔案**：`openless-all/app/src-tauri/src/persistence/mod.rs`（行 61–96）
- macOS：`Application Support/OpenLess` → `Application Support/Mudala`（行 68）
- Windows：`%APPDATA%/OpenLess` → `%APPDATA%/Mudala`（行 74）
- Linux：`$XDG_DATA_HOME/OpenLess` 同 `~/.local/share/OpenLess` → `Mudala`（行 81、88）
- Android：唔改（M1 唔做 Android）

### 2. `log_dir_path()`
- **檔案**：`openless-all/app/src-tauri/src/lib.rs`（行 1771–1812）
- macOS：`~/Library/Logs/OpenLess` → `Mudala`（行 1778）
- Windows：`%LOCALAPPDATA%/OpenLess/Logs` → `Mudala/Logs`（行 1785）
- Linux：`~/.local/share/OpenLess/logs` → `Mudala/logs`（行 1795）
- fallback：`temp_dir/OpenLess` → `temp_dir/Mudala`（行 1810）
- Android：唔改

### 3. 全域 grep 防漏
- 喺 `src-tauri/src/` grep 其他 `"OpenLess"` 字串做資料夾/路徑名，一併改（例如 `android_storage.rs` 嘅 `{filesDir}/OpenLess` 都改，但 Android 分支本身可以唔郁）。
- ⚠️ 唔好改：code comment、`openless_core` 類型名、`OpenLessIme.dll` 檔名、IME window class 常數、Android package 名。

## 後果（已知 trade-off，已接受）
- 改咗之後 app 會喺新資料夾搵唔到舊資料 → 舊 preferences / history 唔會搬過去，模型要重新下載（~900MB）。
- 公開前做，新用戶由 Day 1 就係乾淨嘅 `Mudala` 資料夾。Alex 部 Mac 要重下載一次模型，預咗。

## 驗收
1. `cargo check` 通過。
2. app 正常啟動，新資料夾名係 `Mudala`（唔再係 `OpenLess`）。

## 交付
- push 去 `beta` branch。
