# Favnir v6.9.0 Tasks

Date: 2026-05-27

## Goal

GitHub Public 化に向けた最終整備。
LICENSE / CONTRIBUTING.md / CHANGELOG.md 作成、CI に clippy 追加。

## Phase A — LICENSE 配置

- [x] A-1: `LICENSE`（MIT）をリポジトリルートに作成

## Phase B — CONTRIBUTING.md 作成

- [x] B-1: `CONTRIBUTING.md` を作成（前提条件・ビルド・テスト手順）
- [x] B-2: PR ガイドライン・ブランチ命名規則を記載
- [x] B-3: Rune 追加ガイド（VM primitive → Favnir 層）を記載

## Phase C — CHANGELOG.md 作成

- [x] C-1: `CHANGELOG.md` を作成（v4.0.0〜v6.9.0 サマリー、新しい順）
- [x] C-2: 各バージョンの Added / Changed / Fixed を簡潔に記載

## Phase D — CI に cargo clippy 追加

- [x] D-1: `.github/workflows/ci.yml` の rust ジョブに `cargo clippy --locked -- -D warnings` を追加

## Phase E — 最終確認

- [x] E-1: `LICENSE` ファイルの存在確認
- [x] E-2: `CONTRIBUTING.md` / `CHANGELOG.md` 内容の目視確認
- [x] E-3: CI YAML 構文確認
- [x] E-4: このファイルを完了状態に更新

---

## 手動対応（コード外）

- [x] GitHub リポジトリを Public に変更
- [x] 発表準備（ブログ下書き・connpass LT 登録）

## 完了条件まとめ

- `LICENSE`（MIT）がリポジトリルートに存在する ✓
- `CONTRIBUTING.md` にビルド手順・PR ガイドライン記載 ✓
- `CHANGELOG.md` に v4.0.0〜v6.9.0 サマリー記載 ✓
- CI rust ジョブに `cargo clippy` 追加 ✓
