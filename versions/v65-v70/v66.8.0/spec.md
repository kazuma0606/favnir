# v66.8.0 Spec — AI Pipeline Lint Rules（W055〜W059）

Version: 66.8.0
Status: 未着手
Base tests: 3489
Target tests: 3491

---

## 概要

AI パイプライン特有のアンチパターンを静的解析で検出する lint ルール W055〜W059 を追加する。
LLM・ベクトル・ストリーミング推論の落とし穴を事前に警告する。

ロードマップ `roadmap-v66.1-v67.0.md` の v66.8.0 セクションに準拠。

> **スコープ縮小の明示**: ロードマップでは実際の静的解析による検出（AST を走査して
> パターンを検出する実装）を示しているが、これは将来フェーズ。
> 本バージョンでは `fav/src/lint.rs` にスタブ関数として W055〜W059 を登録し、
> `include_str!` テストで存在確認のみ行う。実際の検出ロジック実装は将来フェーズ。

---

## lint ルール一覧

| コード | 検出内容 | 重大度 |
|---|---|---|
| W055 | 型なし LLM 出力をそのまま下流に流す（`Rune.llm.call` の結果を String のまま使用） | warning |
| W056 | 埋め込み次元の暗黙的キャスト（`Vec<Float>[768]` → `Vec<Float>[1536]` の代入） | error |
| W057 | ベクトル DB への upsert なしの query（空インデックスへの問い合わせリスク） | warning |
| W058 | ストリーミング推論ステージでのバッファなし直接処理（メモリ溢れのリスク） | warning |
| W059 | LLM 呼び出しのリトライなし（外部 API 一時障害への無対策） | info |

---

## 前提確認（T0 で実施）

- `cargo test -j 8 -- --test-threads=8` でベース 3489 tests passed, 0 failed を確認
- `fav/Cargo.toml` の version が `"66.0.0"` であることを確認（v66.0.0 宣言時に設定済み。v66.x sub-version では更新しない。v67.0.0 宣言時に `"67.0.0"` に更新する）
- `fav/src/lint.rs` に `W054` が存在することを確認（W055 の挿入位置の目印）
- `fav/src/lint.rs` に `W055` が存在しないことを確認（新規追加）
- `driver.rs` に `v66700_tests` が存在することを確認（`v66800_tests` の挿入位置）
- `driver.rs` に `v66800_tests` が存在しないことを確認（新規追加）
- `cargo test --bin fav v66700_tests` で 2 件 PASS することを確認（前バージョンが正常）
  - 前バージョンのテスト関数名: `feature_store_define_feature`, `feature_store_versioned_retrieval`
- `versions/current.md` の「進行中バージョン」が `v66.7.0` であることを確認（確認失敗時は前バージョンの tasks.md T4 が完了していることを確認してから current.md を手動修正すること）

---

## 実装スコープ

### 1. `fav/src/lint.rs` — W055〜W059 スタブ追加

**Step A**: `run_all_checks` 関数内（`check_w054_missing_convergence` 呼び出しの直後、`errors` を返す前）に登録を追加する:

```rust
    // v66.8.0: W055〜W059 AI Pipeline Lint Rules
    check_w055_untyped_llm_output(program, &mut errors);
    check_w056_dim_implicit_cast(program, &mut errors);
    check_w057_query_without_upsert(program, &mut errors);
    check_w058_unbuffered_stream_inference(program, &mut errors);
    check_w059_llm_no_retry(program, &mut errors);
```

**Step B**: `check_w054_missing_convergence` 関数の直後にスタブ関数ブロックを追加する:

