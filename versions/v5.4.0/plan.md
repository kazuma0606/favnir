# v5.4.0 Plan — `rune_modules/` Import Resolution

---

## Phase A: `resolver.rs` 変更

### `resolve_rune_import_file` の拡張

現在のコード:
```rust
pub fn resolve_rune_import_file(&self, import_path: &str) -> Option<PathBuf> {
    let root = self.root.as_ref()?;          // ← root なし = None を返す（standalone 非対応）
    let base = self.toml.as_ref()
        .map(|t| t.runes_dir(root))
        .unwrap_or_else(|| root.join("runes"));
    // ...
}
```

変更後:
1. `rune_modules_base` を決定:
   - `root` がある場合: `root/rune_modules/`
   - `root` がない場合: `std::env::current_dir().ok()?.join("rune_modules/")`
2. `rune_modules_base/<name>/` が存在する場合:
   - `rune_modules_base/<name>/rune.toml` を読んで `entry` フィールドを取得
   - entry ファイル `rune_modules_base/<name>/<entry>` を返す
   - rune.toml なし or entry 空: `rune_modules_base/<name>/<name>.fav` を返す
3. fallback: 従来の `runes/` → registry の順

### helper 関数追加

```rust
fn read_rune_entry(rune_dir: &Path, name: &str) -> PathBuf {
    // rune_dir/rune.toml の [rune] entry を読んで返す
    // 失敗時: rune_dir/<name>.fav
}
```

---

## Phase B: `driver.rs` 変更

### `load_all_items` の `ImportDecl { is_rune: true }` 分岐

現在:
```rust
let dep_file = if *is_rune {
    let dir = toml.runes_dir(root).join(path);
    if dir.is_dir() {
        dir.join(format!("{path}.fav"))
    } else {
        toml.runes_dir(root).join(format!("{path}.fav"))
    }
} ...
// ↑ (toml, root) が None の場合はこのブロック全体をスキップ
```

変更後:
```rust
let dep_file = if *is_rune {
    // 1. rune_modules/ を最優先チェック
    let rune_modules_base = if let Some(root) = root {
        root.join("rune_modules")
    } else {
        // standalone: ソースファイルのディレクトリから探す
        Path::new(path).parent()
            .unwrap_or(Path::new("."))
            .join("rune_modules")
    };
    let rune_modules_dir = rune_modules_base.join(import_path);
    if rune_modules_dir.is_dir() {
        read_rune_entry(&rune_modules_dir, import_path)
    } else if let (Some(toml), Some(root)) = (toml, root) {
        // 2. 従来の runes/ パス
        let dir = toml.runes_dir(root).join(import_path);
        if dir.is_dir() {
            dir.join(format!("{import_path}.fav"))
        } else {
            toml.runes_dir(root).join(format!("{import_path}.fav"))
        }
    } else {
        // 3. 何も見つからない（エラーは checker が報告）
        PathBuf::from(format!("rune_modules/{}/{}.fav", import_path, import_path))
    }
};
```

**注意**: `(toml, root)` が None のケース（standalone）でも `rune_modules/` を探せるよう、
ブロックを `if let (Some(toml), Some(root))` の外に出す。

---

## Phase C: Standalone Script 対応

`load_and_check_program`（`cmd_run` から呼ばれる）で、`proj` が None のときも
`import` rune を解決できるようにする。

現在: `proj` が None → `load_all_items` が呼ばれない → imports はマージされない。

変更:
```rust
let merged = if let Some((ref toml, ref root)) = proj {
    let items = load_all_items(&path, Some(toml), Some(root));
    // ...
} else if has_rune_imports(&program) {
    // standalone だが rune_modules/ から解決を試みる
    let items = load_all_items(&path, None, None);
    // ...
} else {
    program
};
```

`has_rune_imports(program)` は `ImportDecl { is_rune: true }` が存在するかチェックする関数。

---

## Phase D: `rune.toml` entry 読み取り helper

`driver.rs` と `resolver.rs` 両方から使えるよう、`toml.rs` に追加:

```rust
/// Read the `entry` field from `<rune_dir>/rune.toml`.
/// Falls back to `<name>.fav` if rune.toml absent or entry not set.
pub fn rune_entry_file(rune_dir: &Path, name: &str) -> PathBuf {
    if let Ok(content) = std::fs::read_to_string(rune_dir.join("rune.toml")) {
        for line in content.lines() {
            let t = line.trim();
            if let Some((k, v)) = t.split_once('=') {
                if k.trim() == "entry" {
                    let entry = v.trim().trim_matches('"');
                    if !entry.is_empty() {
                        return rune_dir.join(entry);
                    }
                }
            }
        }
    }
    rune_dir.join(format!("{}.fav", name))
}
```

---

## Phase E: 全 15 Rune 公開

各 Rune ディレクトリで `fav rune publish` を実行:

```bash
for rune in auth aws csv db duckdb env gen grpc http incremental json log parquet stat validate; do
    cd runes/$rune && fav rune publish && cd ../..
done
```

---

## Phase F: サイトドキュメント更新

1. `site/content/docs/rune-cli.mdx` を新規作成
2. サイドバー (`site/lib/docs.ts` の `buildSidebar`) に "Rune CLI" を追加
3. `npm run build` で静的生成確認

---

## テスト追加

### `resolver.rs` のテスト追加

`test_resolve_rune_from_rune_modules`:
- `rune_modules/csv/csv.fav` + `rune_modules/csv/rune.toml` を作成
- `resolve_rune_import_file("csv")` が `rune_modules/csv/csv.fav` を返すことを確認

`test_resolve_rune_entry_from_rune_toml`:
- `rune.toml` の `entry = "main.fav"` を読んで `rune_modules/csv/main.fav` を返すことを確認

`test_resolve_rune_fallback_to_runes_dir`:
- `rune_modules/` に csv がない場合、従来の `runes/csv/csv.fav` にフォールバックすることを確認

### `driver.rs` の統合テスト追加

`test_run_standalone_with_rune_modules`:
- `import csv` を含む standalone スクリプト + `rune_modules/csv/` を用意
- `fav run main.fav` が正常に動作することを確認

---

## 実装順序

1. `toml.rs` に `rune_entry_file()` 追加（他モジュールへの依存なし）
2. `resolver.rs` の `resolve_rune_import_file()` 修正 + テスト
3. `driver.rs` の `load_all_items()` 修正 + テスト
4. `driver.rs` の standalone 対応（`has_rune_imports` + `load_and_check_program` 修正）
5. `cargo test` で全テスト通過確認
6. E2E: `rune install csv && fav run main.fav` の手動確認
7. 全 15 Rune の公開
8. サイトドキュメント更新
