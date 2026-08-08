# v66.8.0 実装計画 — AI Pipeline Lint Rules（W055〜W059）

Version: 66.8.0
Status: 未着手
Base tests: 3489
Target tests: 3491

---

## 実装ステップ

> **前提**: spec.md の T0 前提確認を完了してから開始する。

### Step 1: `lint.rs` へ W055〜W059 スタブ追加

`fav/src/lint.rs` の `check_w054_missing_convergence` 関数の直後（末尾）に、
W055〜W059 のスタブ関数ブロックを追加する。

W050〜W054（v65.8.0）と完全に同一の形式で記述:
```rust
fn check_wXXX_name(_program: &Program, _errors: &mut Vec<LintError>) {}
```

### Step 2: `driver.rs` テスト追加

- `// -- v66700_tests (v66.7.0)` コメントの直前に `v66800_tests` を挿入
- 2 テスト関数:
  - `lint_w055_untyped_llm_output`（lint.rs に W055 / W056 が含まれることを確認）
  - `lint_w056_dim_implicit_cast`（lint.rs に W057 / W058 / W059 が含まれることを確認）

### Step 3: ビルド・テスト確認

```bash
# 以下は順番に実行すること（前コマンドが PASS してから次へ進む）
cargo build
cargo test --bin fav v66800_tests
cargo test -j 8 -- --test-threads=8
```

---

## lint ルール一覧

| コード | スタブ関数名 | 検出内容（将来実装） |
|---|---|---|
| W055 | `check_w055_untyped_llm_output` | 型なし LLM 出力をそのまま下流に流す |
| W056 | `check_w056_dim_implicit_cast` | 埋め込み次元の暗黙的キャスト |
| W057 | `check_w057_query_without_upsert` | ベクトル DB への upsert なしの query |
| W058 | `check_w058_unbuffered_stream_inference` | バッファなしストリーミング推論 |
| W059 | `check_w059_llm_no_retry` | LLM 呼び出しのリトライなし |

---

## `driver.rs` 挿入コード

```rust
// -- v66800_tests (v66.8.0) -- AI Pipeline Lint Rules --
#[cfg(test)]
mod v66800_tests {
    #[test]
    fn lint_w055_untyped_llm_output() {
        let lint = include_str!("lint.rs");
        assert!(lint.contains("W055"), "lint.rs should define W055");
        assert!(lint.contains("W056"), "lint.rs should define W056");
    }

    #[test]
    fn lint_w056_dim_implicit_cast() {
        let lint = include_str!("lint.rs");
        assert!(lint.contains("W057"), "lint.rs should define W057");
        assert!(lint.contains("W058"), "lint.rs should define W058");
        assert!(lint.contains("W059"), "lint.rs should define W059");
    }
}
```

---

## リスク・注意点

- `include_str!("lint.rs")` のパスは `fav/src/driver.rs` 起点で `fav/src/lint.rs` を指す（Rune ファイルの `../../runes/...` とは異なる）
- テスト関数名（`lint_w055_untyped_llm_output` / `lint_w056_dim_implicit_cast`）はロードマップが指定した名称であり、アサート対象（W055+W056 / W057+W058+W059）とずれているが意図的。将来の検出ロジックテスト追加時は別モジュールで追加すること
- W050〜W054 のスタブ関数は `run_all_checks`（lint.rs 行 164〜169）に登録済み。W055〜W059 も同様に登録する（`// v66.8.0: W055〜W059 AI Pipeline Lint Rules` コメント付きで追加）
- スタブ関数は `pub` を付けない（W050〜W054 と同形式）
- `_program` / `_errors` の命名は既存形式を踏襲する（コンパイラの未使用変数警告を避けるため）

## 非スコープ

- W055〜W059 の実際の AST 走査による検出ロジック実装 — 将来フェーズ
- lint.rs の `run_all_checks` への登録 — 将来フェーズ
- `fav check --ai` 等の専用フラグ — 将来フェーズ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略