```rust
// ── W055〜W059: AI Pipeline Lint Rules (v66.8.0) ─────────────────────────────

// W055: 型なし LLM 出力をそのまま下流に流す（Rune.llm.call の結果を String のまま使用）
// 今バージョンはスタブ（将来フェーズで Rune.llm.call 戻り値の型チェックを実装）
fn check_w055_untyped_llm_output(_program: &Program, _errors: &mut Vec<LintError>) {}

// W056: 埋め込み次元の暗黙的キャスト（Vec<Float>[768] → Vec<Float>[1536] の代入）
// 今バージョンはスタブ（将来フェーズで Vec<Float>[N] 次元型の代入互換チェックを実装）
fn check_w056_dim_implicit_cast(_program: &Program, _errors: &mut Vec<LintError>) {}

// W057: ベクトル DB への upsert なしの query（空インデックスへの問い合わせリスク）
// 今バージョンはスタブ（将来フェーズで Rune.pinecone / pgvector の呼び出し順序を検出）
fn check_w057_query_without_upsert(_program: &Program, _errors: &mut Vec<LintError>) {}

// W058: ストリーミング推論ステージでのバッファなし直接処理（メモリ溢れのリスク）
// 今バージョンはスタブ（将来フェーズで Rune.inference.stream_with_backpressure の欠如を検出）
fn check_w058_unbuffered_stream_inference(_program: &Program, _errors: &mut Vec<LintError>) {}

// W059: LLM 呼び出しのリトライなし（外部 API 一時障害への無対策）
// 今バージョンはスタブ（将来フェーズで Rune.llm / Rune.embed の呼び出しに retry 設定がないことを検出）
fn check_w059_llm_no_retry(_program: &Program, _errors: &mut Vec<LintError>) {}
```

### 2. `driver.rs` — `v66800_tests` 追加

挿入位置: `// -- v66700_tests (v66.7.0)` コメントの直前

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

挿入後、`cargo build` でエラーなしを確認。

---

## 完了条件

- `fav/src/lint.rs` に W055〜W059 のスタブ関数が追加されている
- `cargo test --bin fav v66800_tests` で 2 件 PASS
  - `lint_w055_untyped_llm_output` PASS
  - `lint_w056_dim_implicit_cast` PASS
- `cargo test -j 8 -- --test-threads=8` で 3491 tests passed, 0 failed
- CHANGELOG.md 更新・site/ MDX 作成は意図的に省略（非スコープセクション参照）

---

## 非スコープ

- W055〜W059 の実際の AST 走査による検出ロジック実装 — 将来フェーズ
- W055〜W059 の実際の AST 走査による検出ロジック実装 — 将来フェーズ（スタブ関数の本体は空のまま）
- `fav check --ai` 等の専用フラグ — 将来フェーズ
- CHANGELOG.md 更新 — v67.0.0 宣言時に一括追記
- site/ MDX ドキュメント作成 — v66.9.0 安定化時に一括作成するため今バージョンは省略

---

## 技術ノート

### テスト数増加の根拠

`v66800_tests` モジュール内の `#[test]` fn 2 件（`lint_w055_untyped_llm_output` / `lint_w056_dim_implicit_cast`）で +2。

### `include_str!` パス（`fav/src/driver.rs` 起点）

- `"lint.rs"` → `fav/src/lint.rs`（driver.rs と同じ `fav/src/` ディレクトリ内）

### `contains` 判定の設計方針

- `lint.contains("W055")` — スタブコメント行 `// W055:` にマッチ
- `lint.contains("W056")` — スタブコメント行 `// W056:` にマッチ
- `lint.contains("W057")` — スタブコメント行 `// W057:` にマッチ
- `lint.contains("W058")` — スタブコメント行 `// W058:` にマッチ
- `lint.contains("W059")` — スタブコメント行 `// W059:` にマッチ

### 挿入位置の根拠

`fav/src/lint.rs` の末尾付近（行 3515 前後）に W050〜W054（v65.8.0）のスタブ関数が存在する。
W055〜W059 はその直後に追加する。

### Rust スタブ関数の形式

W050〜W054 と同一形式:
```rust
fn check_wXXX_name(_program: &Program, _errors: &mut Vec<LintError>) {}
```
- 引数はすべてアンダースコアプレフィックス（未使用変数警告を避けるため）
- 関数本体は空（スタブ）
- `pub` / `pub(crate)` は付けない（W050〜W054 と同形式）
