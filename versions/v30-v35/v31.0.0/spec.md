# v31.0.0 仕様書 — Real-World Readiness マイルストーン宣言

## 概要

v30.1〜v30.9 の全コンポーネントが完成したことを受け、
**Real-World Readiness** マイルストーンを正式宣言する。

---

## 背景

ロードマップ v31.0 より:

> **Real-World Readiness の定義（本プロジェクト固有）**
> 「`fav new --template postgres-etl my-project` で生成されたプロジェクトが、
>  `fav check` / `fav run` / `fav test` すべてで通り、
>  実データ（CSV 1000 行）を Postgres に書き込めること」

---

## 達成コンポーネント

| コンポーネント | 完了バージョン | 内容 |
|---|---|---|
| ビルド軽量化 | v30.1.0 | `[profile.dev] debug = 0` で `target/` 削減 |
| postgres-etl テンプレート v2 | v30.2.0 | 4 ファイル構成（types / validators / stages / main） |
| マルチファイル E2E 検証 | v30.3.0 | `fav check` / `fav run` / `fav test` / `fav lint` / `fav fmt` 全通過 |
| Rune import マルチファイル | v30.4.0 | 複数ファイルから同一 Rune import が正常動作 |
| ドッグフードサンプル | v30.5.0 | `examples/csv-to-postgres/` 実装（5 ステージ・README 完備） |
| fav test プロジェクト統合 | v30.6.0 | 引数なし `fav test` で `tests/` + `src/` を一括実行 |
| エラー表示改善 | v30.7.0 | ステージ名・ヒントメッセージ付きランタイムエラー表示 |
| fav new --list | v30.8.0 | 8 テンプレートの一覧表示コマンド |
| ドッグフード修正 | v30.9.0 | `[project]` toml 解析・import 解決・test/new hint |

---

## スコープ

### IN SCOPE

- `fav/Cargo.toml` — version `30.9.0` → `31.0.0`
- `fav/src/driver.rs` — `cargo_toml_version_is_30_9_0` をスタブ化
- `fav/src/driver.rs` — `v310000_tests`（4 件）追加
- `MILESTONE.md` — v31.0.0「Real-World Readiness」セクション追加
- `README.md` — v31.0 マイルストーンの一行を追加
- `CHANGELOG.md` — `[v31.0.0]` セクション追加
- `benchmarks/v31.0.0.json` 新規作成
- `versions/current.md` — v31.0.0 に更新
- **`cargo clean` + `cargo build` + `cargo test`**（マイルストーン版の必須クリーンアップ）

### OUT OF SCOPE

- site/ MDX 更新（次フェーズで実施）
- `fav new --template postgres-etl` + 実 Postgres 接続による手動 E2E（CI 環境非依存）
- `roadmap-v31.1-v32.0.md` の作成（別途作業）

> **cargo clean 注意事項**:
> `cargo clean` を実行すると `fav/tmp/hello.fav` が削除される。
> `bootstrap_c2_artifact_roundtrip` テストはこのファイルに依存するため、
> `cargo clean` 直後に必ず復元すること。
>
> 復元内容:
> ```favnir
> fn add(a: Int, b: Int) -> Int {
>     a + b
> }
>
> fn main() -> Bool {
>     add(1, 2) == 3
> }
> ```

---

## テスト設計（v310000_tests — 4 件）

| # | テスト名 | 確認内容 |
|---|---------|----------|
| 1 | `cargo_toml_version_is_31_0_0` | `Cargo.toml` に `version = "31.0.0"` |
| 2 | `milestone_real_world_readiness_declared` | `MILESTONE.md` に `"Real-World Readiness"` |
| 3 | `readme_mentions_v31_0` | `README.md` に `"v31.0"` |
| 4 | `benchmark_v31_0_0_exists` | `benchmarks/v31.0.0.json` に `"31.0.0"` |

> `v310000_tests` は `use super::*` なし（`include_str!` のみ使用）。

---

## MILESTONE.md 追記内容

```markdown
## v31.0.0 — Real-World Readiness（2026-07-02）

> 「`fav new --template postgres-etl my-project` で生成されたプロジェクトが、
>  `fav check` / `fav run` / `fav test` すべてで通り、
>  実データ（CSV 1000 行）を Postgres に書き込めること」
> = Real-World Readiness の完成を象徴するデモ

v31.0.0 をもって、Favnir の **Real-World Readiness** を正式に宣言する。

`fav new --template postgres-etl` による 4 ファイル構成テンプレート（types / validators / stages / main）が生成され、
`fav check` / `fav test` / `fav lint` の全コマンドが通過する。
`examples/csv-to-postgres/` に CSV 1000 行 → Postgres の実証パイプラインが実装され、
`fav test`（引数なし）がプロジェクト全体のテストを一括実行できるようになった。

### 達成コンポーネント（v30.1〜v30.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| ビルド軽量化 | v30.1 | `[profile.dev] debug = 0` で target/ 削減 |
| postgres-etl テンプレート v2 | v30.2 | 4 ファイル構成・`fav check` 全通過 |
| マルチファイル E2E | v30.3 | 5 コマンド（check/run/test/lint/fmt）全通過 |
| Rune import マルチファイル | v30.4 | 同一 Rune を複数ファイルから import 可能 |
| ドッグフードサンプル | v30.5 | `examples/csv-to-postgres/` 5 ステージ実装 |
| fav test プロジェクト統合 | v30.6 | 引数なし `fav test` でプロジェクト全体実行 |
| エラー表示改善 | v30.7 | ステージ名・ヒント付きランタイムエラー |
| fav new --list | v30.8 | 8 テンプレートの一覧表示 |
| ドッグフード修正 | v30.9 | `[project]` 解析・import 解決・UX hint |

**宣言日**: 2026-07-02
**宣言バージョン**: v31.0.0
```

---

## 完了条件

- `Cargo.toml` version = `"31.0.0"`
- `MILESTONE.md` に `"Real-World Readiness"` セクション
- `README.md` に `"v31.0"` の記述
- `cargo test v310000` — 4/4 PASS
- `cargo test`（`cargo clean` 後）— 全件 PASS（0 failures）
- `CHANGELOG.md` に `[v31.0.0]` セクション
- `benchmarks/v31.0.0.json` 存在
- `versions/current.md` を v31.0.0 に更新
- `tasks.md` が COMPLETE
