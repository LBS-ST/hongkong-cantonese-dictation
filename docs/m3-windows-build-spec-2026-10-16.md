# M3 — Windows Build Spec（香港廣東話離線聽寫 App / 唔打啦）

日期: 2026-10-16
狀態: 待 Alex approve
讀者: Alex + Claude Code Desktop (Opus 4.8)

## 1. 目標
令 fork（唔打啦 / Mudala）成功 build 出 Windows 版 installer（.msi / .exe），Windows 版功能同 Mac 版對齊（本地 SenseVoice 廣東話引擎、港式糾正規則、口語/書面切換、全程離線）。

## 2. 背景
M1 時明示「Windows 留 M3」。2026-10-16 觸發 CI `release-tauri.yml`（workflow_dispatch, platform=all）驗證，結果：
- macOS arm64 job ✅ 成功
- macOS Intel job ❌（compile 成功，最後 DMG bundle script 失敗 — CI 環境問題，另計，唔喺 M3 範圍）
- Windows x64 job ❌ 失敗（6 個 compile error）

## 3. Root cause（已 100% 核實，非猜測）

`openless-all/app/src-tauri/src/build_target.rs` 嘅 `classify_target_os()` **冇 "windows" branch**：

```rust
pub(crate) fn classify_target_os(target: &str, cargo_cfg_target_os: Option<&str>) -> &'static str {
    match cargo_cfg_target_os {
        Some("android") => "android",
        Some("macos") => "macos",
        Some("linux") => "linux",
        Some(_) => "",                       // ← Windows 命中呢度，回傳空字串
        None if target.contains("-android") || target.contains("-androideabi") => "android",
        None if target.ends_with("-apple-darwin") => "macos",
        None if target.contains("-linux-") => "linux",
        None => "",
    }
}
```

因果鏈：
1. Windows build 時 `CARGO_CFG_TARGET_OS = "windows"` → `classify_target_os` 回傳 `""`。
2. `build.rs` 入面 `if matches!(target_os, "windows" | "macos") { cargo:rustc-cfg=sherpa_native }` → `target_os = ""` 唔 match → **冇 set `sherpa_native` cfg**。
3. 所有 `#[cfg(sherpa_native)]` 嘅 sherpa-onnx 代碼被 compile out。
4. `core_adapters.rs` 有 ungated 嘅 Windows code path 引用呢啲嘢 → 6 個 compile error。

**證據**：`build.rs` 第 16-20 行 comment 明寫「the sherpa-onnx runtime is compiled for macOS as well as Windows」+ `matches!(target_os, "windows" | "macos")`，即意圖係 Windows 都要 sherpa_native，只係 `classify_target_os` 漏咗 windows branch。

### 6 個 error（全部喺 `core_adapters.rs`，全因 sherpa_native 未 set）
| error | 位置 | 原因 |
|---|---|---|
| E0425 cannot find value `sherpa` | 1747 | `let sherpa`（1701）用 `#[cfg(sherpa_native)]` |
| E0425 cannot find function `start_sherpa_session` | 1747 | fn（1932）用 `#[cfg(sherpa_native)]` |
| E0560 struct 無 field `sherpa_generation` | 79 | field（62）用 `#[cfg(sherpa_native)]` |
| E0609 no field `sherpa_generation` | 1686 | 同上 |
| E0609 no field `generation` | 2280 | struct field（1663）用 `#[cfg(sherpa_native)]` |
| E0609 no field `current_generation` | 2281 | struct field（1665）用 `#[cfg(sherpa_native)]` |

## 4. Fix（首要，一行起跳）

`build_target.rs` `classify_target_os()` 加返 windows branch：

```rust
Some("windows") => "windows",
```

fallback path（舊 toolchain 冇 CARGO_CFG_TARGET_OS 時）加：

```rust
None if target.contains("-windows-") => "windows",
```

## 5. Fix 後嘅後續（逐關清，唔好預設一次過全綠）

Fix 咗 classify 之後 Windows compile 應該過到 sherpa 呢層，但可能再有下一層 error 浮出（compile 喺第一個 error 就停）。逐關處理，每關 re-build 驗證：

1. **sherpa-onnx crate 喺 Windows 能否 build** —— CI 已有 `prepare-windows-sherpa.ps1` 預備 static libs，應 OK；若 link 失敗睇嗰個 script。
2. **Windows 版 sherpa "generation" 機制同 macOS 對齊** —— 兩邊都行 `sherpa_native`，`core_adapters.rs` 嘅 Windows/macOS 分支要一致。
3. **其他 Windows-only 代碼**（`windows_ime_*.rs`、Foundry runtime、`windows-*.ps1`）rebrand（binary 名 openless→mudala）後有冇 break —— 注意 rebrand 已改 binary 名，`windows-package-*.ps1` / `ci.yml` 寫死 binary 名嘅地方要跟改（見 rebrand 遺留 cleanup 地圖）。

## 6. 驗收標準
1. `release-tauri.yml` Windows job 全綠 → 出到 `.msi` / `.exe`。
2. Windows 版開 app = 「唔打啦」，冇 OpenLess 殘留。
3. 本地 SenseVoice 廣東話引擎喺 Windows 行到（模型下載 + ASR）。
4. 港式糾正規則 + 口語/書面切換 + offline-lock 喺 Windows 生效。
5. 真 Windows 機測試（如 Alex 有 Windows 機；冇就靠 CI `windows-*.ps1` smoke test）。

## 7. Non-goals（唔做）
- 唔改 macOS 版（macOS 已 100% 完成 + universal binary build 到，**勿碰 macOS 相關 code**）。
- 唔加新功能、唔做 iOS/Android。
- 唔碰 Apple 公證流程（等 SMS，獨立進行）。
- 唔搞 macOS Intel CI 嘅 `bundle_dmg.sh` 失敗（另計，universal binary 已喺 Alex Mac build 到）。

## 8. 相關檔案
- `openless-all/app/src-tauri/src/build_target.rs`（**首要 fix**）
- `openless-all/app/src-tauri/build.rs`（sherpa_native 判定）
- `openless-all/app/src-tauri/src/core_adapters.rs`（6 error 位置）
- `openless-all/app/src-tauri/src/asr/local/sherpa_runtime.rs`（sherpa runtime，`#[cfg(sherpa_native)]`）
- `.github/workflows/release-tauri.yml` + `ci.yml`（Windows CI）
