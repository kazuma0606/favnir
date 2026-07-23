# Spec: v48.3.0 — `fav.toml [runes]` 解決ロジック

## 概要

`fav.toml` の `[runes]` テーブルをパースして `FavToml.runes: HashMap<String, String>` に格納する。
`error_catalog.rs` に E0417（パッケージが `[runes]` 未登録）を正式追加する。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/toml.rs` | `FavToml` に `pub runes: HashMap<String, String>` 追加・`parse_fav_toml` の `"runes"` アームで全 kv を `runes_map` に収集 |
| `fav/src/error_catalog.rs` | E0417 `ErrorEntry` 追加（予約コメントを実エントリに差し替え） |
| `fav/src/driver.rs` | `v483000_tests` 追加（2テスト） |
| `fav/Cargo.toml` | version → `"48.3.0"` |
| `CHANGELOG.md` | v48.3.0 エントリ追加 |

---

## 変更詳細

### `toml.rs` — `FavToml` への `runes` フィールド追加

**`FavToml` struct**（`stream: Option<StreamConfig>` の直後に追加）:

```rust
/// Package dependencies declared in `[runes]` (v48.3.0).
/// Maps rune name → version string (e.g., `"kafka" → "2.1.0"`).
pub runes: std::collections::HashMap<String, String>,
```

**`parse_fav_toml` 変数宣言**（`stream_cfg` の直後に追加）:

```rust
let mut runes_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
```

**`"runes"` アームの更新**:

変更前（`path` キーのみ処理）:
```rust
"runes" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        if key == "path" {
            runes_path = Some(val.to_string());
        }
    }
}
```

変更後（全 kv を収集、`path` は従来通り `runes_path` にも設定）:
```rust
"runes" => {
    if let Some((key, val)) = parse_kv(trimmed) {
        if key == "path" {
            runes_path = Some(val.to_string());
        } else {
            // v48.3.0: package name → version map
            runes_map.insert(key.to_string(), val.to_string());
        }
    }
}
```

**`FavToml { ... }` 構造体初期化**（`stream: stream_cfg` の直後に追加）:

```rust
runes: runes_map,
```

---

### `error_catalog.rs` — E0417 追加

E0416 エントリの直後、E0420 の前（現在は予約コメントのみ）に挿入:

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
```

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `rune_resolution_from_toml` | `[runes]` に `kafka = "2.1.0"` がある `fav.toml` 文字列を `parse_fav_toml_pub` でパース → `toml.runes.get("kafka") == Some(&"2.1.0".to_string())` |
| `e0417_rune_not_in_toml` | `ERROR_CATALOG` に `code == "E0417"` のエントリが存在する |

```rust
#[test]
fn rune_resolution_from_toml() {
    use crate::toml::parse_fav_toml_pub;
    let content = "[project]\nname = \"myapp\"\nversion = \"0.1.0\"\n[runes]\nkafka = \"2.1.0\"\npostgres = \"1.0.0\"\n";
    let toml = parse_fav_toml_pub(content);
    assert_eq!(toml.runes.get("kafka"), Some(&"2.1.0".to_string()),
        "kafka should be resolved from [runes]");
    assert_eq!(toml.runes.get("postgres"), Some(&"1.0.0".to_string()),
        "postgres should be resolved from [runes]");
    // `path` キーが runes_map に混入しないことを確認
    assert!(!toml.runes.contains_key("path"),
        "path key must not be collected into runes map");
}

#[test]
fn e0417_rune_not_in_toml() {
    use crate::error_catalog::ERROR_CATALOG;
    let found = ERROR_CATALOG.iter().any(|e| e.code == "E0417");
    assert!(found, "E0417 must be registered in ERROR_CATALOG");
}
```

テスト数: 3049 → **3051**（+2）

---

## 注意事項

- **`FavToml { ... }` 直接構築箇所（6 箇所）にコンパイルエラーが発生する**。`cargo build 2>&1 | grep "^error"` で確認し、各箇所に `runes: std::collections::HashMap::new()` を追記すること:
  - `fav/src/driver.rs` 行 4623 付近
  - `fav/src/middle/resolver.rs` 行 348, 444, 556 付近（3 箇所）
  - `fav/src/middle/checker.rs` 行 8518, 8614 付近（2 箇所）
- 既存の `path = "..."` キーは引き続き `runes_path` に設定する。`runes_map` には入れないこと（`else` ブランチで分岐済み）。テストで `contains_key("path")` が `false` になることを確認すること。
- `HashMap` は `std::collections::HashMap`。`use` は不要（フルパスで記述）。
- `[rune]`（単数形）と `[runes]`（複数形）は別セクション。前者は `rune.toml` 専用・`parse_fav_toml` 内で `"rune"` / `"project"` アームが処理。後者（複数形）が v48.3.0 で拡張する `fav.toml` の依存宣言セクション。誤記は **サイレントに無視** されるため注意。
- E0417 の実際の発行ロジック（`checker.rs` での `ImportKind::Package` × `FavToml.runes` 突き合わせ）は v48.5.0 以降のスコープ。本バージョンは toml パースと error catalog 定義のみ（スコープ分割は `roadmap-v48.1-v49.0.md` に明記済み）。

---

## 完了条件

- `cargo test` 3051 passed, 0 failed（3049 + 2 件）
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.3.0"`
- `CHANGELOG.md` に v48.3.0 エントリ追加
- `versions/current.md` を v48.3.0（3051 tests）に更新、進行中バージョンを `v48.4.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T2 全 `[x]`）
