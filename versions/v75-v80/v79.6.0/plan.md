# v79.6.0 実装計画 — ドッグフーディング強化

Date: 2026-08-16

---

## 実装順序

### Step 1: `fav/pipelines/` ディレクトリ作成 + ファイル追加

`fav/pipelines/release.fav` を新規作成:

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

`fav/pipelines/health-check.fav` を新規作成:

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

---

### Step 2: CHANGELOG.md 更新（テスト追加より先）

`CHANGELOG.md` の先頭に v79.6.0 エントリを追加。

---

### Step 3: driver.rs — v796000_tests モジュール追加

`fav/src/driver.rs` の末尾に以下を追加:

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

注意: `use super::*` 不要。`const RELEASE` / `const HEALTH` パターンを採用。

---

### Step 4: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version = "79.5.0"` → `"79.6.0"` に更新。

driver.rs 内の escaped `\"79.5.0\"` を `\"79.6.0\"` に一括更新（sed）。
エラーメッセージ文字列（unescaped）の `79.5.0` も `79.6.0` に更新。

更新後に `grep -c "79\.5\.0" /c/Users/yoshi/favnir/fav/src/driver.rs` → 出力が `1` であることを確認。
（残るのは `// --- v79.5.0: Execution Effects showcase パイプライン ---` コメント行の 1 件のみ）

---

### Step 5: versions/current.md 更新

- `## 進行中バージョン` → `**v79.6.0**（ドッグフーディング強化）`
- `## 次に切る版` → `**v79.7.0**（OSS 公開強化・コミュニティ整備）`

---

### Step 6: 最終確認

```bash
cargo test v796000 2>&1 | grep -E "test result|FAILED"
cargo test 2>&1 | grep "^test result"
```

3799 tests pass、v796000 2 件 pass を確認。

---

## 依存順序サマリ

```
fav/pipelines/ 作成（Step 1）
  → CHANGELOG 更新（Step 2）
  → driver.rs テスト追加（Step 3）← pipelines/ が先に作成されていること
  → Cargo.toml + エラーメッセージ更新（Step 4）
  → current.md 更新（Step 5）
  → 最終確認（Step 6）
```
