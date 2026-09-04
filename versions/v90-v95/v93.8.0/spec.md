# Spec: v93.8.0 — サイトドキュメント更新

## Background

v93.2.0〜v93.7.0 で実装した SAP Metadata Infer 機能（`fav infer --from sap`、型変換関数群、`fav fmt` 適用）に対して、
リファレンスサイトのドキュメントが未作成の状態である。
v93.8.0 では以下 2 ファイルを追加・更新してユーザー向けドキュメントを整備する。

## Goals

1. `site/content/docs/cli/infer.mdx` を**新規作成**する。
   - `fav infer --from sap --metadata <url>` の使い方を記載
   - `fav infer --from sap --metadata-file <path>` の使い方（CI/オフライン向け）を記載
   - ファイルに `sap-metadata` という文字列を含める（テスト要件）
2. `site/content/docs/runes/sap-odata.mdx` を**更新**する。
   - EDM 型 → Favnir 型マッピング表を追加（この追加により `metadata` という語が導入される）
   - `NavigationProperty` と `ExpandClause` の対応表を追加
   - 注意: 既存ファイルに `metadata` は含まれていないため、マッピング表の追加が必須
3. `driver.rs` に `mod v93800_tests`（2 件）を追加し、4,136 tests を達成する。

## Files to Create / Modify

| ファイル | 変更内容 |
|---|---|
| `site/content/docs/cli/infer.mdx` | **新規作成** — `fav infer` CLI ドキュメント |
| `site/content/docs/runes/sap-odata.mdx` | **更新** — EDM 型マッピング表・ExpandClause 対応表を追加 |
| `fav/src/driver.rs` | `mod v93800_tests` を追加（2 テスト） |
| `CHANGELOG.md` | v93.8.0 エントリを追加 |
| `versions/roadmap/roadmap-v93.1-v94.0.md` | 確認のみ（v93.7.0 T6b で `4134 + 2 = 4136` に修正済み・変更不要） |

## Notes

- サイトは MDX ファイルをファイルシステムから自動検出するため、nav 定義への手動登録は不要。

## `infer.mdx` 必須要素

- タイトル: `fav infer`
- `--from sap --metadata <url>` コマンド例
- `--from sap --metadata-file <path>` コマンド例（CI/オフライン向け）
- 文字列 `sap-metadata` を含む（セクション ID またはリンクとして使用）

## `sap-odata.mdx` 追加要素

### EDM 型 → Favnir 型マッピング表

| EDM 型 | Favnir 型 |
|---|---|
| `Edm.String` | `String` |
| `Edm.Int32` / `Edm.Int64` | `Int` |
| `Edm.Decimal` | `Float` |
| `Edm.Boolean` | `Bool` |
| `Edm.DateTime` / `Edm.DateTimeOffset` | `String`（ISO 8601） |
| `Edm.Guid` | `String` |
| その他 | `String`（フォールバック） |

### NavigationProperty → ExpandClause 対応表

| `NavigationProperty` | 生成される Favnir ヘルパー |
|---|---|
| `to_BusinessPartnerAddress` | `business_partner_expand_business_partner_address` |
| `to_Customer` | `business_partner_expand_customer` |

## Success Criteria

- `cargo test 2>&1 | grep "test result"` → `4136 tests, 0 failures`
- `cargo clippy --locked -- -D warnings` → pass
- `./target/debug/fav fmt --check self/compiler.fav` → pass
- `./target/debug/fav fmt --check self/checker.fav` → pass
- `docs_infer_mentions_sap_metadata`: `site/content/docs/cli/infer.mdx` に `sap-metadata` が含まれる
- `docs_sap_odata_mentions_metadata_infer`: `site/content/docs/runes/sap-odata.mdx` に `metadata` が含まれる
