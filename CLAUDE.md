# ALICE-Font — Claude Code 設定

## プロジェクト概要

Parametric metafont renderer - 40-byte parameters to infinite-resolution SDF glyphs

| 項目 | 値 |
|------|-----|
| クレート名 | `alice-font` |
| バージョン | 0.1.0 |
| ライセンス | MIT |
| リポジトリ | `ext-sakamoro/ALICE-Font` |
| テスト数 | 157 |
| clippy (all+pedantic+nursery, all-targets) | 0 warnings |
| features | `std`, `ffi`, `pyo3` |
| Eco-Systemブリッジ | bridge_font.rs |

## バインディング

| ターゲット | ファイル | feature |
|-----------|---------|---------|
| C-ABI (FFI) | `src/ffi.rs` — 28関数 (`aa_font_*`) | `ffi` |
| PyO3 (Python) | `src/python.rs` — 4クラス + 3関数 | `pyo3` |
| Unity C# | `bindings/unity/AliceFont.cs` — 28 DllImport + RAII | — |
| UE5 C++ | `bindings/ue5/AliceFont.h` — 28 extern C + RAII | — |

## コーディングルール

メインCLAUDE.md「Git Commit設定」参照。日本語コミット・コメント、署名禁止、作成者 `Moroya Sakamoto`。

## ALICE 品質基準

ALICE-KARIKARI.md「100/100品質基準」参照。clippy基準: `pedantic+nursery`

| 指標 | 値 |
|------|-----|
| clippy (pedantic+nursery) | 0 warnings |
| テスト数 | 157 |
| fmt | clean |

## Eco-System パイプライン

本クレートはALICE-Eco-Systemの以下のパスで使用:
- Path D (Anime→Font)

## 情報更新ルール

- バージョンアップ時: このCLAUDE.mdのバージョンを更新
- APIの破壊的変更時: ALICE-Eco-Systemブリッジへの影響をメモ
- テスト数/品質の変化時: 品質基準セクションを更新
- 新feature追加時: プロジェクト概要テーブルを更新
