# Roadmap v41.1.0 〜 v42.0.0 — Type Precision

Date: 2026-07-11
Status: 計画中（v41.0 完了後に詳細確定）

---

## 目標

v41.0「Streaming Foundations」でウィンドウ・Watermark 基盤を整備した。
このフェーズは **「Refinement type・パターン強化・Row polymorphism により、型でデータの意味を精緻に表現できるようにする」** を実現する。

---

## バージョン計画

### v41.1.0 — Refinement type 基盤 ✅ COMPLETE（2026-07-11）

```favnir
type Age  = Int    where (>= 0 && <= 150)
type Name = String where (len > 0 && len < 256)

fn greet(name: Name) -> String { "Hello, " ++ name }
```

parser・checker.fav に `where` 節の型制約構文を追加。

**完了条件**: Rust テスト 3 件（推定 2843 tests passed, 0 failed）

---

### v41.2.0 — Refinement type `fav check` 統合・E0404 系 ✅ COMPLETE（2026-07-11）

refinement invariants 収集基盤整備・エラーコード追加:
- E0404: refinement 条件違反（※E0401/E0402/E0403 は SLA アノテーション系として使用済み）
- E0405: ambiguous refinement type
- E0406: refinement 条件の型不一致

checker.fav TypeDef に `invariants` フィールド追加 + `check_refinement_alias` 統合。
ast_lower_checker.rs に `invariants` フィールド追加（空リスト）。
（実際の違反検出は v41.3.0 以降）

**完了条件**: Rust テスト 3 件（推定 2851 tests passed, 0 failed）

---

### v41.3.0 — タプルパターン match

```favnir
match (status, count) {
  ("ok", 0) -> "empty ok"
  ("ok", n) -> "ok: " ++ Int.to_string(n)
  (err, _)  -> "error: " ++ err
}
```

parser にタプル式・パターンのデシュガー対応を追加（`(a,b)` → RecordConstruct / `(p1,p2)` → Pattern::Record）。
checker.fav に設計コメント追加（fav check 統合は v41.4.0 以降）。

**完了条件**: Rust テスト 3 件（推定 2856 tests passed, 0 failed）

---

### v41.4.0 — ガード付き match ✅ COMPLETE（2026-07-11）

```favnir
match score {
  n if n >= 90 -> "A"
  n if n >= 70 -> "B"
  _            -> "C"
}
```

`if` ガード節を match アームに追加。checker.fav の網羅性チェックに対応。

**完了条件**: Rust テスト 3 件（推定 2859 tests passed, 0 failed）

---

### v41.5.0 — Row polymorphism 強化 ✅ COMPLETE（2026-07-11）

record spread の checker.fav 統合（`ERecordSpread` / `TeRecord` バリアント追加）。
`sv("()")` lowering バグ修正。RecordDiff は v42.0+ スコープ外。

```favnir
fn extend_user(u: { name: String }) -> { name: String, active: Bool } {
  { ..u, active: true }
}
```

**完了条件**: Rust テスト 3 件（推定 2862 tests passed, 0 failed）

---

### v41.6.0 — Newtype 自動 impl ✅ COMPLETE（2026-07-11）

```favnir
type Kg(Float)     // + / * / - を Float から自動継承
type Meter(Float)
```

`type Name(Inner)` 宣言に対して算術演算子の自動委譲を生成。

**完了条件**: Rust テスト 3 件（推定 2865 tests passed, 0 failed）

---

### v41.7.0 — W030 lint ✅ COMPLETE（2026-07-11）

refinement 条件の冗長ガード検出。
例: `type PositiveInt = Int where |v| v >= 0` の変数に `if x >= 0` ガードは W030。

**完了条件**: Rust テスト 2 件（推定 2867 tests passed, 0 failed）

---

### v41.8.0 — Type Precision cookbook ✅ COMPLETE（2026-07-11）

- `site/content/cookbook/refinement-types.mdx` — 新規作成
- `site/content/docs/language/refinement-types.mdx` — type alias refinement + W030 セクション追加

**完了条件**: Rust テスト 1 件（推定 2868 tests passed, 0 failed）

---

### v41.9.0 — v42.0 前調整・安定化 ✅ COMPLETE（2026-07-12）

コードフリーズ（新規機能追加なし）。`site/content/docs/type-precision.mdx` 新規作成。

**完了条件**: meta テスト 2 件（推定 2870 tests passed, 0 failed）

---

### v42.0.0 — Type Precision 宣言 ★クリーンアップ ✅ COMPLETE（2026-07-12）

**宣言文（暫定）**:

> 「`type Age = Int where (>= 0)` で値の意味を型に刻める。
>  タプルパターンとガード付き match でより精緻な分岐が書ける。
>  Newtype は内側の型の演算を自動継承する。
>
>  これが Favnir v42.0 — Type Precision の姿である。」

**完了条件**:
- v41.1〜v41.9 の全機能が動作する
- `cargo test` 全通過（failures=0 かつテスト数 ≥ 2870 + 4 = **2874**）
- `v42000_tests` 4 件 pass（内訳: `cargo_toml_version_is_42_0_0` / `changelog_has_v42_0_0` / `milestone_has_type_precision` / `readme_mentions_type_precision`）
- `MILESTONE.md` に `"Type Precision"` が含まれる
- `★クリーンアップ`（`cargo clean`）完了

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v40.1-v45.0.md`
- 前サブスプリント: `versions/roadmap/roadmap-v40.1-v41.0.md`
- 達成宣言: `MILESTONE.md`
