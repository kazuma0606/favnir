# v39.1.0 spec — RBAC Rune

## バージョン概要

| 項目 | 内容 |
|---|---|
| バージョン | v39.1.0 |
| テーマ | RBAC Rune — ロールベースアクセス制御 |
| 前提 | v39.0.0 COMPLETE — Intelligence & Assistance マイルストーン宣言済み |
| 完了条件 | `v39100_tests` 全テスト pass・`cargo test` 0 failures（≥ 2788 件） |

## 背景と目的

v39.0「Intelligence & Assistance」で AI 補助機能が整った。v39.x では「チームで安全に運用できる」Enterprise Governance フェーズに移行する。
v39.1.0 では RBAC（Role-Based Access Control）Rune を追加し、パイプライン実行の権限制御を型安全に実装できるようにする。

**想定動作**:
```favnir
import auth

fn admin_pipeline(ctx: AppCtx) -> Result<Unit, String> !Http {
  bind _ <- auth.require_role(ctx, "admin")
  bind _ <- auth.check_permission(ctx, "write:warehouse")
  // 以降の処理
  Result.ok(unit)
}
```

JWT ベースの認証トークン検証（`auth.verify_jwt`）も提供し、認証・認可の主要ユースケースをカバーする。

## 実装スコープ

### 1. `runes/auth/auth.fav` — 新規作成

```favnir
// runes/auth/auth.fav — RBAC Rune v39.1.0
// ロールベースアクセス制御（RBAC）Rune

fn require_role(ctx: AppCtx, role: String) -> Result<Unit, String> !Http {
  bind token <- ctx.auth.token
  bind payload <- verify_jwt(ctx, token)
  bind roles <- Http.get_json(ctx.auth.roles_url)
  if List.contains(roles, role) {
    Result.ok(unit)
  } else {
    Result.err("Access denied: role '" ++ role ++ "' required")
  }
}

fn check_permission(ctx: AppCtx, permission: String) -> Result<Unit, String> !Http {
  bind token <- ctx.auth.token
  bind payload <- verify_jwt(ctx, token)
  bind perms <- Http.get_json(ctx.auth.permissions_url)
  if List.contains(perms, permission) {
    Result.ok(unit)
  } else {
    Result.err("Access denied: permission '" ++ permission ++ "' required")
  }
}

fn verify_jwt(ctx: AppCtx, token: String) -> Result<String, String> !Http {
  bind resp <- Http.post_json(ctx.auth.verify_url, { token: token })
  Result.ok(resp.subject)
}
```

**キーワード**: `fn require_role`

### 2. `runes/auth/rune.toml` — 新規作成

```toml
[rune]
name = "auth"
version = "1.0.0"
description = "RBAC authentication and authorization Rune for Favnir"
author = "Favnir Core Team"
```

### 3. `driver.rs` — テストモジュール追加

#### `v39000_tests::cargo_toml_version_is_39_0_0` のスタブ化

```rust
// Stubbed: version bumped to 39.1.0 — assertion intentionally removed
```

#### `v39100_tests` モジュール新規追加

```rust
// ── v39100_tests (v39.1.0) — RBAC Rune ───────────────────────────────────────
#[cfg(test)]
mod v39100_tests {
    // include_str! のみ使用のため imports 不要

    #[test]
    fn cargo_toml_version_is_39_1_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("39.1.0"), "Cargo.toml must contain version 39.1.0");
    }

    #[test]
    fn changelog_has_v39_1_0() {
        let src = include_str!("../../CHANGELOG.md");
        assert!(src.contains("[v39.1.0]"), "CHANGELOG.md must contain [v39.1.0]");
    }

    #[test]
    fn auth_rune_exists() {
        let src = include_str!("../../runes/auth/auth.fav");
        assert!(
            src.contains("fn require_role"),
            "runes/auth/auth.fav must contain fn require_role"
        );
    }
}
```

**`include_str!` のみ使用のため `use super::*` / imports 不要。**

`auth_rune_exists` の `include_str!` パス: `../../runes/auth/auth.fav`
（`fav/src/` から 2 階層上 = `favnir/` ルート → `runes/auth/auth.fav`）

### 4. `CHANGELOG.md` — `[v39.1.0]` エントリ追加

`## [v39.0.0]` ヘッダ行の直前に挿入:

```
## [v39.1.0] — YYYY-MM-DD

### Added
- `runes/auth/auth.fav` — RBAC Rune（`require_role` / `check_permission` / `verify_jwt`）
- `runes/auth/rune.toml` — Rune 設定ファイル
- `v39100_tests` 3 テスト追加

---
```

**セパレータは `—`（全角ダッシュ U+2014）**

### 5. その他ドキュメント更新

- `fav/Cargo.toml`: `39.0.0` → `39.1.0`
- `versions/current.md`: 最新安定版 → v39.1.0、次バージョン → v39.2.0
- `versions/roadmap/roadmap-v39.1-v40.0.md`: v39.1.0 を ✅ 完了済みにマーク・テスト件数を 3 件に更新

## テスト数の計算

| バージョン | 実績 |
|---|---|
| v39.0.0 | 2785 |
| v39.1.0 追加分 | +3 |
| v39.1.0 期待値 | 2788 |

ロードマップは「Rust テスト 3 件」と記載しており、meta 2 件（バージョン確認・CHANGELOG 確認）+ 機能 1 件（auth_rune_exists）の計 3 件で一致する。

## ロードマップとの整合

ロードマップ v39.1.0:
- `runes/auth/auth.fav` — require_role / check_permission / verify_jwt
- Rust テスト 3 件

本 spec はロードマップの完了条件をそのまま実装する。

## 完了条件

| # | 条件 | 検証方法 |
|---|---|---|
| 1 | `runes/auth/auth.fav` に `fn require_role` が含まれる | `auth_rune_exists` テスト |
| 2 | `CHANGELOG.md` に `[v39.1.0]` が含まれる | `changelog_has_v39_1_0` テスト |
| 3 | `Cargo.toml` バージョンが `39.1.0` | `cargo_toml_version_is_39_1_0` テスト |
| 4 | `cargo test` 全通過（failures=0 かつテスト数 ≥ 2788） | `cargo test` 実行結果（v39.0.0 実績 2785 + 3 件 = 2788） |
| 5 | `roadmap-v39.1-v40.0.md` の v39.1.0 が ✅ かつテスト件数が 3 件 | T8 後に目視確認 |
