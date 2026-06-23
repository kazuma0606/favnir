# v21.8.0 実装計画 — `fav migrate` 強化

## 実装方針

既存の `cmd_migrate` / `migrate_source` / `migrate_effects_in_source` を**最小限の変更で拡張**する。
新機能はすべて既存関数の薄いラッパーとして実装し、既存テストへのリグレッションを防ぐ。

---

## タスク順序

| タスク | 内容 | 依存 |
|---|---|---|
| T1 | `migrate_fav_toml_source` 実装（driver.rs） | なし |
| T2 | `cmd_migrate` 更新（`dry_run`・サマリー）（driver.rs） | なし |
| T3 | `main.rs` CLI フラグ追加（`--from` / `--to` / `--config`） | T1, T2 |
| T4 | Cargo.toml バージョン更新 + `v217000_tests` に `#[ignore]` | なし |
| T5 | `v218000_tests` 追加（driver.rs） | T1, T2 |
| T6 | CHANGELOG + `site/content/docs/cli/migrate.mdx` | T5 |

---

## T1: `migrate_fav_toml_source(src: &str) -> String`

`driver.rs` の `migrate_source` の直後に追加。

```rust
/// Migrate old-format fav.toml to current format.
/// Transforms:
///   [rune_dependencies]  →  [dependencies]
///   rune_version = "..." →  version = "..."
///   rune_path = "..."    →  path = "..."
pub fn migrate_fav_toml_source(src: &str) -> String {
    let mut out = String::new();
    for line in src.lines() {
        let trimmed = line.trim_start();
        if trimmed == "[rune_dependencies]" {
            let indent_len = line.len() - trimmed.len();
            out.push_str(&line[..indent_len]);
            out.push_str("[dependencies]\n");
        } else {
            // Use str::replace to handle both standalone keys and inline tables
            // e.g. `http = { rune_version = "1.0", rune_path = "..." }`
            let modified = line
                .replace("rune_version = ", "version = ")
                .replace("rune_path = ", "path = ");
            out.push_str(&modified);
            out.push('\n');
        }
    }
    // Preserve trailing newline behaviour (same as migrate_source)
    if !src.ends_with('\n') && out.ends_with('\n') {
        out.pop();
    }
    out
}
```

**注意:** `strip_prefix` アプローチは行頭キー（`rune_version = "1.0"`）にしか対応できない。実装は `str::replace` を用いて TOML インラインテーブル（`http = { rune_version = "1.0", rune_path = "..." }`）も処理できる。

**注意:** `migrate_source` と同様、元の末尾改行の有無を保持する。

---

## T2: `cmd_migrate` 更新

### 変更点 1: `_dry_run` → `dry_run`

```rust
pub fn cmd_migrate(
    file: Option<&str>,
    in_place: bool,
    dry_run: bool,     // ← アンダースコア除去
    check: bool,
    dir: Option<&str>,
    from_effects: bool,
    from_version: Option<&str>,   // ← 追加
    to_version: Option<&str>,     // ← 追加
    config_file: Option<&str>,    // ← 追加
) {
```

### 変更点 2: ルーティングヘルパー追加（テスト可能化）

`cmd_migrate` 内に埋め込む前に、ルーティングロジックをヘルパー関数として抽出する（単体テストが可能になる）:

```rust
/// --from/--from-effects フラグから use_effects フラグを解決する。
/// テスト可能化のためにヘルパー関数として抽出。
pub fn resolve_use_effects(from_version: Option<&str>, from_effects: bool) -> bool {
    from_effects || matches!(from_version, Some("v13") | Some("13"))
}
```

### 変更点 3: `--from/--to` によるルーティング

```rust
// --config: fav.toml 移行
if let Some(config_path) = config_file {
    let src = std::fs::read_to_string(config_path)
        .unwrap_or_else(|e| { eprintln!("error: cannot read {}: {}", config_path, e); std::process::exit(1); });
    let migrated = migrate_fav_toml_source(&src);
    if migrated == src {
        println!("fav.toml is already up-to-date.");
    } else if in_place {
        std::fs::write(config_path, &migrated)
            .unwrap_or_else(|e| eprintln!("error: cannot write {}: {}", config_path, e));
        println!("Migrated: {}", config_path);
    } else {
        // --in-place なし: diff 表示のみ（dry-run 相当）
        // .fav ファイルと同じ統一フォーマット（ヘッダー行 + 行番号付き diff）を使う
        println!("--- {}", config_path);
        println!("+++ {} (migrated)", config_path);
        for (i, (old, new)) in src.lines().zip(migrated.lines()).enumerate() {
            if old != new {
                println!(" {:4}: - {}", i + 1, old);
                println!(" {:4}: + {}", i + 1, new);
            }
        }
        println!("(dry-run: use --in-place to apply changes)");
    }
    return;
}

// --from v13 (--to v14 または省略) は --from-effects 相当
let use_effects = resolve_use_effects(from_version, from_effects);
```

