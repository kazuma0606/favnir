# Favnir v6.9.0 Plan — OSS 公開準備

作成日: 2026-05-27

## 実装方針

ファイル追加・CI 更新のみ。Rust ソースコードの変更なし。

## Phase 順序

```
A. LICENSE 配置
B. CONTRIBUTING.md 作成
C. CHANGELOG.md 作成
D. CI に cargo clippy 追加
E. 最終確認
```

---

## Phase A — LICENSE 配置

`LICENSE` をリポジトリルート（`/c/Users/yoshi/favnir/LICENSE`）に作成。

- 著作者: Yoshiki Kazuma
- 年: 2026
- MIT License テキスト

---

## Phase B — CONTRIBUTING.md 作成

対象読者: 外部コントリビュータ（OSS 公開後）。

記載内容:
1. **前提条件** — Rust stable、Node.js 22、wasm-pack
2. **ビルド手順** — `cargo build --release`、`npm run build`
3. **テスト手順** — `cargo test`（1043 件）、`fav check examples/`
4. **PR ガイドライン** — ブランチ命名規則、コミットメッセージ形式、レビュープロセス
5. **Rune 追加ガイド** — VM primitive → Favnir 層の構造

---

## Phase C — CHANGELOG.md 作成

フォーマット: [Keep a Changelog](https://keepachangelog.com/ja/1.0.0/)

バージョンサマリー（新しい順）:

| バージョン | 日付 | テーマ |
|-----------|------|--------|
| v6.9.0 | 2026-05-27 | OSS 公開準備 |
| v6.8.0 | 2026-05-27 | Rune エコシステム補完（db/http docs） |
| v6.6.0 | 2026-05-27 | T.validate 完成 |
| v6.5.0 | 2026-05-27 | サイトドキュメント補完 |
| v6.4.0 | 2026-05-27 | Playground 改善・WASM List 対応 |
| v6.3.0 | 2026-05-26 | Self-host stage/seq |
| v6.2.0 | 2026-05-25 | Bootstrap 検証完了 |
| v6.1.0 | 2026-05-24 | compiler.fav フルインライン化 |
| v6.0.0 | 2026-05-21 | セルフホスト完成 |
| v5.0.0〜v5.5.0 | 2026-05 | AWS 本番稼働・CI/CD |
| v4.1.0〜v4.12.0 | 2025〜2026 | Rune エコシステム構築 |

---

## Phase D — CI に cargo clippy 追加

`.github/workflows/ci.yml` の rust ジョブに以下のステップを追加:

```yaml
- name: Clippy
  working-directory: fav
  run: cargo clippy --locked -- -D warnings
```

`Build` ステップの後・`Test` ステップの前に配置。

---

## Phase E — 最終確認

- `LICENSE` がルートに存在する
- `CONTRIBUTING.md` / `CHANGELOG.md` の内容を目視確認
- CI YAML の構文確認
- tasks.md を完了状態に更新
