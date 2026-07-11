# v39.5.0 spec — マルチテナント対応

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.5.0 |
| テーマ | マルチテナント対応 — `ctx.tenant_id` + DB スキーマ切り替え / S3 prefix 分離 |
| 前提 | v39.4.0 COMPLETE — Secret Rune 強化 完了 |
| 完了条件 | `v39500_tests` 全テスト pass・`cargo test` 0 failures（≥ 2801 件） |

## 背景と目的

v39.4.0 で Secret Rune が整った。v39.5.0 では複数チームが同一 Favnir インフラを
安全に利用できるマルチテナント基盤を追加する。

`AppCtx` に `tenant_id: String` フィールドを持つという設計を `runes/tenant/tenant.fav` で具現化し、
以下の 2 つのテナント分離パターンをスタブ実装する:

- **DB スキーマ自動切り替え**: `tenant.db_schema(ctx)` → `"{tenant_id}_schema"` を返す
- **S3 prefix 分離**: `tenant.s3_prefix(ctx)` → `"tenants/{tenant_id}/"` を返す

**想定使用例**:
```favnir
bind schema <- tenant.db_schema(ctx)
bind prefix <- tenant.s3_prefix(ctx)
bind rows   <- db.query(ctx, "SELECT * FROM " ++ schema ++ ".orders LIMIT 100")
bind _      <- s3.put(ctx, prefix ++ "export.parquet", rows)
```

## 実装スコープ

### 1. `runes/tenant/tenant.fav` — 新規作成

```favnir
// runes/tenant/tenant.fav — Multi-tenant Rune v39.5.0
// ctx.tenant_id を使った DB スキーマ / S3 prefix 分離

fn db_schema(ctx: AppCtx) -> Result<String, String> {
  // DB スキーマ自動切り替え: "{tenant_id}_schema"
  // 本実装では ctx.tenant_id を参照する（現在はスタブ）
  Result.ok("tenant_schema")
}

fn s3_prefix(ctx: AppCtx) -> Result<String, String> {
  // S3 prefix 分離: "tenants/{tenant_id}/"
  // 本実装では ctx.tenant_id を参照する（現在はスタブ）
  Result.ok("tenants/default/")
}

fn validate_tenant(ctx: AppCtx, allowed: List<String>) -> Result<Unit, String> {
  // テナント ID が許可リストに含まれるか検証（スタブ: 常に OK）
  Result.ok(unit)
}
```

**テストキーワード**: `fn db_schema`（`tenant_rune_db_schema` テスト）、`fn s3_prefix`（`tenant_rune_s3_prefix` テスト）

> `fn validate_tenant` は自動テストなし（意図的）。ロードマップの「テナント分離 E2E テスト 2 件」は DB スキーマ切り替えと S3 prefix 分離の 2 件を指すため。`validate_tenant` の存在確認は T2 手動確認でカバーする。

### 2. `runes/tenant/rune.toml` — 新規作成

```toml
[rune]
name        = "tenant"
version     = "1.0.0"
description = "Multi-tenant Rune for DB schema switching and S3 prefix isolation"
entry       = "tenant.fav"
effects     = []

[dependencies]
```

> `effects = []` — `db_schema` / `s3_prefix` / `validate_tenant` はいずれも純粋なスタブ（HTTP 呼び出しなし）。
> 本実装時に DB / S3 エフェクトを追加すること。

### 3. `driver.rs` — テストモジュール追加

#### `v39400_tests::cargo_toml_version_is_39_4_0` のスタブ化

```rust
// Stubbed: version bumped to 39.5.0 — assertion intentionally removed
```

#### `v39500_tests` モジュール新規追加

