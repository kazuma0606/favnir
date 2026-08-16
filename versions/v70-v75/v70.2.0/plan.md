# v70.2.0 Plan — `fav migrate` 完成（構文自動移行ツール）

Date: 2026-08-09
Status: 計画中

---

## 実装ステップ（依存順）

### Step 1: `resolve_use_effects` に `"v35"` 追加

`fav/src/driver.rs` の `resolve_use_effects` を修正:

```rust
pub fn resolve_use_effects(from_version: Option<&str>, from_effects: bool) -> bool {
    from_effects || matches!(from_version, Some("v13") | Some("13") | Some("v35") | Some("35"))
}
```

これで `--from v35` が `migrate_effects_in_line` を起動するようになる。

確認: `cargo test` で既存テストが引き続き pass することを確認。

---

### Step 2: `migrate_io_calls_in_source` 追加

driver.rs に新規関数を追加（`migrate_effects_in_source` の直後）:

```rust
/// Rewrite IO.* stdlib calls to ctx.io.* form.
/// IO.println(x) → ctx.io.println(x)
/// IO.args()     → ctx.io.argv()
/// IO.read_file(p)     → ctx.io.read_file_raw(p)
/// IO.write_file(p, d) → ctx.io.write_file_raw(p, d)
pub fn migrate_io_calls_in_source(src: &str) -> String {
    src.lines()
        .map(|line| migrate_io_calls_in_line(line))
        .collect::<Vec<_>>()
        .join("\n")
        + if src.ends_with('\n') { "\n" } else { "" }
}

fn migrate_io_calls_in_line(line: &str) -> String {
    line
        .replace("IO.write_file(", "ctx.io.write_file_raw(")
        .replace("IO.read_file(", "ctx.io.read_file_raw(")
        .replace("IO.println(", "ctx.io.println(")
        .replace("IO.args()", "ctx.io.argv()")
}
```

**注意**: `IO.write_file` を `IO.read_file` より先に置換する（部分文字列の衝突を避けるため）。

確認: `cargo test` で既存テストが引き続き pass することを確認。

---

### Step 3: `cmd_migrate` で IO コール変換を適用

`cmd_migrate` 内の `use_effects` ブランチで `migrate_effects_in_source` のみが呼ばれている箇所に `migrate_io_calls_in_source` を追加:

```rust
let (migrated, w010s) = if use_effects {
    let (eff_migrated, w010s) = migrate_effects_in_source(&src);
    let io_migrated = migrate_io_calls_in_source(&eff_migrated);
    (io_migrated, w010s)
} else {
    (migrate_source(&src), Vec::new())
};
```

確認: `cargo test` で既存テストが引き続き pass することを確認。

---

### Step 4: `v702000_tests` モジュール追加

driver.rs の末尾に追加:

```rust
#[cfg(test)]
mod v702000_tests {
    #[test]
    fn migrate_effect_annotation_to_ctx() {
        let src = "fn run(x: Int) -> Unit !IO {\n    x\n}\n";
        let (migrated, _) = super::migrate_effects_in_source(src);
        assert!(
            migrated.contains("ctx: AppCtx") || migrated.contains("CommonCtx"),
            "!IO should be converted to ctx param: {}", migrated
        );
        assert!(!migrated.contains("!IO"), "!IO should be removed: {}", migrated);
    }

    #[test]
    fn migrate_io_stdlib_to_ctx_io() {
        let src = "IO.println(msg)\nIO.args()\nIO.read_file(path)\nIO.write_file(path, data)\n";
        let migrated = super::migrate_io_calls_in_source(src);
        assert!(migrated.contains("ctx.io.println(msg)"), "IO.println should be migrated");
        assert!(migrated.contains("ctx.io.argv()"), "IO.args() should be migrated");
        assert!(migrated.contains("ctx.io.read_file_raw(path)"), "IO.read_file should be migrated");
        assert!(migrated.contains("ctx.io.write_file_raw(path, data)"), "IO.write_file should be migrated");
        assert!(!migrated.contains("IO."), "No IO. calls should remain");
    }
}
```

---

### Step 5: CHANGELOG.md 更新

v70.2.0 エントリを v70.1.0 の直前に追加:

```markdown
## [v70.2.0] — 2026-08-09 — fav migrate 完成（構文自動移行ツール）

### Added
- `migrate_io_calls_in_source`: IO.* stdlib コールを `ctx.io.*` 形式に変換
- `resolve_use_effects` に `"v35"` / `"35"` を追加（`--from v35` 対応）
- `v702000_tests`: 2 件追加（3561 → 3563 tests）
  - `migrate_effect_annotation_to_ctx`
  - `migrate_io_stdlib_to_ctx_io`

### Changed
- `cmd_migrate --from v35`: エフェクトアノテーション変換と IO stdlib 変換を同時適用
```

---

### Step 6: 最終確認

- `cargo test v702000` で 2 件 pass
- `cargo test` 全体で 3563 tests pass（0 failures）
- `versions/current.md` を v70.2.0 進行中に更新
- `fav/Cargo.toml` バージョンを `70.1.0` → `70.2.0` に更新
- driver.rs 内の旧バージョン文字列テストを一括更新（`replace_all: true`）
