# 唔打啦（Mudala）— Universal Binary（Intel + Apple Silicon）Spec

## 目標
產出可以喺 **Intel（x86_64）同 Apple Silicon（arm64）** Mac 都行到嘅 app。一個檔（universal）最好，分開兩個 .dmg 都得。

## 現況（已查證）
- `scripts/build-mac.sh` 已分 arm64 / x86_64 兩個 case 處理，但係**分開 build**（跟 `uname -m`），唔係 universal。產出 `.dmg` 名會帶 arch（`_aarch64` / `_x64`）。
- arm64 build 會加 `--config src-tauri/tauri.macos-mlx.conf.json`（MLX 框架，Qwen3-ASR 用，arm64 only）。
- `src-tauri/Cargo.toml` 行 141–146：`qwen3-asr-rs`（`features = ["mlx"]`）淨係 `target_arch = "aarch64"` 先有。x86_64 自動唔包。
- ⚠️ **關鍵**：offline mode（M1）唔用 Qwen3/MLX，唯一引擎係 SenseVoice（sherpa-onnx）。所以 MLX 係死 code，可以喺 universal build 排除，唔影響功能。

## 做法（agent 揀最乾淨、build 唔壞嘅）
1. `rustup target add x86_64-apple-darwin`（加 Intel target）。
2. 產出 universal binary，兩個方向二選一：
   - **(a) 排除 MLX**（推薦，最簡單）：universal build 唔帶 `qwen3-asr-rs` mlx feature，兩邊都淨 SenseVoice。產出一個 .dmg 同時支援兩邊。
   - **(b) 分開 build**：arm64 + x86_64 兩個 .dmg（`build-mac.sh` 已 support），兩個都上載。
3. ⚠️ 唔好改 offline-lock（SenseVoice 依然係唯一引擎、雲端照舊禁用）。
4. ⚠️ 改咗 build script / tauri config 嘅話，要 `cargo check` 通過 + 產出嘅 .app 用 `lipo -info` 確認包含兩個 arch。

## 驗收
1. 產出嘅 `.app`/`.dmg`：`lipo -info <app>/Contents/MacOS/mudala` 顯示 `arm64 x86_64`（方案 a），或兩個分開 .dmg（方案 b）。
2. build 通過（`npm run tauri build` 或 `build-mac.sh`）。
3. 最終 Alex 喺佢部 Intel Mac 實測開到（呢步等產出後 Alex 驗）。

## 交付
- 如改咗 build script / config：push 去 `beta` branch。
- 話我知產出咗邊個方案（a 定 b）、.dmg/.app 路徑、`lipo -info` 結果。
