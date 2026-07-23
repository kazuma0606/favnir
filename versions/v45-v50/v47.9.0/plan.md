# Plan: v47.9.0 — stdlib ドキュメント + v48.0 前調整

## 実装ステップ

### Step 1: `site/content/docs/stdlib/float.mdx` 新規作成

以下の内容で `site/content/docs/stdlib/float.mdx` を作成する（フェンスは実装時に付与）:

    ---
    title: "Float / Int 拡張"
    order: 10
    category: "標準ライブラリ"
    description: "Float.round / Float.clamp / Float.abs / Int.to_hex / Int.abs (v47.5.0)"
    ---

    # Float / Int 拡張

    v47.5.0 で追加した浮動小数点・整数の拡張関数。

    ## 関数一覧

    | 関数 | シグネチャ | 説明 |
    |------|-----------|------|
    | `Float.round` | `Float, Int -> Float` | 小数点以下 n 桁に丸める |
    | `Float.clamp` | `Float, Float, Float -> Float` | 範囲内にクランプ（lo ≤ f ≤ hi） |
    | `Float.abs` | `Float -> Float` | 絶対値 |
    | `Int.to_hex` | `Int -> String` | 16 進数文字列に変換（小文字） |
    | `Int.abs` | `Int -> Int` | 絶対値（i64::MIN はエラー） |

    ## 使用例

        bind rounded <- Float.round(3.14159, 2)   // 3.14
        bind clamped <- Float.clamp(150.0, 0.0, 100.0)  // 100.0
        bind hex     <- Int.to_hex(255)           // "ff"

### Step 2: `site/content/docs/stdlib/v2.mdx` 新規作成

```mdx
---
title: "Standard Library 2.0"
order: 1
category: "標準ライブラリ"
description: "Favnir Standard Library 2.0 — v47 シリーズ全追加関数の索引"
---

# Standard Library 2.0

v47.1〜v47.9 で追加した標準ライブラリ関数の索引。

## 追加関数一覧

### List（v47.1〜v47.3）
- `List.zip` / `List.chunk` — v47.1.0
- `List.flat_map` / `List.group_by` / `List.dedupe` — v47.2.0
- `List.scan` / `List.take_while` / `List.drop_while` — v47.3.0

### String（v47.4）
- `String.pad_left` / `String.trim_start` / `String.repeat` — v47.4.0

### Float / Int（v47.5）
- `Float.round` / `Float.clamp` / `Float.abs` / `Int.to_hex` / `Int.abs` — v47.5.0

### Option（v47.6）
- `Option.map` / `Option.unwrap_or` / `Option.and_then` / `Option.is_some` / `Option.is_none` — v47.6.0

### Result（v47.7）
- `Result.map` / `Result.map_err` / `Result.and_then` / `Result.is_ok` / `Result.is_err` — v47.7.0

### Map（v47.8）
- `Map.merge` / `Map.filter_values` / `Map.map_values` / `Map.keys` / `Map.values` — v47.8.0
```

### Step 3: `site/content/docs/stdlib/option.mdx` 更新

既存の関数一覧テーブルの末尾に以下を追記:

    | `Option.map` | `Option(A), (A->B) -> Option(B)` | 値を変換 |
    | `Option.unwrap_or` | `Option(A), A -> A` | 値を取り出す（none はデフォルト値） |
    | `Option.and_then` | `Option(A), (A->Option(B)) -> Option(B)` | フラットマップ |
    | `Option.is_some` | `Option(A) -> Bool` | Some かどうか |
    | `Option.is_none` | `Option(A) -> Bool` | None かどうか |

### Step 4: `site/content/docs/stdlib/result.mdx` 更新

既存の関数一覧テーブルの末尾に以下を追記:

    | `Result.map` | `Result(T,E), (T->U) -> Result(U,E)` | ok 値を変換 |
    | `Result.map_err` | `Result(T,E), (E->F) -> Result(T,F)` | err 値を変換 |
    | `Result.and_then` | `Result(T,E), (T->Result(U,E)) -> Result(U,E)` | フラットマップ |
    | `Result.is_ok` | `Result(T,E) -> Bool` | Ok かどうか |
    | `Result.is_err` | `Result(T,E) -> Bool` | Err かどうか |