**`from_version` の `--from v1` / unknown 値のハンドリング（use_effects が false の場合）:**

```rust
// use_effects == false のとき: --from v1 または --from unknown のハンドリング
if !use_effects {
    if let Some(v) = from_version {
        if !matches!(v, "v1" | "1") {
            eprintln!("warning: unknown --from version '{}', defaulting to v1→v2 migration", v);
        }
    }
}
// → migrate_source() を呼ぶ（既存パス）
```

### 変更点 4: 移行サマリー

```rust
// 末尾に必ず出力
if !check {
    println!(
        "Migration complete: {} file(s) migrated, {} file(s) already up-to-date.",
        changed_count,
        files.len() - changed_count
    );
} else if any_needs_migration {
    println!("{} file(s) need migration.", changed_count); // needs_count は使わず changed_count を流用
} else {
    println!("All files are already up-to-date.");
}
```

既存の `if changed_count == 0 && !check { println!("All files are already...") }` は削除して上記サマリーに統合。

---

## T3: `main.rs` CLI フラグ追加

`Some("migrate")` ブランチに追加:

```rust
let mut from_version: Option<String> = None;
let mut to_version: Option<String> = None;
let mut config_file: Option<String> = None;

// while ループ内に追加（値が欠落した場合は exit(1)、--dir と同じパターン）:
"--from" => {
    from_version = Some(args.get(i + 1).cloned().unwrap_or_else(|| {
        eprintln!("error: --from requires a version argument");
        std::process::exit(1);
    }));
    i += 2;
}
"--to" => {
    to_version = Some(args.get(i + 1).cloned().unwrap_or_else(|| {
        eprintln!("error: --to requires a version argument");
        std::process::exit(1);
    }));
    i += 2;
}
"--config" => {
    config_file = Some(args.get(i + 1).cloned().unwrap_or_else(|| {
        eprintln!("error: --config requires a file path argument");
        std::process::exit(1);
    }));
    i += 2;
}
```

`cmd_migrate` 呼び出しを更新:

```rust
cmd_migrate(
    file.as_deref(), in_place, dry_run, check, dir.as_deref(), from_effects,
    from_version.as_deref(), to_version.as_deref(), config_file.as_deref(),
);
```

**`--from` / `--to` / `--config` の値トークンを while ループで `i += 2` してスキップすること。**
`for a in args` を使うと値トークン（`"v13"`, `"v14"`, `"fav.toml"`）が `file` に誤判定される。

---

## T4: Cargo.toml + `#[ignore]`

```toml
version = "21.8.0"
```

`v217000_tests::version_is_21_7_0` のみに `#[ignore]` を追加。
他の `v217000_tests` テスト（`doc_site_generates_index_html` 等）は引き続き実行する。

---

## T5: `v218000_tests` — 8 件

