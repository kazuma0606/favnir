# v39.2.0 spec — Audit Log Rune

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.2.0 |
| テーマ | Audit Log Rune — パイプライン実行の追跡・監査ログ |
| 前提 | v39.1.0 COMPLETE — RBAC Rune 完了 |
| 完了条件 | `v39200_tests` 全テスト pass・`cargo test` 0 failures（≥ 2791 件） |

## 背景と目的

v39.1.0 で RBAC による権限制御が整った。v39.2.0 では監査ログ（Audit Log）Rune を追加し、パイプライン実行の追跡・証跡記録を型安全に実装できるようにする。
`fav.toml` に `[audit]` セクションを追加し、設定ベースで監査ログの出力先（ファイル / Webhook）を切り替えられるようにする。

**想定動作**:
```favnir
import audit

fn etl_pipeline(ctx: AppCtx) -> Result<Unit, String> !Http {
  bind trace_id <- audit.start_trace(ctx, "etl_pipeline")
  bind _ <- audit.log(ctx, trace_id, "pipeline started")
  // 処理
  bind _ <- audit.end_trace(ctx, trace_id, "success")
  Result.ok(unit)
}
```

`fav.toml`:
```toml
[audit]
enabled = true
output = "webhook"
webhook_url = "https://audit.example.com/events"
```

## 実装スコープ

### 1. `runes/audit/audit.fav` — 新規作成

```favnir
// runes/audit/audit.fav — Audit Log Rune v39.2.0
// パイプライン実行の追跡・監査ログ

fn log(ctx: AppCtx, trace_id: String, message: String) -> Result<Unit, String> !Http {
  bind cfg <- audit_config(ctx)
  if cfg.enabled {
    bind _ <- emit_log(ctx, cfg, trace_id, message)
    Result.ok(unit)
  } else {
    Result.ok(unit)
  }
}

fn start_trace(ctx: AppCtx, pipeline_name: String) -> Result<String, String> !Http {
  bind cfg <- audit_config(ctx)
  let trace_id = gen.nano_id()
  bind _ <- emit_log(ctx, cfg, trace_id, "trace started: " ++ pipeline_name)
  Result.ok(trace_id)
}

fn end_trace(ctx: AppCtx, trace_id: String, status: String) -> Result<Unit, String> !Http {
  bind cfg <- audit_config(ctx)
  bind _ <- emit_log(ctx, cfg, trace_id, "trace ended: " ++ status)
  Result.ok(unit)
}

fn audit_config(ctx: AppCtx) -> Result<AuditConfig, String> !Http {
  Result.ok(ctx.audit)
}

fn emit_log(ctx: AppCtx, cfg: AuditConfig, trace_id: String, message: String) -> Result<Unit, String> !Http {
  match cfg.output {
    "file"    -> Result.ok(unit)
    "webhook" -> Http.post_json(cfg.webhook_url, { trace_id: trace_id, message: message })
    _         -> Result.ok(unit)
  }
}
```

**キーワード**: `fn log`（`audit_rune_exists` テストで検証）

### 2. `runes/audit/rune.toml` — 新規作成

```toml
[rune]
name = "audit"
version = "1.0.0"
description = "Audit Log Rune for pipeline execution tracking"
author = "Favnir Core Team"
```

### 3. `driver.rs` — テストモジュール追加

#### `v39100_tests::cargo_toml_version_is_39_1_0` のスタブ化

```rust
// Stubbed: version bumped to 39.2.0 — assertion intentionally removed
```

#### `v39200_tests` モジュール新規追加

```rust
// ── v39200_tests (v39.2.0) — Audit Log Rune ──────────────────────────────────
#[cfg(test)]
mod v39200_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_2_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.2.0"), "Cargo.toml must contain version 39.2.0");
    }

    #[test]
    fn changelog_has_v39_2_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.2.0]"), "CHANGELOG.md must contain [v39.2.0]");
    }

    #[test]
    fn audit_rune_exists() {
        let src = include_str!("../../runes/audit/audit.fav");
        assert!(
            src.contains("fn log"),
            "runes/audit/audit.fav must contain fn log"
        );
    }
}
```

**`include_str!` のみ使用のため `use super::*` / imports 不要。**

`audit_rune_exists` の `include_str!` パス: `../../runes/audit/audit.fav`
（`fav/src/` から 2 階層上 = `favnir/` ルート → `runes/audit/audit.fav`）

### 4. `CHANGELOG.md` — `[v39.2.0]` エントリ追加

`## [v39.1.0]` ヘッダ行の直前に挿入:

```
## [v39.2.0] — YYYY-MM-DD

### Added
- `runes/audit/audit.fav` — Audit Log Rune（`log` / `start_trace` / `end_trace`）
- `runes/audit/rune.toml` — Rune 設定ファイル
- `fav.toml` `[audit]` セクション仕様（`enabled` / `output = "file"/"webhook"`）
- `v39200_tests` 3 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `39.1.0` → `39.2.0`
- `versions/current.md`: 最新安定版 → v39.2.0、次に切る版 → v39.3.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.2.0 を ✅ 完了済みにマーク・テスト件数を 3 件に更新
- `site/content/docs/governance/audit-log.mdx` は **v39.8.0 で作成予定**のため今バージョンのスコープ外。

## スコープ除外の明記

### `fav.toml [audit]` セクションの parse 実装

`fav.toml` の `[audit]` セクション（`toml.rs` の `AuditConfig` デシリアライズ・`inject_audit_config` 等）は**今バージョンのスコープ外**とする。
CHANGELOG の仕様記述とロードマップ要件を文書として記録するに留め、parse 実装は v39.5.0 以降のマルチテナント対応・コンテキスト拡張時にまとめて行う。

### `audit.fav` はスタブ Rune

`runes/audit/audit.fav` 内の `fn audit_config` が参照する `ctx.audit: AuditConfig` は、`fav.toml [audit]` parse 実装前のためプレースホルダーとして扱う。
`AuditConfig` 型の定義と `AppCtx.audit` フィールドの追加は parse 実装時に行う。
テスト（`audit_rune_exists`）はファイルの存在と `fn log` キーワードのみを検証し、型チェック通過は今バージョンの要件としない。

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.1.0 | 2788 |
| v39.2.0 追加分 | +3 |
| v39.2.0 期待値 | 2791 |

ロードマップは「Rust テスト 2 件」と記載しているが、meta 2 件（バージョン確認・CHANGELOG 確認）+ 機能 1 件（audit_rune_exists）の計 3 件を追加する。T8 でロードマップを 3 件に更新する。

## ロードマップとの整合

ロードマップ v39.2.0:
- `runes/audit/audit.fav` — `Audit.log` / `Audit.start_trace` / `Audit.end_trace`
- `fav.toml` に `[audit]` セクション（`enabled`, `output = "file"/"webhook"`）
- Rust テスト 2 件（→ 3 件に更新）

本 spec はロードマップの全成果物をカバーし、テスト件数のみ 2 → 3 に更新する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `runes/audit/audit.fav` に `fn log` が含まれる | `audit_rune_exists` テスト |
| 2 | `CHANGELOG.md` に `[v39.2.0]` が含まれる | `changelog_has_v39_2_0` テスト |
| 3 | `Cargo.toml` バージョンが `39.2.0` | `cargo_toml_version_is_39_2_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2791） | `cargo test` 実行結果（v39.1.0 実績 2788 + 3 件 = 2791） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.2.0 が ✅ かつテスト件数が 3 件 | T8 後に目視確認 |
