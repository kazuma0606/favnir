# Favnir v8.2.0 実装計画

Date: 2026-05-29

---

## フェーズ構成

```
Phase A  list_stdlib.fav — List 高レベル関数（Favnir 実装）
Phase B  string_stdlib.fav — String 高レベル関数（Favnir 実装）
Phase C  map_stdlib.fav — Map 高レベル関数（Favnir 実装）
Phase D  stdlib_fav_runner.rs — ローダー＋vm.rs 統合
Phase E  統合テスト（各モジュール 3 件）
Phase F  fav check 自己チェック + ドキュメント + commit
```

---

## Phase A: list_stdlib.fav

**ファイル**: `fav/self/stdlib/list_stdlib.fav`

実装する関数と使用するプリミティブ：

| 関数 | 実装方針 |
|------|---------|
| `intersperse` | `List.fold` でアキュムレータに挟みながら構築 |
| `flat_map` | `List.fold` + `List.append` でネストを展開 |
| `zip_with` | `List.fold` + インデックス管理（List.nth 代用） |
| `scan` | `List.fold` で中間状態をリストに記録 |
| `take_while` | 再帰（`List.first` + `List.rest`） |
| `drop_while` | 再帰（条件が偽になるまで `List.rest`） |
| `chunk` | 再帰 + `List.take`/`List.drop` |
| `uniq` | `List.fold` + `List.contains` で重複除去 |
| `sort_by` | insertion sort（`List.fold` ベース） |
| `group_by` | `List.fold` + `Map.get`/`Map.set` でグルーピング |

**使用できる List プリミティブ（Rust 側に確認済み）**:
- `List.fold`, `List.map`, `List.filter`, `List.length`
- `List.first`, `List.rest`, `List.append`, `List.push`
- `List.contains`, `List.take`, `List.drop`, `List.reverse`
- `List.nth`（`List.first(List.drop(xs, n))` で代用）

**エクスポートする関数名（vm.rs から呼べる名前）**:
```
intersperse, flat_map, zip_with, scan,
take_while, drop_while, chunk, uniq, sort_by, group_by
```

---

## Phase B: string_stdlib.fav

**ファイル**: `fav/self/stdlib/string_stdlib.fav`

| 関数 | 実装方針 |
|------|---------|
| `words` | `String.split(s, " ")` のラッパー（空要素除去あり） |
| `lines` | `String.split(s, "\n")` のラッパー |
| `pad_left` | `String.repeat` で padding 生成 + concat |
| `pad_right` | 同上 |
| `repeat` | `List.fold` で concat |
| `capitalize` | `String.upper(String.slice(s, 0, 1))` + rest |
| `indent` | `lines` → 各行に `" " * n` を先頭追加 → `String.join` |

**使用できる String プリミティブ**:
- `String.split`, `String.concat`, `String.length`
- `String.slice`, `String.upper`, `String.lower`
- `String.join`, `String.trim`

---

## Phase C: map_stdlib.fav

**ファイル**: `fav/self/stdlib/map_stdlib.fav`

| 関数 | 実装方針 |
|------|---------|
| `map_values` | `Map.keys` → `List.fold` で新 Map 構築 |
| `filter` | `Map.keys` → `List.fold` で条件 true のみ保持 |
| `merge` | `Map.keys(b)` → `List.fold` で a に追加 |
| `from_list` | `List.fold` + `Map.set` |
| `to_list` | `Map.keys` → `List.map` で `(k, v)` ペアに |
| `count` | `Map.keys` → `List.length` |

**使用できる Map プリミティブ**:
- `Map.empty`, `Map.get`, `Map.set`, `Map.keys`, `Map.has`

---

## Phase D: stdlib_fav_runner.rs

**ファイル**: `src/stdlib_fav_runner.rs`（main.rs のみで宣言）

```
OnceLock<Arc<FvcArtifact>>  ×3（list / string / map）

pub fn call_stdlib(ns: &str, fname: &str, args: Vec<VMValue>)
  -> Option<Result<VMValue, VMError>>
```

**vm.rs 統合点**:
`vm_call_builtin` の先頭（既存 match の前）に dispatch 追加。
失敗（None）なら既存 Rust 実装にフォールバック。

**注意点**:
- クロージャ引数を受け取る関数（zip_with の comparator 等）は
  VMValue::Closure を stdlib runner 経由で渡せる必要がある
- クロージャ呼び出しは `VM::call_closure` を使う
- 最初のリリースではクロージャ引数不要の関数だけ Favnir 化し、
  closure 引数ありは Rust のまま（sort_by / group_by は後回し）

---

## Phase E: 統合テスト

**driver.rs** に `stdlib_v82_tests` モジュール追加：

```
E-1: list_intersperse — [1,2,3] を "," で intersperse → [1,",",2,",",3]
E-2: list_flat_map   — [[1,2],[3]] を flat_map(id) → [1,2,3]
E-3: list_uniq       — [1,2,1,3,2] → [1,2,3]
E-4: string_words    — "hello world" → ["hello","world"]
E-5: string_repeat   — "ab" × 3 → "ababab"
E-6: string_capitalize — "hello" → "Hello"
E-7: map_from_list   — [("a",1),("b",2)] → Map{a:1,b:2}
E-8: map_count       — 2 entries → 2
E-9: map_merge       — {a:1}+{b:2} → {a:1,b:2}
```

---

## Phase F: 最終確認・ドキュメント

- F-1: `fav check fav/self/stdlib/*.fav` — エラーなし
- F-2: `cargo test` — 1115+ tests passing
- F-3: `site/content/docs/stdlib/` にドキュメント追加または更新
- F-4: tasks.md を完了状態に更新
- F-5: commit

---

## リスク・注意点

| リスク | 対策 |
|-------|------|
| クロージャ引数を Favnir 側に渡せない | Phase D でクロージャ引数なし関数のみ先行 Favnir 化 |
| zip_with が 2リスト同時走査できない | `List.fold` + インデックスを tuple で持つパターン |
| sort_by の insertion sort が O(n²) | stdlib は正確性優先、パフォーマンスは後回し |
| vm.rs の dispatch 追加でオーバーヘッド | OnceLock + 直接 fn lookup で最小化 |