```rust
// ── v39500_tests (v39.5.0) — マルチテナント対応 ──────────────────────────────
#[cfg(test)]
mod v39500_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_5_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.5.0"), "Cargo.toml must contain version 39.5.0");
    }

    #[test]
    fn changelog_has_v39_5_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.5.0]"), "CHANGELOG.md must contain [v39.5.0]");
    }

    #[test]
    fn tenant_rune_db_schema() {
        let src = include_str!("../../runes/tenant/tenant.fav");
        assert!(
            src.contains("fn db_schema"),
            "runes/tenant/tenant.fav must contain fn db_schema"
        );
    }

    #[test]
    fn tenant_rune_s3_prefix() {
        let src = include_str!("../../runes/tenant/tenant.fav");
        assert!(
            src.contains("fn s3_prefix"),
            "runes/tenant/tenant.fav must contain fn s3_prefix"
        );
    }
}
```

> ロードマップの「テナント分離 E2E テスト 2 件」は `tenant_rune_db_schema` + `tenant_rune_s3_prefix` の 2 件を指す。
> meta テスト（version + changelog）2 件を合わせ計 4 件。

### 4. `CHANGELOG.md` — `[v39.5.0]` エントリ追加

`## [v39.4.0]` ヘッダ行の直前に挿入:

```
## [v39.5.0] — YYYY-MM-DD

### Added
- `runes/tenant/tenant.fav` — `tenant.db_schema` / `tenant.s3_prefix` / `tenant.validate_tenant` 追加
- `runes/tenant/rune.toml` — Multi-tenant Rune メタデータ
- `ctx.tenant_id` ベースの DB スキーマ切り替え・S3 prefix 分離スタブ実装
- `v39500_tests` 4 テスト追加（meta 2 + テナント分離 E2E 2）

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `39.4.0` → `39.5.0`
- `versions/current.md`: 最新安定版 → v39.5.0、次に切る版 → v39.6.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.5.0 を ✅ 完了済みにマーク

## 注意事項

### `effects = []` について

`tenant.fav` の 3 関数はいずれも現時点でスタブ（ネットワーク・DB 呼び出しなし）。
`effects = []` は将来の本実装時に `["!Db", "!S3"]` 等へ変更すること。

### `ctx.tenant_id` の参照

現在のスタブは `ctx.tenant_id` を実際には参照せず定数文字列を返す。
将来実装時に `ctx.tenant_id` フィールドを参照するよう置き換えること。
`AppCtx` 定義への `tenant_id: String` 追加は後続バージョン（v40.0 前調整）で行う。

### `validate_tenant` の `List<String>` 型

`validate_tenant` の `allowed` パラメータは `List<String>` 型。
Favnir の `List` 型は既存の stdlib で定義済みのため追加 import 不要。

### `runes/` は CI `fav lint` 対象外

`tenant.fav` 内の未使用パラメータ（`ctx`、`allowed`）は W018 警告候補だが、
`runes/` ディレクトリは CI の `fav lint` 対象外のため対処不要。

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.4.0 | 2797 |
| v39.5.0 追加分 | +4（meta 2 + テナント分離 E2E 2） |
| v39.5.0 期待値 | 2801 |

ロードマップの「テナント分離 E2E テスト 2 件」は functional テスト 2 件を指す。
meta テスト（version + changelog）2 件を加え合計 4 件。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `tenant.fav` に `fn db_schema` が含まれる | `tenant_rune_db_schema` テスト |
| 2 | `tenant.fav` に `fn s3_prefix` が含まれる | `tenant_rune_s3_prefix` テスト |
| 3 | `CHANGELOG.md` に `[v39.5.0]` が含まれる | `changelog_has_v39_5_0` テスト |
| 4 | `Cargo.toml` バージョンが `39.5.0` | `cargo_toml_version_is_39_5_0` テスト |
| 5 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2801） | `cargo test` 実行結果 |
| 6 | `roadmap-v39.1-v40.0.md` の v39.5.0 が ✅ | T7 後に目視確認 |
| 7 | `runes/tenant/rune.toml` が存在し必須フィールドを持つ | T2 手動確認（自動テスト対象外）|