```rust
mod v218000_tests {
    use super::*;

    fn repo_path(rel: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(rel)
    }

    #[test]
    fn version_is_21_8_0() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains("\"21.8.0\""));
    }

    #[test]
    fn migrate_routing_v13_uses_effects() {
        // resolve_use_effects ヘルパーのルーティングをテスト（HIGH 1 の修正）
        // --from v13 のとき use_effects = true になること
        assert!(resolve_use_effects(Some("v13"), false), "--from v13 → use_effects");
        assert!(resolve_use_effects(Some("13"), false),  "--from 13  → use_effects");
        assert!(resolve_use_effects(None, true),         "--from-effects → use_effects");
        // --from v1 や None のとき use_effects = false になること
        assert!(!resolve_use_effects(Some("v1"), false), "--from v1 → no effects");
        assert!(!resolve_use_effects(None, false),       "no flag   → no effects");
    }

    #[test]
    fn migrate_routing_v1_applies_migrate_source() {
        // --from v1 相当: migrate_source で trf→stage 変換が行われること
        let src = "trf Double: Int -> Int = |n| { n * 2 }";
        let result = migrate_source(src);
        assert!(result.contains("stage Double"), "migrate_source で trf→stage");
        // --from v13 相当: migrate_effects_in_source が呼び出せること
        // 注意: 行末が `{` で終わる fn 定義のみ !Effect を除去する。インラインボディ付きは変換しない。
        // ctx パラメータの自動追加は行わない（W010 警告で手動追加を促す設計）。
        let src2 = "fn load() -> String !Postgres {";
        let (migrated, _warnings) = migrate_effects_in_source(src2);
        assert!(!migrated.contains("!Postgres"), "!Effect 注記が除去されること");
    }

    #[test]
    fn migrate_toml_rune_deps_section() {
        let src = "[rune_dependencies]\nhttp = \"1.0\"\n";
        let out = migrate_fav_toml_source(src);
        assert!(out.contains("[dependencies]"), "section renamed");
        assert!(!out.contains("[rune_dependencies]"), "old section removed");
    }

    #[test]
    fn migrate_toml_rune_version_and_path_keys() {
        // rune_version と rune_path の両方を確認（LOW 9 の修正）
        let src = "[rune_dependencies]\nhttp = { rune_version = \"1.0\", rune_path = \"../http\" }\n";
        let out = migrate_fav_toml_source(src);
        assert!(out.contains("version = \"1.0\""), "rune_version key renamed");
        assert!(!out.contains("rune_version"), "old rune_version key removed");
        assert!(out.contains("path = \"../http\""), "rune_path key renamed");
        assert!(!out.contains("rune_path"), "old rune_path key removed");
    }

    #[test]
    fn migrate_toml_no_change_on_modern() {
        let src = "[dependencies]\nhttp = { version = \"1.0\" }\n";
        let out = migrate_fav_toml_source(src);
        assert_eq!(src, out, "modern format should be unchanged (idempotent)");
        // 末尾改行なしのケースも確認
        let src2 = "[dependencies]\nfoo = \"2.0\"";
        let out2 = migrate_fav_toml_source(src2);
        assert_eq!(src2, out2, "no trailing newline: idempotent");
    }

    #[test]
    fn changelog_has_v21_8_0() {
        let path = repo_path("CHANGELOG.md");
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("[v21.8.0]"));
    }

    #[test]
    fn migrate_mdx_exists() {
        let path = repo_path("site/content/docs/cli/migrate.mdx");
        assert!(path.exists(), "migrate.mdx が見つかりません: {:?}", path);
    }
}
```

---

## T6: CHANGELOG + MDX

### CHANGELOG.md エントリ

```markdown
## [v21.8.0] — 2026-06-20

### Added
- `fav migrate --from v13 --to v14` — バージョン指定による移行種別の明示的選択
- `fav migrate --from v1 --to v2` — trf/flw → stage/seq 移行の明示指定
- `fav migrate --config fav.toml` — `fav.toml` 形式の自動移行
- `migrate_fav_toml_source()` — `[rune_dependencies]`/`rune_version` 等の旧形式を変換
- 移行サマリー出力（`Migration complete: X file(s) migrated, Y file(s) already up-to-date.`）

### Fixed
- `--dry-run` フラグのパラメータ名を `_dry_run` → `dry_run` に修正（明示的に機能するよう）
```

### `site/content/docs/cli/migrate.mdx`

`fav migrate` コマンドリファレンス。各フラグの説明・使用例・移行パスの表を含む。

---

## 実装上の注意点

### `cmd_migrate` シグネチャ変更の影響

`cmd_migrate` は `main.rs` からのみ呼ばれている（`pub fn` だが test からは直接呼ばれていない）。
シグネチャ変更後は `main.rs` の呼び出し箇所を必ず更新する。

`grep -n "cmd_migrate" fav/src/main.rs` で呼び出し箇所を確認してから変更する。

### `--from` なしの後方互換

`from_version` が `None` かつ `from_effects` が `false` の場合 → `migrate_source()`（既存動作）。
`from_version` が `Some("v13")` または `Some("13")` の場合 → `migrate_effects_in_source()`。
`from_version` が `Some("v1")` または `Some("1")` の場合 → `migrate_source()`。
それ以外の `from_version` 値 → `eprintln!("warning: unknown --from version '{}', defaulting to v1→v2 migration", v)` の上で `migrate_source()`。

### `--config` の `--in-place` なし動作

`--config fav.toml` のみ（`--in-place` なし）のとき: diff 表示のみ（書き込まない）。
`--config fav.toml --in-place` のとき: ファイルを直接書き換える。

### `migrate_fav_toml_source` の末尾改行

`migrate_source` と同じロジック:
```rust
if !src.ends_with('\n') && out.ends_with('\n') {
    out.pop();
}
```

---

## リスクと対策

| リスク | 対策 |
|---|---|
| `cmd_migrate` シグネチャ変更で `main.rs` のコンパイルエラー | `cargo check` を T2 完了後すぐに実行して確認 |
| `--from` / `--to` の値トークンが `file` に誤判定される | while ループ内で `i += 2` でスキップ（T3 参照） |
| `migrate_fav_toml_source` が現行 fav.toml を変換してしまう | `migrate_toml_no_change_on_modern` テストで べき等性を確認 |
| 既存 migrate_tests がリグレッション | `cmd_migrate` の既存テストは関数レベルではなく `migrate_line` / `migrate_source` を直接テストしているため影響なし |
