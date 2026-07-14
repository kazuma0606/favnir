# Roadmap v43.1.0 〜 v44.0.0 — Language Expressiveness

Date: 2026-07-11
Status: 計画中（v43.0 完了後に詳細確定）

> **注**: 本スプリントは型推論 6 カテゴリを段階的に実装するため、v43.13 まで拡張（通常 9 版から 13 版）。

---

## 目標

v43.0「Real-Time Power」でリアルタイム処理基盤を完成させた。
このフェーズは **「型推論を強化し、ジェネリクスや戻り値型を手で書かなくてもコンパイラが補完してくれるようにする」** を実現する。

型推論 6 カテゴリ:
1. 戻り値型推論（Return type omission）
2. ジェネリック型引数推論（Call-site generic inference）
3. ラムダ引数型推論（Contextual lambda inference）
4. パイプライン型伝播（Pipeline stage typing）
5. 構造体リテラル推論（Structural inference）
6. 双方向型推論（Bidirectional / top-down）

---

## バージョン計画

### v43.1.0 — 戻り値型推論（Return type omission）✅ COMPLETE（2026-07-12）

```favnir
// 推論前（必須）
fn double(x: Int) -> Int { x * 2 }

// 推論後（省略可）
fn double(x: Int) { x * 2 }   // -> Int をブロック末尾式から推論
```

checker.fav・compiler.fav 両方に対応。省略時は末尾式の型を戻り値型として確定。

**完了条件**: Rust テスト 3 件（実績 2903 tests passed, 0 failed）

---

### v43.2.0 — 戻り値型推論: `fav check` 統合・E0410 系 ✅ COMPLETE（2026-07-12）

推論失敗時のエラー追加:
- E0410: ambiguous return type（末尾式から戻り値型が確定できない）
- E0411: return type mismatch（省略型と明示型の不一致）

`fav check --show-types` で推論された戻り値型を表示。

**完了条件**: Rust テスト 4 件（実績 2907 tests passed, 0 failed）

---

### v43.3.0 — ジェネリック型引数推論（Call-site inference）✅ COMPLETE（2026-07-12）

```favnir
fn identity<A>(x: A) -> A { x }

bind v <- identity(42)       // A = Int を引数から確定
bind s <- identity("hello")  // A = String を引数から確定
```

checker.fav の型変数単一化（`unify`）に call-site 推論パスを追加。

**完了条件**: Rust テスト 3 件（推定 2910 → 実績 2910 tests passed, 0 failed）

---

### v43.4.0 — ジェネリック推論: 曖昧ケース検出（E0412）✅ COMPLETE（2026-07-12）

複数の型変数が競合する場合に E0412 ambiguous type variable を報告。

**完了条件**: Rust テスト 4 件（推定 2914 → 実績 2914 tests passed, 0 failed）

---

### v43.5.0 — ラムダ引数型推論（Contextual lambda inference）✅ COMPLETE（2026-07-12）

```favnir
// 推論前（明示）
[1, 2, 3] |> List.map(|x: Int| x * 2)

// 推論後（List<Int> から x: Int が伝播）
[1, 2, 3] |> List.map(|x| x * 2)
```

パイプライン上流の型を下流ラムダの引数型に伝播。

**完了条件**: Rust テスト 3 件（推定 2917 → 実績 2917 tests passed, 0 failed）

---

### v43.6.0 — パイプライン型伝播（Pipeline stage typing）✅ COMPLETE（2026-07-12）

```favnir
stage Transform {
  bind rows  <- Csv.read("data.csv")           // Stream<Row>  — 推論
  bind nums  <- List.map(rows, |r| r.value)    // List<Float>  — 推論
  bind valid <- List.filter(nums, |v| v > 0.0) // List<Float>  — 推論
}
```

stage 内の中間型を明示せずとも checker.fav が伝播・確定。

**完了条件**: Rust テスト 3 件（推定 2920 → 実績 2920 tests passed, 0 failed）

---

### v43.7.0 — 構造体リテラル推論（Structural inference）

```favnir
// 名前付きレコードリテラルが関数シグネチャと一致する
type Point = { x: Int  y: Int }
fn make() -> Point { Point { x: 1  y: 2 } }
```

