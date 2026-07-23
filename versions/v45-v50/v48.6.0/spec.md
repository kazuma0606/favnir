# Spec: v48.6.0 — 循環 import 検出 + E0418

## 概要

import グラフを構築しトポロジカルソートで循環を検出する。
`E0418`（`circular import detected`）を `error_catalog.rs` に追加。
`driver.rs` に `detect_circular_imports` 純粋関数を追加する（MVP: ファイルI/Oなし・グラフ入力を受け取る設計）。

---

## 変更ファイル

| ファイル | 変更内容 |
|---|---|
| `fav/src/error_catalog.rs` | E0418 `ErrorEntry` を追加（予約コメントを差し替え） |
| `fav/src/driver.rs` | `detect_circular_imports` 追加 + `v486000_tests` 追加（2テスト）|
| `fav/Cargo.toml` | version → `"48.6.0"` |
| `CHANGELOG.md` | v48.6.0 エントリ追加 |

---

## 実装詳細

### `error_catalog.rs` — E0418 追加

既存の予約コメントを差し替える:

```
// ── E0418〜E0419: 予約（将来拡張用） ─────────────────────────────────────────
```
↓
```rust
ErrorEntry {
    code: "E0418",
    title: "circular import detected",
    category: "imports",
    description: "A circular dependency was detected in the import graph. \
                  Module `a` transitively imports itself, forming a cycle.",
    example: "// a.fav imports b.fav, b.fav imports a.fav\nimport b  // E0418: circular import detected",
    fix: "Break the cycle by extracting shared definitions into a separate module \
          that neither a.fav nor b.fav imports.",
    suggestion: Some("Extract shared definitions into a third module to break the import cycle."),
},
// ── E0419: 予約（将来拡張用） ─────────────────────────────────────────────────
```

### `driver.rs` — `detect_circular_imports`

```rust
/// import グラフの循環検出（v48.6.0 MVP）。
/// `graph`: モジュール名 → 直接 import するモジュール名の Vec
/// 循環が存在する場合はそのパス（Vec<String>）を返す。なければ None。
/// DFS カラーリング（white=0 / gray=1 / black=2）で検出。
pub fn detect_circular_imports(
    graph: &std::collections::HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    use std::collections::HashMap;
    let mut color: HashMap<&str, u8> = HashMap::new();
    let mut path: Vec<String> = Vec::new();

    fn dfs<'a>(
        node: &'a str,
        graph: &'a HashMap<String, Vec<String>>,
        color: &mut HashMap<&'a str, u8>,
        path: &mut Vec<String>,
    ) -> bool {
        color.insert(node, 1); // gray
        path.push(node.to_string());
        if let Some(neighbors) = graph.get(node) {
            for nb in neighbors {
                let c = *color.get(nb.as_str()).unwrap_or(&0);
                if c == 1 {
                    // cycle found: complete the cycle in path
                    path.push(nb.clone());
                    return true;
                }
                if c == 0 && dfs(nb, graph, color, path) {
                    return true;
                }
            }
        }
        color.insert(node, 2); // black
        path.pop();
        false
    }

    for node in graph.keys() {
        if *color.get(node.as_str()).unwrap_or(&0) == 0 {
            if dfs(node, graph, &mut color, &mut path) {
                return Some(path);
            }
        }
    }
    None
}
```

---

## テスト（+2）

| テスト名 | 内容 |
|---|---|
| `circular_import_e0418` | `{ "a" → ["b"], "b" → ["a"] }` で `detect_circular_imports` を呼ぶと `Some(cycle)` が返り、cycle に `"a"` と `"b"` が含まれる |
| `non_circular_import_ok` | `{ "a" → ["b"], "b" → ["c"], "c" → [] }` で `None` が返る |

```rust
#[test]
fn circular_import_e0418() {
    use crate::driver::detect_circular_imports;
    let mut graph = std::collections::HashMap::new();
    graph.insert("a".to_string(), vec!["b".to_string()]);
    graph.insert("b".to_string(), vec!["a".to_string()]);
    let result = detect_circular_imports(&graph);
    assert!(result.is_some(), "circular import must be detected");
    let cycle = result.unwrap();
    assert!(cycle.contains(&"a".to_string()), "cycle must mention 'a'");
    assert!(cycle.contains(&"b".to_string()), "cycle must mention 'b'");
}

#[test]
fn non_circular_import_ok() {
    use crate::driver::detect_circular_imports;
    let mut graph = std::collections::HashMap::new();
    graph.insert("a".to_string(), vec!["b".to_string()]);
    graph.insert("b".to_string(), vec!["c".to_string()]);
    graph.insert("c".to_string(), vec![]);
    let result = detect_circular_imports(&graph);
    assert!(result.is_none(), "no cycle must return None");
}
```

テスト数: 3056 → **3058**（+2）

---

## 注意事項

- `detect_circular_imports` は **ファイル解決なし** の純粋関数（MVP）。実際のファイルパスから依存グラフを構築するロジックは v48.6.0 のスコープ外。ロードマップ記載の「`driver.rs` の依存グラフ解析ロジックに組み込む」は v48.6.0 MVP スコープ外とすることを合意済み（完全統合は v49.0.0 以降）。
- `pub` 修飾子をつけること（テストから参照するため）。
- 既存の `driver.rs` にはパイプラインステージ向けのカーン法実装（`toposort_stages` 相当）がある。それとは別に import グラフ専用の DFS 実装を追加する。
- `detect_circular_imports` の再帰 DFS はスタックオーバーフローのリスクがあるが、テスト対象は小規模グラフなので MVP では許容する。
- `HashMap::keys()` の列挙順は非決定的なため、返される cycle path の開始ノードは実行ごとに変動する。テストは `cycle.contains(&"a")` 形式で検証すること（順序アサートは禁止）。

---

## 完了条件

- `cargo test` 3058 passed, 0 failed（3056 + 2 件）
- `error_catalog.rs` の `ERROR_CATALOG` 配列に `E0418` エントリが含まれること
- `cargo clippy -- -D warnings` クリーン
- `fav/Cargo.toml` version → `"48.6.0"`
- `CHANGELOG.md` に v48.6.0 エントリ追加
- `versions/current.md` を v48.6.0（3058 tests）に更新、進行中バージョンを `v48.7.0` に更新
- `tasks.md` を COMPLETE に更新（T0〜T3 全 `[x]`）
- `cargo clean` はこのバージョンのスコープ外（v49.0.0 で実施）
- `site/` MDX 更新は不要（E0417/E0418 のエラーコードドキュメントは v48.9.0 のドキュメント整備スプリントで対応）
