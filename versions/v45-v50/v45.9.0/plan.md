# Plan: v45.9.0 — examples 更新 Phase 2 + v46.0 前調整

Date: 2026-07-16
Status: TODO

---

## ステップ

### Step 1 — 事前確認

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2986 tests passed, 0 failed を確認。

---

### Step 2 — `examples/pipeline/stage_seq_demo.fav` コメント修正

ファイルを読み込み、ヘッダーコメントの「alias for `trf`」説明を正確な現在の説明に更新する。

変更箇所（1・2行目 + 8行目のコメント）:
```
// 変更前 1行目:
// stage_seq_demo.fav — v1.9.0: stage/seq keyword aliases for trf/flw
// 変更後 1行目:
// stage_seq_demo.fav — pipeline stage/seq keywords demonstration

// 変更前 2行目:
// `stage` is an alias for `trf` (transform stage in a pipeline)
// 変更後 2行目:
// `stage` defines a transform stage in a pipeline

// 変更前 8行目:
// `seq` is an alias for `flw` (sequence of stages)
// 変更後 8行目:
// `seq` defines a sequence of pipeline stages
```

---

### Step 3 — `site/content/docs/language-refinement-overview.mdx` 新規作成

`site/content/docs/precision-and-flow-overview.mdx` と同構造で作成。

```mdx
---
title: Language Refinement Overview
description: Favnir v45.x スプリント「Language Refinement」の達成事項と v46.0 への道筋
---

# Language Refinement Overview

...（v45.1〜v45.9 の達成事項テーブル・主要機能の説明）
```

---

### Step 4 — `driver.rs`: v459000_tests 追加

`v458000_tests` の `}` 直後に `v459000_tests` モジュールを追加（2件）:
- `examples_structure_valid`: examples/ に 50 件以上の .fav ファイルが存在
- `language_refinement_overview_doc_exists`: MDX ファイルが存在し `"Language Refinement"` を含む

`walkdir` は v458000_tests と同様に `#[cfg(not(target_arch = "wasm32"))]` を付与。

---

### Step 5 — テスト実行

```bash
cd /c/Users/yoshi/favnir/fav && cargo test -j 8 -- --test-threads=8 2>&1 | grep "test result"
```

2988 tests passed, 0 failed を確認。

---

### Step 6 — Clippy

```bash
cd /c/Users/yoshi/favnir/fav && cargo clippy -- -D warnings 2>&1 | tail -5
```

---

### Step 7 — バージョン更新・ドキュメント

1. `fav/Cargo.toml`: `version = "45.9.0"`
2. `CHANGELOG.md`: v45.9.0 エントリ追加
3. `versions/current.md`: v45.9.0（2988 tests）に更新
4. `versions/v45-v50/v45.9.0/tasks.md`: COMPLETE に更新

---

## 実装順序まとめ

```
Step 1: cargo test（事前確認: 2986 tests）
Step 2: stage_seq_demo.fav — コメント修正
Step 3: site/ — language-refinement-overview.mdx 新規作成
Step 4: driver.rs — v459000_tests 追加（2件）
Step 5: cargo test（全通過確認: 2988 tests）
Step 6: cargo clippy
Step 7: バージョン・ドキュメント更新
```
