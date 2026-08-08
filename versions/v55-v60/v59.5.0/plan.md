# v59.5.0 Plan — Migration Toolkit（v1 → Enterprise マイグレーション）

Date: 2026-07-30

---

## 実装順序

### Step 1: Cargo.toml バージョン更新

`fav/Cargo.toml` の `version` を `"59.4.0"` → `"59.5.0"` に変更。

### Step 2: roadmap 更新

`roadmap-v59.1-v60.0.md` に以下を行う:
- v59.6.0 のベース数（「着手時に更新」）を `3318` に確定

（v59.5.0 の実績欄はテスト確認後 T8 で記入）

### Step 3: driver.rs に cmd_migrate_dry_run 追加

`cmd_marketplace_publish` の直後（`v59400_tests` の直前）に追加:

```rust
/// v59.5.0: fav migrate --from v1 --to enterprise --dry-run のガイダンス出力。
pub fn cmd_migrate_dry_run() -> String {
    let sample_src = "import rune \"kafka\"\nstage Parse: Stream<Event> -> Stream<Order> = |e| Ok(e)";
    let mut out = String::from("[analyze] pipeline.fav\n");
    for line in sample_src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("import rune \"") {
            if let Some(rune_name) = rest.strip_suffix('"') {
                out.push_str(&format!(
                    "  [WARN] import rune \"{}\" → import {}  (W035: legacy_import_rune)\n",
                    rune_name, rune_name
                ));
            }
        }
    }
    out.push_str("  [WARN] !Http effect: add TLS config to fav.toml  (new in v57.3)\n");
    out.push_str("  [INFO] No RBAC config detected: add [security.rbac] if needed\n");
    out.push_str("  [INFO] No [env.*] sections: consider multi-env config (v58.6)\n");
    out
}
```

### Step 4: driver.rs に migrate_enterprise_import 追加

`cmd_migrate_dry_run` の直後に追加:

```rust
/// v59.5.0: `import rune "X"` → `import X` の自動変換（W035 auto-fix）。
pub fn migrate_enterprise_import(src: &str) -> String {
    let result = src
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("import rune \"") {
                if let Some(rune_name) = rest.strip_suffix('"') {
                    let indent_len = line.len() - trimmed.len();
                    return format!("{}import {}", &line[..indent_len], rune_name);
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    if src.ends_with('\n') {
        result + "\n"
    } else {
        result
    }
}
```

### Step 5: driver.rs テストモジュール追加

**注意: Step 3・4（関数追加）を必ず先に行うこと。**

`v59500_tests` を `v59400_tests` の直前に挿入:

```rust
// -- v59500_tests (v59.5.0) -- Migration Toolkit --
#[cfg(test)]
mod v59500_tests {
    // NOTE: テスト関数名 cmd_migrate_dry_run が pub fn と同名のため
    //       use super:: は使わず super:: 修飾のみで呼び出す（v59400_tests と同パターン）

    #[test]
    fn cmd_migrate_dry_run() {
        let output = super::cmd_migrate_dry_run();
        assert!(output.contains("[WARN]"), "dry-run should contain [WARN]");
        assert!(output.contains("import rune"), "dry-run should mention legacy import rune");
        assert!(output.contains("RBAC"), "dry-run should mention RBAC");
    }

    #[test]
    fn cmd_migrate_auto_fix_import() {
        let src = "import rune \"kafka\"\nstage Parse: Stream<Event> -> Stream<Order> = |e| Ok(e)";
        let fixed = super::migrate_enterprise_import(src);
        assert!(fixed.contains("import kafka"), "should fix import rune to import kafka");
        assert!(!fixed.contains("import rune \"kafka\""), "should remove legacy import rune syntax");
    }
}
```

### Step 6: driver.rs ローリングチェック更新

既存 7 件を更新（`replace_all` 推奨）:

- `version = \"59.4.0\"` → `version = \"59.5.0\"`（7 件）
- failure メッセージ 7 件を `"59.5.0"` に更新:
  - `"Cargo.toml version should be 59.4.0, got: {}"` → `"59.5.0"`（5 件）
  - `"Cargo.toml version should be 59.4.0 (rolling check from v57.0.0), got: {}"` → `"59.5.0 (rolling check from v57.0.0)"`
  - `"Cargo.toml version should be 59.4.0 (rolling check from v56.9.0), got: {}"` → `"59.5.0 (rolling check from v56.9.0)"`

**注意**: `v59100_tests`〜`v59400_tests` に rolling check はないため更新対象は 7 件。

---

## 注意点

- `strip_suffix('"')` は char リテラルを使う（`strip_suffix("\"")` と等価だが読みやすい）
- `src.ends_with('\n')` でソース末尾の改行を保持すること（`migrate_source` と同様）
- `v59500_tests` には `use super::` を一切書かない。`super::cmd_migrate_dry_run()` / `super::migrate_enterprise_import(...)` と完全修飾する（v59400_tests と同パターン）

---

## 事後処理（Step 7）

- `CHANGELOG.md` に v59.5.0 エントリを追加
- `versions/current.md` を v59.5.0 / 3318 tests に更新
- `versions/roadmap/roadmap-v59.1-v60.0.md` の v59.5.0 実績欄を更新・v59.6.0 ベース数を 3318 に確定
- `versions/v55-v60/v59.5.0/tasks.md` を COMPLETE ステータスに更新
