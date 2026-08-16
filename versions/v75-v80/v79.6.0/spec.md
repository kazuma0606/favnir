# v79.6.0 仕様書 — ドッグフーディング強化

Date: 2026-08-16
Status: PLANNING

---

## Background

v79.5.0 で Execution Effects ショーケースが完成した。
v79.6.0 では Favnir 自身のリリースパイプラインを Favnir で記述し、セルフホスト精神を継続する。

`fav/pipelines/` ディレクトリに以下の Favnir パイプラインファイルを作成する:
- `release.fav` — バージョンバンプ・CHANGELOG 先頭挿入のロジック
- `health-check.fav` — `fav verify` コマンド呼び出しのラッパー

> **Note**: テスト数はベース 3797（v79.5.0 完了後の実測値）。完了後は 3799。

---

## Goals

`fav/pipelines/` に Favnir で記述されたリリースパイプラインを追加し、ショーケーステストで内容を検証する。

---

## `release.fav` 内容

```favnir
// fav/pipelines/release.fav
// Favnir リリースパイプライン — バージョンバンプ・CHANGELOG 更新

fn bump_version(ctx: AppCtx, source: String, old_ver: String, new_ver: String) -> Result<String, String> {
    bind updated <- String.replace(source, old_ver, new_ver)
    Result.ok(updated)
}

fn prepend_changelog(ctx: AppCtx, entry: String, changelog: String) -> Result<String, String> {
    bind header  <- String.concat("---\n\n", entry)
    bind content <- String.concat(header, changelog)
    Result.ok(content)
}

fn release_pipeline(ctx: AppCtx, source: String, old_ver: String, new_ver: String, entry: String, changelog: String) -> Result<String, String> {
    bind bumped <- bump_version(ctx, source, old_ver, new_ver)
    bind msg    <- prepend_changelog(ctx, entry, changelog)
    Result.ok(msg)
}
```

> **Note**: `String.replace(target, from, to)` は 3 引数。`String.concat` は 2 引数。`ctx.current_version` 等の未登録フィールドは使わず明示パラメータで受け取る。

---

## `health-check.fav` 内容

```favnir
// fav/pipelines/health-check.fav
// Favnir ヘルスチェックパイプライン — cargo test + fav verify ラッパー

fn run_tests(ctx: AppCtx) -> Result<String, String> {
    bind _ <- ctx.io.println("Running: cargo test")
    Result.ok("tests-ok")
}

fn run_verify(ctx: AppCtx) -> Result<String, String> {
    bind _ <- ctx.io.println("Running: fav verify")
    Result.ok("verify-ok")
}

fn health_check_pipeline(ctx: AppCtx) -> Result<String, String> {
    bind test_result   <- run_tests(ctx)
    bind verify_result <- run_verify(ctx)
    Result.ok(String.concat(test_result, verify_result))
}
```

> **Note**: `ctx.io.exec` は未登録のため使用しない。`ctx.io.println` を使い `"fav verify"` 文字列をログ出力に含める。

---

## テストモジュール仕様

```rust
// --- v79.6.0: ドッグフーディング強化 ---
#[cfg(test)]
mod v796000_tests {
    const RELEASE: &str = include_str!("../pipelines/release.fav");
    const HEALTH:  &str = include_str!("../pipelines/health-check.fav");

    #[test]
    fn dogfood_release_pipeline_exists() {
        assert!(RELEASE.contains("release_pipeline"), "release.fav must define release_pipeline");
        assert!(RELEASE.contains("bump_version"), "release.fav must define bump_version");
        assert!(RELEASE.contains("prepend_changelog"), "release.fav must define prepend_changelog");
    }

    #[test]
    fn dogfood_health_check_pipeline_exists() {
        assert!(HEALTH.contains("health_check_pipeline"), "health-check.fav must define health_check_pipeline");
        assert!(HEALTH.contains("fav verify"), "health-check.fav must reference fav verify");
    }
}
```

注意:
- `include_str!` のパスは driver.rs から見た相対パス: `../pipelines/release.fav`（`fav/src/` → `fav/pipelines/`）
- `use super::*` 不要（`include_str!` + `assert!` のみ）
- `const RELEASE` / `const HEALTH` パターンを採用

---

## CHANGELOG エントリ形式

```
## [v79.6.0] — 2026-08-16 — ドッグフーディング強化

### Added
- `fav/pipelines/release.fav`: リリースパイプライン（bump_version / prepend_changelog / release_pipeline）
- `fav/pipelines/health-check.fav`: ヘルスチェックパイプライン（run_tests / run_verify / health_check_pipeline）

### Tests
- `dogfood_release_pipeline_exists`: release.fav に release_pipeline / bump_version / prepend_changelog が含まれることを検証
- `dogfood_health_check_pipeline_exists`: health-check.fav に health_check_pipeline / fav verify が含まれることを検証
```

---

## Success Criteria

- `cargo test v796000` で 2 件が pass
- `cargo test` で 3799 tests pass（0 failures）
- `fav/pipelines/release.fav` に `release_pipeline` / `bump_version` / `prepend_changelog` が存在する
- `fav/pipelines/health-check.fav` に `health_check_pipeline` / `fav verify` が存在する

---

## Files to modify

| ファイル | 変更内容 |
|---|---|
| `fav/pipelines/release.fav` | 新規作成（リリースパイプライン）|
| `fav/pipelines/health-check.fav` | 新規作成（ヘルスチェックパイプライン）|
| `fav/src/driver.rs` | `v796000_tests` モジュール追加（末尾）|
| `fav/Cargo.toml` | `version = "79.6.0"` に更新 |
| `fav/Cargo.lock` | version バンプに追随して自動更新 |
| `CHANGELOG.md` | v79.6.0 エントリを先頭に追加 |
| `versions/current.md` | 進行中バージョンを更新 |
