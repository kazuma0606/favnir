# Favnir v6.4.0 Tasks

Date: 2026-05-27

## Goal

Playground を「データエンジニア向けデモ」として機能させる。
WASM ビルドパイプラインを整備し、List / Record 型に対応し、
stage/seq パイプライン例をデフォルトサンプルとして表示する。

## Phase A — WASM ビルドパイプライン整備

- [x] A-1: `fav/` 内の WASM エントリポイント（`crates/favnir-wasm/src/lib.rs`）と `Cargo.toml` を確認
- [x] A-2: `scripts/build-wasm.sh` を作成（wasm-pack build → site/public/wasm/）
- [x] A-3: `scripts/deploy-site.sh` の冒頭で `build-wasm.sh` を呼ぶよう変更
- [x] A-4: `bash scripts/build-wasm.sh` が成功し `site/public/wasm/favnir.js` が更新されること（fav_check + fav_compile 両エクスポート確認済み）

## Phase B — WASM バックエンド: List 型対応

- [x] B-1: `favnir_type_to_wasm_results` / `favnir_type_to_wasm_params` に `Type::List(_) => i32` を追加
- [x] B-2: `wasm_local_for_type` は既存コードで自動対応（`[ValType::I32]` → `Single` ローカル）
- [x] B-3: `List.singleton` — Nil+Cons セルを bump_alloc で生成するヘルパー関数を追加
- [x] B-4: `List.singleton` / `List.length` / `List.is_empty` の inline WASM 実装
- [x] B-5: メモリ `minimum: 1` → `minimum: 2` に修正（ヒープ領域 65536+ を有効化）
- [x] B-6: `cargo test wasm` 53 件通過

## Phase C — WASM バックエンド: Record 型対応

- [x] C-1: スキップ — `Type::Record` は Favnir 型システムに存在しない（Named 型として表現）
- [x] C-2〜C-5: スキップ — v6.5.0 以降に持ち越し

## Phase D — Playground サンプルコード更新

- [x] D-1: `site/app/playground/page.tsx` の `EXAMPLE_CODE` を stage/seq パイプライン例に変更
- [x] D-2: 「非対応」メッセージを `List<Int>`/stage/seq 対応後の状態に更新
- [x] D-3: dev サーバー起動 + JS バンドル確認で stage/seq サンプルコード（Transform/Double/AddOne）の配信を確認済み

## Phase E — テストと検証

- [x] E-1: `cargo test` 1033 件全通過（v6.3.0 から変化なし）
- [x] E-2: このファイルを完了状態に更新

## Recommended execution order

1. Phase A（パイプライン）→ 2. Phase B（List）→ 3. Phase C（Record）→ 4. Phase D（サンプル）→ 5. Phase E（検証）

## 完了条件まとめ

- `scripts/build-wasm.sh` が存在し deploy-site から呼ばれる
- `List<Int>` を使うプログラムが Playground でコンパイル・実行できる
- Record 型を使うプログラムが Playground でコンパイル・実行できる
- Playground のデフォルトサンプルが stage/seq パイプライン例になっている
- `cargo test` 全テスト通過
