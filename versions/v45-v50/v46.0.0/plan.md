# Plan: v46.0.0 — Language Refinement 宣言

Date: 2026-07-16
Status: TODO

---

## ステップ

### Step 1 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2988 tests passed, 0 failed を確認。

---

### Step 2 — `MILESTONE.md` 更新

`MILESTONE.md` の先頭（v45.0.0 エントリの上）に v46.0.0「Language Refinement」エントリを追記。

追記内容（`"Language Refinement"` を必ず含める）:

```markdown
## v46.0.0 — Language Refinement（2026-07-16）

> 「`return` によるガード節・`match` 完全網羅・型エイリアスの明確な境界・
>  改善されたエラーメッセージが揃い、Favnir の構文が成熟した。
>
>  これが Favnir v46.0 — Language Refinement の姿である。」

v46.0.0 をもって、Favnir の **Language Refinement** を正式に宣言する。

### 達成コンポーネント（v45.1〜v45.9）

| コンポーネント | バージョン | 内容 |
|---|---|---|
| `return` 構文 AST + parser | v45.1 | ReturnStmt ノード・parser 解析 |
| `return` 型チェック + E0415 | v45.2 | 戻り型不一致エラー |
| `return` compiler + VM | v45.3 | Return opcode・早期脱出実行 |
| `match` 網羅性 + W034/E0416 | v45.4 | 非網羅 match の警告・エラー |
| 型エイリアス完全化 | v45.5 | 透過的互換性・opaque 非互換性 |
| エラーメッセージ改善 Phase 1 | v45.6 | E0101〜E0200 suggestion 追加 |
| エラーメッセージ改善 Phase 2 + 数値リテラル `_` | v45.7 | E0201〜E0413 suggestion・`1_000_000` |
| examples 更新 Phase 1 | v45.8 | !Effect 除去確認・return ガード節 |
| examples 更新 Phase 2 + v46.0 前調整 | v45.9 | stage_seq_demo 修正・overview 作成 |
```

---

### Step 3 — `README.md` 更新

README.md の適切な場所（バージョン履歴や機能説明付近）に以下を追記:

```markdown
v46.0 — Language Refinement: `return` 構文・`match` 完全網羅・型エイリアス・エラーメッセージ改善が揃い、Favnir の構文が成熟しました。
```

`"Language Refinement"` という文字列を必ず含めること。

---

### Step 4 — `driver.rs`: v46000_tests 追加

まず `v459000_tests` モジュールの存在を確認する:

```bash
grep -n "v459000_tests" src/driver.rs
```

出力行番号を確認し、その終端 `}` の直後に `v46000_tests` モジュールを追加（4件）:
- `cargo_toml_version_is_46_0_0`
- `changelog_has_v46_0_0`
- `milestone_has_language_refinement`
- `readme_mentions_language_refinement`

`include_str!` マクロを使用。`../Cargo.toml`・`../../CHANGELOG.md`・`../../MILESTONE.md`・`../../README.md` のパスを使う。

---

### Step 5 — テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

まだ `Cargo.toml` を `46.0.0` に更新していない段階では `cargo_toml_version_is_46_0_0` が失敗する。
その他の 3 件が pass し、`cargo_toml_version_is_46_0_0` のみ失敗することを確認してから進む。

---

### Step 6 — `fav/Cargo.toml` バージョン更新

```
version = "45.9.0"  →  version = "46.0.0"
```

---

### Step 7 — 再テスト

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2992 passed（2988 + 4件）, 0 failed を確認。

> 注: 宣言テストは `#[cfg(not(target_arch = "wasm32"))]` を付けないため、
> wasm32 でも含まれる。ただし `include_str!` のみ使用のためビルド上問題なし。

---

### Step 8 — Clippy

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

---

### Step 9 — `cargo clean` ★クリーンアップ

```bash
cd /c/Users/yoshi/favnir/fav && cargo clean
```

---

### Step 10 — `fav/tmp/hello.fav` 復元

`cargo clean` 後に `fav/tmp/hello.fav` を復元する（`bootstrap_c2_artifact_roundtrip` テストが依存）:

```
fn add(a: Int, b: Int) -> Int { a + b }
fn main() -> Bool { add(1, 2) == 3 }
```

---

### Step 11 — cargo clean 後テスト確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

再ビルド後 2992 passed, 0 failed を確認。

---

### Step 12 — バージョン・ドキュメント更新

1. `CHANGELOG.md`: v46.0.0 エントリ追加
2. `versions/current.md`: v46.0.0（2992 tests）に更新、次バージョン = v46.1.0（未定）
3. `versions/v45-v50/v46.0.0/tasks.md`: COMPLETE に更新

---

## 実装順序まとめ

```
Step 1:  cargo test（事前確認: 2988 tests）
Step 2:  MILESTONE.md — Language Refinement エントリ追加
Step 3:  README.md — Language Refinement 追記
Step 4:  driver.rs — v46000_tests 追加（4件）
Step 5:  cargo test（cargo_toml 以外 3件 pass 確認）
Step 6:  Cargo.toml version → 46.0.0
Step 7:  cargo test（全 2992 pass 確認）
Step 8:  cargo clippy
Step 9:  cargo clean ★
Step 10: fav/tmp/hello.fav 復元
Step 11: cargo test（再ビルド後 2992 pass 確認）
Step 12: CHANGELOG・current.md・tasks.md 更新
```