### Step 5: `site/content/cookbook/stdlib-v2.mdx` 新規作成

v47 シリーズの新関数を使ったサンプルパイプライン。frontmatter に `title: "Standard Library 2.0 サンプル"` を含める。

### Step 6: `site/content/docs/stdlib/list.mdx` 更新

既存の関数一覧テーブルの末尾に以下を追記:

```mdx
| `List.zip` | `List(A), List(B) -> List((A,B))` | 2 リストをペアに結合 |
| `List.chunk` | `List(A), Int -> List(List(A))` | n 要素ずつに分割 |
| `List.flat_map` | `List(A), (A->List(B)) -> List(B)` | map して flatten |
| `List.group_by` | `(A->String), List(A) -> Map(String,List(A))` | キーでグループ化 |
| `List.dedupe` | `List(A) -> List(A)` | 重複除去（順序保持） |
| `List.scan` | `List(A), B, (B,A->B) -> List(B)` | 累積値リスト（init 含む） |
| `List.take_while` | `List(A), (A->Bool) -> List(A)` | 条件を満たす先頭部分 |
| `List.drop_while` | `List(A), (A->Bool) -> List(A)` | 条件を満たす先頭を除去 |
```

### Step 7: `site/content/docs/stdlib/string.mdx` 更新

既存の関数一覧テーブルの末尾に以下を追記:

```mdx
| `String.pad_left` | `String, Int, String -> String` | 左パディング（幅・埋め文字指定） |
| `String.trim_start` | `String -> String` | 先頭空白除去 |
| `String.repeat` | `String, Int -> String` | n 回繰り返し |
```

### Step 8: `site/content/docs/stdlib/map.mdx` 更新

既存の関数一覧テーブルの末尾に以下を追記:

```mdx
| `Map.merge` | `Map(K,V), Map(K,V) -> Map(K,V)` | 2 マップを結合（右辺優先） |
| `Map.filter_values` | `Map(K,V), (V->Bool) -> Map(K,V)` | 値でフィルタリング |
| `Map.map_values` | `Map(K,V), (V->W) -> Map(K,W)` | 全値を変換 |
```

### Step 9: `driver.rs` に `v479000_tests` 追加

挿入位置: `v478000_tests` モジュールの直前。

```rust
// -- v479000_tests (v47.9.0) -- stdlib ドキュメント存在確認 --
#[cfg(test)]
mod v479000_tests {
    #[test]
    fn stdlib_v2_doc_exists() {
        let content = include_str!("../../site/content/docs/stdlib/float.mdx");
        assert!(content.contains("Float.round"), "float.mdx should document Float.round");
    }

    #[test]
    fn stdlib_v2_overview_exists() {
        let content = include_str!("../../site/content/docs/stdlib/v2.mdx");
        assert!(
            content.contains("Standard Library 2.0"),
            "v2.mdx should mention Standard Library 2.0"
        );
    }
}
```

### Step 10: `Cargo.toml` バージョン更新

```toml
version = "47.9.0"
```

### Step 11: `CHANGELOG.md` 更新

```markdown
## [v47.9.0] — 2026-07-18

### Added
- `site/content/docs/stdlib/float.mdx` 新規作成（Float.round / Float.clamp / Float.abs / Int.to_hex / Int.abs）
- `site/content/docs/stdlib/v2.mdx` 新規作成（Standard Library 2.0 概要）
- `list.mdx` / `string.mdx` / `map.mdx` に v47 シリーズ追加関数を追記
- `driver.rs`: `v479000_tests` 追加（`stdlib_v2_doc_exists` / `stdlib_v2_overview_exists` 2テスト）
```

### Step 12: `versions/current.md` 更新

- 最新安定版を `v47.9.0`（3041 tests）に更新
- 進行中バージョン・次に切る版を `v48.0.0` に更新

---

## テスト数

| バージョン | テスト数 | 差分 |
|---|---|---|
| v47.8.0 | 3039 | ベース |
| v47.9.0 | 3041 | +2 |
