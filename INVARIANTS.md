# Favnir 不変条件

このドキュメントは「絶対に壊してはいけないこと」を定義します。
人間・AI を問わず、すべてのコントリビューターが変更前に確認してください。

---

## なぜ不変条件が必要か

Favnir はセルフホスト言語です。コンパイラ自身が Favnir で書かれているため、
壊れ方が連鎖します。「動いているように見えて、実は仕様から外れている」状態を
防ぐために、検証可能な基準を明文化します。

---

## I. コア不変条件（絶対に破ってはいけない）

### I-1. Bootstrap 検証が通ること

```bash
cd fav
cargo test bootstrap_full_self_hosting
```

`compiler.fav` が自分自身をコンパイルした結果（Stage1 → Stage2 → Stage3）の
バイトコードが一致すること。これが Favnir の正しさの根拠です。

**これが壊れた変更はマージしてはいけません。**

### I-2. Self-check が通ること

```bash
./target/debug/fav check self/compiler.fav
./target/debug/fav check self/checker.fav
./target/debug/fav check self/cli.fav
```

Favnir 自身の型チェッカー（`checker.fav`）がセルフホスト実装を
エラーなしで検査できること。

### I-3. 全テストが通ること（件数が減らないこと）

```bash
cargo test
# 期待: 1261 passed 以上
```

テスト件数は単調増加が原則です。既存テストの削除・スキップは禁止です。
`#[ignore]` を追加するには理由をコメントに明記してください。

---

## II. 設計上の不変条件（変えるには議論が必要）

### II-1. VM は Rust のまま

実行エンジン（`fav/src/backend/vm.rs`）は恒久的に Rust で実装します。
Favnir で VM を書き直すことはしません。

**理由**: メモリ安全性・JIT 最適化の余地・FFI 境界の明確化。

### II-2. `--legacy` フラグは削除しない

非推奨化済みですが、後方互換のため残します。
Rust パイプラインへのフォールバック手段として機能します。

### II-3. `fav run` / `fav check` のデフォルトは Favnir パイプライン

- `fav check` → `checker.fav` 経由（Rust checker は `--legacy` のみ）
- `fav run` → `compiler.fav` 経由（Rust compiler は `--legacy` のみ）

Rust パイプラインをデフォルトに戻してはいけません。

### II-4. エフェクトシステムの完全性

副作用を持つ操作は必ずエフェクト宣言が必要です。

```favnir
fn fetch(url: String) -> Result<String, String> !Http { ... }
//                                               ^^^^^ 必須
```

新しい Rune を追加するとき、対応するエフェクト（`!Http` / `!Snowflake` 等）を
`checker.fav` と `checker.rs` の両方に登録してください。
エフェクトなしで副作用を持つ関数を追加してはいけません。

---

## III. 言語仕様の不変条件

### III-1. 後方互換性

既存の `.fav` ファイルが `fav run` でエラーになるような変更は
メジャーバージョンアップ（例: v10 → v11）なしに行ってはいけません。

### III-2. パーサーの一貫性

`compiler.fav` のパーサーと `fav/src/frontend/parser.rs` の挙動は
一致していなければなりません。どちらかを変更したら両方を更新してください。

---

## IV. プロセス上の不変条件

### IV-1. 1 PR = 1 つの変更

機能追加・バグ修正・リファクタリングを 1 つの PR に混在させません。

### IV-2. 新機能にはテストが必要

新しいエラーコード・構文・Rune を追加するときは、
同じ PR にテストを含めてください。テストなしの機能追加は受け付けません。

### IV-3. AI による変更のガイドライン

Favnir は AI（Claude）と協働で開発されています。AI に指示するときは：

- 変更前に対象ファイルを必ず読んでから編集する
- 「リファクタリング」「改善」を単独で指示しない（機能変更と分離する）
- 変更後に I-1〜I-3 のコア不変条件を検証する
- セッションをまたぐ重要な決定は `memory/` または `versions/` に記録する

---

## チェックリスト（PR 前に確認）

```
[ ] cargo test bootstrap_full_self_hosting  が通る
[ ] fav check self/compiler.fav            がエラーなし
[ ] fav check self/checker.fav             がエラーなし
[ ] cargo test                             で全件通る（件数が減っていない）
[ ] 新機能の場合、対応するテストが含まれている
[ ] エフェクトを持つ新 Rune の場合、checker.fav と checker.rs に登録済み
```
