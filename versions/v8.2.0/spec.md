# Favnir v8.2.0 Spec

Date: 2026-05-29
Theme: stdlib Favnir 化 — List / String / Map 高レベル関数のセルフホスト

---

## 概要

v8.1.0 で checker.fav が `fav check` パイプラインに接続された。
v8.2.0 では、Rust `vm.rs` にハードコードされている **高レベル標準ライブラリ関数** を
Favnir 自身で書いた `self/stdlib/*.fav` に移植し、起動時にロード・実行する。

これにより：
- stdlib の実装が Favnir ソースとして可読・テスト可能になる
- Rust vm.rs の肥大化が止まる（新しい stdlib 関数は Favnir で追加できる）
- セルフホスト度が大きく上がる（checker.fav + stdlib.fav が Favnir で動く）

---

## 移植対象関数

### List 高レベル関数（`self/stdlib/list_stdlib.fav`）

| 関数 | シグネチャ | 現在の実装 |
|------|-----------|-----------|
| `List.zip_with` | `(List<A>, List<B>, (A, B) -> C) -> List<C>` | vm.rs |
| `List.group_by` | `(List<A>, (A) -> String) -> Map<String, List<A>>` | vm.rs |
| `List.sort_by` | `(List<A>, (A, A) -> Int) -> List<A>` | vm.rs |
| `List.intersperse` | `(List<A>, A) -> List<A>` | runes/stdlib/list.fav |
| `List.flat_map` | `(List<A>, (A) -> List<B>) -> List<B>` | vm.rs |
| `List.scan` | `(List<A>, B, (B, A) -> B) -> List<B>` | 未実装 |
| `List.take_while` | `(List<A>, (A) -> Bool) -> List<A>` | 未実装 |
| `List.drop_while` | `(List<A>, (A) -> Bool) -> List<A>` | 未実装 |
| `List.chunk` | `(List<A>, Int) -> List<List<A>>` | 未実装 |
| `List.uniq` | `(List<A>) -> List<A>` | 未実装 |

### String 高レベル関数（`self/stdlib/string_stdlib.fav`）

| 関数 | シグネチャ | 現在の実装 |
|------|-----------|-----------|
| `String.words` | `(String) -> List<String>` | 未実装（split " " の糖衣） |
| `String.lines` | `(String) -> List<String>` | 未実装（split "\n" の糖衣） |
| `String.pad_left` | `(String, Int, String) -> String` | 未実装 |
| `String.pad_right` | `(String, Int, String) -> String` | 未実装 |
| `String.repeat` | `(String, Int) -> String` | 未実装 |
| `String.capitalize` | `(String) -> String` | 未実装 |
| `String.indent` | `(String, Int) -> String` | 未実装（各行に空白を追加） |

### Map 高レベル関数（`self/stdlib/map_stdlib.fav`）

| 関数 | シグネチャ | 現在の実装 |
|------|-----------|-----------|
| `Map.filter` | `(Map<K,V>, (K, V) -> Bool) -> Map<K,V>` | 未実装 |
| `Map.map_values` | `(Map<K,V>, (V) -> W) -> Map<K,W>` | 未実装 |
| `Map.merge` | `(Map<K,V>, Map<K,V>) -> Map<K,V>` | 未実装 |
| `Map.from_list` | `(List<(String, V)>) -> Map<String, V>` | runes/stdlib/map.fav |
| `Map.to_list` | `(Map<K,V>) -> List<(String, V)>` | 未実装 |
| `Map.count` | `(Map<K,V>) -> Int` | 未実装 |

---

## アーキテクチャ

### ローダー: `src/stdlib_fav_runner.rs`

checker_fav_runner.rs と同じパターン：

```
static LIST_STDLIB_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();
static STRING_STDLIB_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();
static MAP_STDLIB_ARTIFACT: OnceLock<Arc<FvcArtifact>> = OnceLock::new();

pub fn call_stdlib(ns: &str, fname: &str, args: Vec<VMValue>) -> Option<Result<VMValue, VMError>>
```

- `ns` が `"List"` / `"String"` / `"Map"` のとき該当アーティファクトを検索
- 関数が見つからなければ `None`（Rust フォールバックへ）
- 見つかれば `VM::run` で実行 → `Some(result)`

### vm.rs への統合

`vm_call_builtin` の先頭で `stdlib_fav_runner::call_stdlib(ns, fname, args)` を試みる：

```rust
// 新しい Favnir 実装を優先
if let Some(result) = stdlib_fav_runner::call_stdlib(ns, fname, args.clone()) {
    return result;
}
// Rust フォールバック（既存実装）
```

### Favnir ソースの配置

```
fav/self/stdlib/
  list_stdlib.fav    — List 高レベル関数
  string_stdlib.fav  — String 高レベル関数
  map_stdlib.fav     — Map 高レベル関数
```

---

## 移植戦略

### プリミティブは Rust のまま

`List.map`、`List.filter`、`List.fold`、`String.split`、`Map.get` などの
コア primitive は Rust のまま（クロージャ呼び出しの仕組みを要する）。

Favnir 側で実装するのは、これらの **上に乗る** 高レベル関数のみ。

### ブートストラップ安全性

stdlib_fav_runner.rs は OnceLock なので起動時に一度だけコンパイル。
checker_fav_runner.rs と独立したキャッシュを持つ（相互依存なし）。

---

## 完了条件

- `fav check fav/self/stdlib/list_stdlib.fav` — エラーなし
- `fav check fav/self/stdlib/string_stdlib.fav` — エラーなし
- `fav check fav/self/stdlib/map_stdlib.fav` — エラーなし
- 既存テスト全件通過（1106+）
- 新規統合テスト 9 件以上（各モジュール 3 件）
- ドキュメント更新（stdlib の実装が Favnir であることを明記）
