# v25.0.0 — Practical Self-Hosting マイルストーン宣言

## テーマ

v24.0〜v24.8 で達成した「すべてのコアコンポーネントを Favnir で実装した」状態を、
**v25.0.0 = v1.0 リリース候補**として正式に宣言する。

> 「Favnir は Rust の力を借りながら、Rust を使わずに Favnir の世界を記述できる」

---

## 達成済みコンポーネント（v24.0 時点）

| コンポーネント | 実装 | 達成バージョン |
|---|---|---|
| コンパイラ（compiler.fav） | Favnir ✓ | v8.5.0〜 |
| 型チェッカー（checker.fav） | Favnir ✓ | v8.1.0〜 |
| CLI（cli.fav） | Favnir ✓ | v7.6.0〜 |
| VM（vm.fav） | Favnir ✓ | v24.0.0 |
| VM エンジン（実行基盤） | Rust（永続・設計上） | — |

---

## 成果物

### T1: `MILESTONE.md` — 宣言ドキュメント（リポジトリルート）

- `"Practical Self-Hosting"` を含む（テスト要件）
- `"compiler.fav"` を含む（テスト要件）
- 達成バージョン一覧（v8.1.0〜v24.0.0）
- Rust を設計上永続維持する旨の説明
- 最終テスト手順（ロードマップ §v25.0 の 5 項目のうち項目 1 のみ達成済み。項目 2〜5 は vm.fav Phase 6 未実装のため v25.x に延期）

### T2: `README.md` 更新

- `"v25.0"` を追記（テスト要件。`v1.0` は任意追記。偽陽性防止のためテストは `"v25.0"` 単独で検証）
- マイルストーン達成バッジ / セクション追加
- インストール手順が v25.0.0 を参照するよう更新

### T3: `site/content/docs/v1-release.mdx` — v1.0 リリースノート

- `"v25.0"` を含む（テスト要件）
- v24.1〜v24.8 の各バージョンで達成した機能一覧
- v1.x 後方互換性保証（STABILITY.md 参照リンク）

### T4: `versions/roadmap-v20.1-v25.0.md` 更新

- v24.1〜v24.8 を「完了」に更新
- v25.0.0 を「宣言済み」に更新
- （`versions/roadmap-master.md` は v17〜v20 用のため対象外）

---

## Rust テスト（v250000_tests、5 件）

v248000_tests に削除対象の `version_is_X` テストがないため削除なし（v247000_tests 以前の `version_is` テストは実装時に削除済み）。5 件を純粋追加する。

| テスト名 | 内容 | 期待値 |
|---|---|---|
| `milestone_md_has_selfhost_declaration` | `MILESTONE.md` に `"Practical Self-Hosting"` + `"compiler.fav"` が含まれる | assert |
| `readme_mentions_v1_release` | `README.md` に `"v25.0"` が含まれる | assert |
| `stability_md_exists` | `STABILITY.md` に `"v1.x"` が含まれる（v24.4.0 で作成済み） | assert |
| `site_v1_release_page_exists` | `site/content/docs/v1-release.mdx` に `"v25.0"` が含まれる | assert |
| `changelog_has_v25_0_0` | `CHANGELOG.md` に `[v25.0.0]` が含まれる | assert |

---

## テスト件数

- 削除: なし（v248000_tests に削除対象の `version_is_X` テストが存在しない）
- 追加: `v250000_tests`（5 件）
- 合計: **1969 + 5 = 1974 件**

---

## スコープ外（v25.x 以降）

以下はロードマップ §v25.0「最終テスト」に記載があるが、vm.fav Phase 6（ユーザー定義関数ディスパッチ）が未実装のため v25.x に延期。
未実装の具体的なオペコード: `CallFn`（ユーザー定義関数の直接ディスパッチ）/ クロージャキャプチャ / 再帰呼び出しスタック管理。

- `fav run --vm=self/vm.fav self/compiler.fav -- hello.fav` の E2E 自動テスト
- `fav run --vm=self/vm.fav self/checker.fav` / `self/cli.fav` の E2E テスト
- 4-stage bootstrap 全 6 fixture（Stage 4 = vm.fav 実行）

ロードマップ §v25.0「最終テスト」5 項目のうち、項目 1（`cargo test` 全件 PASS）のみ v25.0.0 で達成。項目 2〜5 は v25.x に延期。

---

## 完了条件

- [ ] `MILESTONE.md` 作成済み（`"Practical Self-Hosting"` + `"compiler.fav"` 含む）
- [ ] `README.md` に `"v25.0"` を追記済み
- [ ] `site/content/docs/v1-release.mdx` 作成済み（`"v25.0"` 含む）
- [ ] `versions/roadmap-v20.1-v25.0.md` の v24.1〜v25.0 を更新済み
- [ ] `cargo test v250000 --bin fav` — 5/5 PASS
- [ ] `cargo test --bin fav` — リグレッションなし（1974 件合格）
- [ ] `CHANGELOG.md` に v25.0.0 エントリ
- [ ] `benchmarks/v25.0.0.json` 作成済み（test_count: 1974）
