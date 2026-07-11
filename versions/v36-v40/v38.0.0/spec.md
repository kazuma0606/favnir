# v38.0.0 spec — Multi-Source ETL Power マイルストーン宣言

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v38.0.0 |
| テーマ | Multi-Source ETL Power マイルストーン宣言・★クリーンアップ |
| 前提 | v37.9.0 COMPLETE — v38.0 前調整・安定化完了 |
| 完了条件 | `v38000_tests` 全テスト pass・`cargo test` 0 failures・`MILESTONE.md` 更新 |

## 背景と目的

v37.1〜v37.9 のスプリントで以下を達成した。本バージョンはこれらを統合して Multi-Source ETL Power マイルストーンを正式宣言し、v38 世代に移行する。

### 達成内容

| バージョン | 内容 |
|---|---|
| v37.1.0 | 境界付きジェネリクス実用強化（`Serialize` / `Deserialize` 制約）|
| v37.2.0 | 行多相実用強化（ネスト行型 `R with { addr: { city: String, .. } }`）|
| v37.3.0 | `List.join_on` VM ビルトイン（left semi-join）|
| v37.4.0 | `List.fan_out` / `List.fan_in` VM ビルトイン（チャンク分散処理）|
| v37.5.0 | CDC Rune — Debezium JSON イベント処理（`filter_inserts` / `extract_op`）|
| v37.6.0 | `fav explain --lineage --format dot/svg` — リネージグラフ可視化 |
| v37.7.0 | `fav new --template multi-source` — マルチソース ETL プロジェクトテンプレート |
| v37.8.0 | Multi-Source cookbook 5 本（join / CDC / fan-out / generics / lineage）|
| v37.9.0 | v38.0 前調整・安定化（lineage サマリー行・Multi-Source ETL ドキュメント）|

## ロードマップとの差異

ロードマップの完了条件「テスト数 ≥ 2737」は v37.9.0 実績 2737 件で達成済み（本バージョン +4 件で 2741 件）。
ロードマップ記載の「GitHub Issues P1/P2 ラベル付きオープンバグ 0 件」条件は Favnir が OSS 公開前のため GitHub Issues が存在しない。本バージョンでは対象外とする（v36.0 / v37.0 と同規約）。

## 実装スコープ

| ファイル | 変更内容 |
|---|---|
| `MILESTONE.md` | v38.0 Multi-Source ETL Power 宣言セクション追加（先頭に挿入）|
| `README.md` | v38.0 マイルストーン宣言行を追加 |
| `CHANGELOG.md` | `## [v38.0.0]` エントリ追加 |
| `fav/src/driver.rs` | `v37900_tests::cargo_toml_version_is_37_9_0` スタブ化 |
| `fav/src/driver.rs` | `v38000_tests` モジュール（4 件）追加 |
| `fav/Cargo.toml` | バージョン `37.9.0` → `38.0.0` |
| ビルドキャッシュ | `cargo clean`（★クリーンアップ） |
| `versions/v36-v40/v38.0.0/tasks.md` | COMPLETE 更新 |

## v38000_tests の設計

| テスト名 | 検証内容 | `include_str!` パス |
|---|---|---|
| `cargo_toml_version_is_38_0_0` | Cargo.toml に `"38.0.0"` が含まれる | `"../Cargo.toml"` |
| `changelog_has_v38_0_0` | `CHANGELOG.md` に `[v38.0.0]` が含まれる | `"../../CHANGELOG.md"` |
| `milestone_has_multi_source_etl_power` | `MILESTONE.md` に `"Multi-Source ETL Power"` が含まれる | `"../../MILESTONE.md"` |
| `readme_mentions_multi_source_etl` | `README.md` に `"Multi-Source ETL"` が含まれる | `"../../README.md"` |

imports 不要（`include_str!` のみ使用）。

## 宣言文

```
List.join_on で 2 つのリストを型安全に結合し、
List.fan_out / List.fan_in で大規模データを並列処理し、
CDC Rune で Debezium イベントをストリーミング処理できる。
fav explain --lineage でデータフローを DOT/SVG グラフとして可視化し、
fav new --template multi-source でマルチソース ETL プロジェクトを即座に生成できる。

これが Favnir v38.0 — Multi-Source ETL Power の姿である。
```

## MILESTONE.md への追加内容

```
## v38.0.0 — Multi-Source ETL Power（2026-07-09）

> 「`List.join_on` で 2 つのリストを型安全に結合し、
>  `List.fan_out` / `List.fan_in` で大規模データを並列処理し、
>  CDC Rune で Debezium イベントをストリーミング処理できる。
>  `fav explain --lineage` でデータフローを DOT/SVG グラフとして可視化し、
>  `fav new --template multi-source` でマルチソース ETL プロジェクトを即座に生成できる。
>
>  これが Favnir v38.0 — Multi-Source ETL Power の姿である。」

v38.0.0 をもって、Favnir の **Multi-Source ETL Power** を正式に宣言する。

### 達成コンポーネント（v37.1〜v37.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| 境界付きジェネリクス | v37.1 | `T with Serialize/Deserialize` 制約 |
| 行多相実用強化 | v37.2 | ネスト行型 `R with { addr: { city: String, .. } }` |
| List.join_on | v37.3 | left semi-join VM ビルトイン |
| List.fan_out / fan_in | v37.4 | チャンク分散・再集約 VM ビルトイン |
| CDC Rune | v37.5 | Debezium JSON イベント処理 |
| lineage DOT/SVG | v37.6 | `fav explain --lineage --format dot/svg` |
| multi-source テンプレート | v37.7 | `fav new --template multi-source` |
| cookbook 5 本 | v37.8 | join / CDC / fan-out / generics / lineage レシピ |
| 安定化 | v37.9 | lineage サマリー行・Multi-Source ETL ドキュメント |

**宣言日**: 2026-07-09

---
```

挿入位置: `# Favnir Milestones` ヘッダの直後、`## v37.0.0` セクションの直前。

## README.md への追加行

```markdown
**v38.0（2026-07-09）で、[Multi-Source ETL Power](./MILESTONE.md) マイルストーンを宣言しました。**
```

挿入位置: `**v37.0（2026-07-09）で、[Data Quality First]...` 行の直後。

## ★クリーンアップ

v38.0.0 は x.0.0 マイルストーンのため `cargo clean` が必須（v31〜v37 の x.0.0 と同規約）。

**注意**: `cargo clean` により `fav/tmp/hello.fav` が消える可能性がある（v30.0.0 での知見）。
クリーンアップ前後で `fav/tmp/hello.fav` の存在を確認し、消失した場合は以下の内容で復元すること:
```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

T2 の順序: `fav/tmp/hello.fav` 存在確認 → `cargo clean` → `hello.fav` 存在確認 → `cargo test`

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v37.9.0 | 2737 |
| v38.0.0 追加分（v38000_tests 4 件 + v37900_tests スタブ化 0 件変化） | +4 |
| v38.0.0 期待値 | 2741 |

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `MILESTONE.md` に `"Multi-Source ETL Power"` が含まれる | `milestone_has_multi_source_etl_power` テスト |
| 2 | `README.md` に `"Multi-Source ETL"` が含まれる | `readme_mentions_multi_source_etl` テスト |
| 3 | `CHANGELOG.md` に `[v38.0.0]` が含まれる | `changelog_has_v38_0_0` テスト |
| 4 | `Cargo.toml` バージョンが `38.0.0` | `cargo_toml_version_is_38_0_0` テスト |
| 5 | `cargo clean` 実施済み | T2 実行記録 |
| 6 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2741） | `cargo test` 実行結果（2737 + 4 = 2741） |
