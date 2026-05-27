# Favnir v6.9.0 Spec — OSS 公開準備

作成日: 2026-05-27

## テーマ

GitHub Public 化に向けた最終整備。

## 現状

| 項目 | 現状 |
|------|------|
| `.github/workflows/ci.yml` | `cargo build` + `cargo test` + `site build` あり |
| `cargo clippy` lint | CI に未追加 |
| `LICENSE` | リポジトリルートになし（`fav/target/doc` 内のみ） |
| `CONTRIBUTING.md` | なし |
| `CHANGELOG.md` | なし |
| GitHub リポジトリ公開設定 | Private（手動対応） |

## スコープ

### コード・ファイル変更

| 作業 | ファイル | 内容 |
|------|---------|------|
| LICENSE 配置 | `LICENSE` | MIT ライセンステキスト（ルートに配置） |
| CONTRIBUTING.md 作成 | `CONTRIBUTING.md` | ビルド手順・テスト手順・PR ガイドライン |
| CHANGELOG.md 作成 | `CHANGELOG.md` | v4.0.0〜v6.9.0 のサマリー |
| CI に clippy 追加 | `.github/workflows/ci.yml` | `cargo clippy -- -D warnings` を rust ジョブに追加 |

### 手動対応（Claude 対応外）

- GitHub リポジトリを Public に変更
- 発表準備（ブログ下書き・connpass LT 登録）

## 完了条件

- `LICENSE`（MIT）がリポジトリルートに存在する
- `CONTRIBUTING.md` にビルド手順・テスト手順・PR ガイドラインが記載されている
- `CHANGELOG.md` に v4.0.0〜v6.9.0 のサマリーが記載されている
- CI の rust ジョブに `cargo clippy` が追加されている
- 既存の `cargo test` が引き続き通る（1043 件）
