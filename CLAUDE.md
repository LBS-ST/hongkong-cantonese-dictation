# CLAUDE.md — 香港廣東話離線聽寫 App

## 項目
fork OpenLess → 香港廣東話離線聽寫（Mac + Windows），SenseVoice yue 引擎，全程離線開源。

## 權威文件
- **SPEC.md** = M1 唯一權威 spec。開工前必讀，照做勿改決定。

## 硬規則（任何改動都要遵守）
1. 全程離線：禁用/隱藏所有雲端 ASR + 雲端 LLM 潤飾，本地 SenseVoice yue 做默認引擎
2. 語言鎖 yue，輸出口語粵字（呢 / 唔 / 嘅），**唔轉書面語**
3. 保留 AGPL-3.0 license + attribution；README 加 SenseVoice 模型（FunASR Model License）attribution
4. M1 唔加新功能，唔碰香港化（M2 先做）
5. 唔做 iOS/Android、唔做雲端、唔做收費

## 開發
- Alex Mac local（Codex / Claude Code 寫 code），唔喺 VPS 跑 dev build
- 默認 branch = `beta`，M1 工作開 feature branch 再合併
