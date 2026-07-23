# Plan: v48.3.0 — `fav.toml [runes]` 解決ロジック

## 変更ファイル一覧

| ファイル | 変更内容 |
|---|---|
| `fav/src/toml.rs` | `FavToml.runes: HashMap<String, String>` フィールド追加・`parse_fav_toml` の `"runes"` アーム更新 |
| `fav/src/error_catalog.rs` | E0417 `ErrorEntry` 追加 |
| `fav/src/driver.rs` | `v483000_tests` モジュール追加（2テスト） |
| `fav/Cargo.toml` | version → `"48.3.0"` |
| `CHANGELOG.md` | v48.3.0 エントリ追加 |
| `versions/current.md` | v48.3.0 に更新、進行中 v48.4.0 |
| `versions/v45-v50/v48.3.0/tasks.md` | COMPLETE に更新 |

---

## 変更詳細

### Step 1: `toml.rs` — `FavToml` struct に `runes` フィールド追加

**対象行**: `FavToml` struct の末尾（`stream: Option<StreamConfig>` の直後、行 312 付近）

```rust
/// Optional stream configuration (v40.5.0).
pub stream: Option<StreamConfig>,
/// Package dependencies declared in `[runes]` (v48.3.0).
/// Maps rune name → version string (e.g., `"kafka" → "2.1.0"`).
pub runes: std::collections::HashMap<String, String>,
```

### Step 2: `toml.rs` — `parse_fav_toml` のローカル変数追加

**対象行**: `stream_cfg` 変数宣言の直後（行 389 付近）

```rust
let mut stream_cfg: Option<StreamConfig> = None;
let mut runes_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
```

### Step 3: `toml.rs` — `"runes"` アームを更新

**変更前**（行 505〜511）:

```rust
"runes" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        if key == "path" {
            runes_path = Some(val.to_string());
        }
    }
}
```

**変更後**:

```rust
"runes" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        if key == "path" {
            runes_path = Some(val.to_string());
        } else {
            // v48.3.0: rune name → version (e.g. kafka = "2.1.0")
            runes_map.insert(key.to_string(), val.to_string());
        }
    }
}
```

### Step 4: `toml.rs` — `FavToml { ... }` 構造体初期化に `runes` 追加

**対象行**: `FavToml { ... }` ブロック（行 855〜884）の末尾フィールドの後

```rust
    stream: stream_cfg,
    runes: runes_map,   // v48.3.0 追加
}
```

### Step 5: `cargo build` でコンパイルエラー確認（6 箇所必須修正）

`FavToml` にフィールドを追加したため、`FavToml { ... }` を直接構築している以下の 6 箇所で **必ず** コンパイルエラーが発生する。各箇所の末尾フィールド `stream: None,` の直後に `runes: std::collections::HashMap::new(),` を追記する。

| ファイル | 行（目安） |
|---|---|
| `fav/src/driver.rs` | 4623 付近 |
| `fav/src/middle/resolver.rs` | 348 付近 |
| `fav/src/middle/resolver.rs` | 444 付近 |
| `fav/src/middle/resolver.rs` | 556 付近 |
| `fav/src/middle/checker.rs` | 8518 付近 |
| `fav/src/middle/checker.rs` | 8614 付近 |

`cargo build 2>&1 | grep "^error"` でエラーが残っていないことを確認してから次ステップへ。

### Step 6: `error_catalog.rs` — E0417 追加

**対象箇所**: E0416 エントリの直後（行 631 付近）の予約コメント:

```
// ── E0417〜E0419: 予約（将来拡張用） ─────────────────────────────────────────
```

このコメントを以下のエントリに差し替える:

```rust
ErrorEntry {
    code: "E0417",
    title: "package not declared in [runes]",
    category: "imports",
    description: "A bare `import <name>` was used, but `<name>` is not listed in the \
                  `[runes]` table of `fav.toml`. All package imports must be declared \
                  with their version.",
    example: "// fav.toml has no [runes] entry for \"unknown\"\nimport unknown  // E0417",
    fix: "Add `unknown = \"<version>\"` to the `[runes]` table in `fav.toml`.",
    suggestion: Some("Add `<name> = \"<version>\"` to `[runes]` in fav.toml."),
},
// ── E0418〜E0419: 予約（将来拡張用） ─────────────────────────────────────────
```

### Step 7: `driver.rs` — `v483000_tests` 追加

挿入位置: `v482000_tests` の直前。

```rust
// -- v483000_tests (v48.3.0) -- fav.toml [runes] 解決ロジック --
#[cfg(test)]
mod v483000_tests {
    use crate::toml::parse_fav_toml_pub;
    use crate::error_catalog::ERROR_CATALOG;