名前付きレコードリテラル（`TypeName { field: val ... }`）の型を関数戻り値・引数型と照合する。
既存の `ERecordLit → tname` 機構で動作するバリデーションリリース。

**スコープ外（→ v43.8.0）**: 匿名レコードリテラル `{ name: "Alice", age: 30 }`（`tname = ""`）の文脈推論。
リスト・タプルリテラルの文脈推論も v43.8.0 以降のスコープ。

**完了条件**: Rust テスト 2 件（推定 2922 → 実績 2922 tests passed, 0 failed）✅ COMPLETE（2026-07-13）

---

### v43.8.0 — 双方向型推論（Bidirectional / top-down）

期待型の下向き伝播。関数が `Int -> Bool` を期待すれば `|x| x > 0` の `x: Int` が確定。

```favnir
fn filter_positive(xs: List<Int>) -> List<Int> {
  List.filter(xs, |x| x > 0)   // x: Int は xs の要素型から伝播
}
```

**完了条件**: Rust テスト 3 件（推定 2925 → 実績 2925 tests passed, 0 failed）✅ COMPLETE（2026-07-13）

---

### v43.9.0 — `fav check --show-inference`

全式に推論された型を注釈表示。型推論のデバッグ支援。

**完了条件**: Rust テスト 2 件（推定 2927 → 実績 2927 tests passed, 0 failed）✅ COMPLETE（2026-07-13）

---

### v43.10.0 — `fav check --explain` 静的解説統合

`get_explain_text`（E0001〜E0021 静的テキスト）ベースの MVP 実装。
型チェックエラー発生時に解説テキストを出力。LLM 統合は将来バージョン。

**完了条件**: Rust テスト 2 件（推定 2929 → 実績 2929 tests passed, 0 failed）✅ COMPLETE（2026-07-13）

---

### v43.11.0 — Opaque type 完全化

```favnir
opaque type Token = String   // 外部からの String への暗黙 coerce を禁止
```

`opaque` contextual keyword を parser に追加。`TypeDef.is_opaque` フィールド追加。E0413（opaque coerce 禁止）を AST レベルで実装。checker.fav 統合は将来版。

**完了条件**: Rust テスト 3 件（推定 2932 → 実績 2932 tests passed, 0 failed）✅ COMPLETE（2026-07-13）

---

### v43.12.0 — W031〜W033 lint（冗長型注釈の警告）

- W031: 推論可能な戻り値型の明示的注釈
- W032: 推論可能なジェネリック型引数の明示
- W033: 推論可能なラムダ引数型の明示

**完了条件**: Rust テスト 3 件（推定 2935 → 実績 2935 tests passed, 0 failed）✅ COMPLETE（2026-07-13）

---

### v43.13.0 — Language Expressiveness cookbook + 安定化

- `site/content/cookbook/type-inference-guide.mdx`
- `site/content/docs/language/type-inference.mdx`
- `site/content/docs/language-expressiveness.mdx`

コードフリーズ（新規機能追加なし）。

**完了条件**: meta テスト 2 件（推定 2937 → 実績 2937 tests passed, 0 failed）✅ COMPLETE（2026-07-13）

---

### v44.0.0 — Language Expressiveness 宣言 ★クリーンアップ

**宣言文（暫定）**:

> 「戻り値型は省略でき、ジェネリクスは呼び出し側から推論される。
>  ラムダ引数はパイプライン上流の型から確定し、
>  `opaque type` で型の境界を守れる。
>
>  これが Favnir v44.0 — Language Expressiveness の姿である。」

**完了条件**:
- v43.1〜v43.13 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ 2937 + 4 = **2941**）✅ 実績 2941
- `v44000_tests` 4 件 pass（内訳: `cargo_toml_version_is_44_0_0` / `changelog_has_v44_0_0` / `milestone_has_language_expressiveness` / `readme_mentions_language_expressiveness`）✅
- `MILESTONE.md` に `"Language Expressiveness"` が含まれる ✅
- `★クリーンアップ`（`cargo clean`）完了 ✅

✅ COMPLETE（2026-07-13）

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v40.1-v45.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v42.1-v43.0.md`
- 達成宣言: `MILESTONE.md`
