# Plan: v47.1.0 — `List.zip` / `List.chunk`

## 方針

`List.zip` / `List.chunk` は vm.rs と checker.rs に実装済み。
本バージョンは `driver.rs` にテスト 2 件を追加し、動作を Rust テストで保証する。

---

## 実装ステップ

### Step 1: `fav/Cargo.toml` バージョン更新

```toml
version = "47.1.0"
```

### Step 2: `driver.rs` に `v471000_tests` 追加

挿入位置: `v47000_tests` モジュールの直後。

```rust
// -- v471000_tests (v47.1.0) -- List.zip / List.chunk 動作確認 --
#[cfg(test)]
mod v471000_tests {
    use crate::frontend::parser::Parser;
    use super::{build_artifact, exec_artifact_main};

    #[test]
    fn list_zip_pairs() {
        let src = r#"
fn main() -> Bool {
  bind names  <- List.from(["alice", "bob"])
  bind scores <- List.from([90, 80])
  bind pairs  <- List.zip(names, scores)
  List.length(pairs) == 2
}
"#;
        let program = Parser::parse_str(src, "list_zip_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }

    #[test]
    fn list_chunk_batches() {
        let src = r#"
fn main() -> Bool {
  bind data    <- List.from([1, 2, 3, 4, 5])
  bind batches <- List.chunk(data, 2)
  List.length(batches) == 3
}
"#;
        let program = Parser::parse_str(src, "list_chunk_test.fav").expect("parse");
        let artifact = build_artifact(&program);
        let value = exec_artifact_main(&artifact, None).expect("exec");
        assert_eq!(value, crate::value::Value::Bool(true));
    }
}
```

### Step 3: `CHANGELOG.md` 更新

```markdown
## [v47.1.0] — 2026-07-17

### Added
- `List.zip` / `List.chunk` 動作確認テスト 2 件（`list_zip_pairs` / `list_chunk_batches`）
```

### Step 4: `versions/current.md` 更新

- 最新安定版を `v47.1.0`（3018 tests）に更新
- 進行中バージョンを `v47.2.0` に更新

---

## 注意事項

### `List.zip` の戻り値形式

- `vm.rs` line 11154: `List.zip` は `VMValue::Record` で `"first"` と `"second"` フィールドを持つリストを返す
- テストでは `List.length(pairs) == 2` のみ確認（Record フィールドアクセスは `List.first` が Option を返すため複雑になる）

### `List.chunk` の端数処理

- 要素数が chunk サイズで割り切れない場合、余りは最後のバッチに含まれる
- 5 要素 / サイズ 2 → バッチ数 3（`ceil(5/2) = 3`）

### exec パターン

他バージョン（例: v31000_tests 等）と同じパターン:
```rust
let program = Parser::parse_str(src, "name.fav").expect("parse");
let artifact = build_artifact(&program);
let value = exec_artifact_main(&artifact, None).expect("exec");
assert_eq!(value, crate::value::Value::Bool(true));
```
- `VMValue` ではなく `crate::value::Value` を使用
- `exec_artifact_main` は `fn exec_artifact_main(artifact, db_path) -> Result<Value, String>` (driver.rs line 3102)

### T0 前提確認

vm.rs に実装が存在することを事前確認:
- `List.zip` → line 11154
- `List.chunk` → line 11332
- `List.zip_with` → line 3873（本バージョンではテスト対象外）

---

## テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.0.0 | 3016 | ベース |
| v47.1.0 | 3018 | +2 |