    #[test]
    fn rune_resolution_from_toml() {
        let content = "[project]\nname = \"myapp\"\nversion = \"0.1.0\"\n[runes]\nkafka = \"2.1.0\"\npostgres = \"1.0.0\"\n";
        let toml = parse_fav_toml_pub(content);
        assert_eq!(toml.runes.get("kafka"), Some(&"2.1.0".to_string()),
            "kafka should be resolved from [runes]");
        assert_eq!(toml.runes.get("postgres"), Some(&"1.0.0".to_string()),
            "postgres should be resolved from [runes]");
    }

    #[test]
    fn e0417_rune_not_in_toml() {
        let found = ERROR_CATALOG.iter().any(|e| e.code == "E0417");
        assert!(found, "E0417 must be registered in ERROR_CATALOG");
    }
}
```

### Step 8: `Cargo.toml` version → `"48.3.0"`

### Step 9: `CHANGELOG.md` 更新

```markdown
## [v48.3.0] — 2026-07-18 — `fav.toml [runes]` 解決ロジック

### Added
- `toml.rs`: `FavToml.runes: HashMap<String, String>` フィールド追加（`[runes]` テーブル全 kv を収集）
- `error_catalog.rs`: E0417（`package not declared in [runes]`）正式追加
- `driver.rs`: `v483000_tests` 追加（`rune_resolution_from_toml` / `e0417_rune_not_in_toml` 2テスト）

### Changed
- `Cargo.toml` version: `48.2.0` → `48.3.0`
```

---

## 実装順序

1. `toml.rs` — `FavToml` struct に `runes` フィールド追加
2. `toml.rs` — `parse_fav_toml` に `runes_map` 変数追加
3. `toml.rs` — `"runes"` アームを更新（全 kv を `runes_map` に収集）
4. `toml.rs` — `FavToml { ... }` に `runes: runes_map` 追加
5. `cargo build` でコンパイルエラー確認 → `FavToml { ... }` を直接構築する箇所に `runes: HashMap::new()` 追記
6. `error_catalog.rs` — E0417 `ErrorEntry` 追加
7. `driver.rs` — `v483000_tests` を `v482000_tests` 直前に追加
8. `Cargo.toml` version → `"48.3.0"`
9. `CHANGELOG.md` v48.3.0 エントリ追加
10. `cargo test` で 3051 passed, 0 failed を確認
11. `cargo clippy -- -D warnings` クリーン確認
12. `versions/current.md` 更新（v48.3.0、次 v48.4.0）
13. `versions/roadmap/roadmap-v48.1-v49.0.md` の v48.3.0 完了条件テスト数（3051）を実績として記入
14. `tasks.md` COMPLETE に更新

---

## 注意事項

- `HashMap` はフルパス `std::collections::HashMap` で記述（`use` 不要）。
- `"runes"` アームで `path` キーは従来通り `runes_path` に設定し `runes_map` には入れない（`else` 分岐で排除済み）。テストで `contains_key("path") == false` を確認すること。
- `[rune]`（単数形）と `[runes]`（複数形）は別物。v48.3.0 は `[runes]`（複数形）のみ対象。
- E0417 の実際の発行ロジック（`checker.rs` で `ImportKind::Package` と `FavToml.runes` を突き合わせる）は v48.5.0 以降のスコープ（`roadmap-v48.1-v49.0.md` に明記済み）。
- Step 5 のコンパイルエラーは必ず 6 箇所発生する（表参照）。「エラーがない場合はスキップ」という判断は誤り。
- `rune_resolution_from_toml` テストは `[project]` セクションを必ず含めること（`name` / `version` フィールドが空文字でも OK だが section が必要）。
