# Favnir エフェクトシステム 形式的仕様

## 概要

Favnir のエフェクトシステムは「capability 引数がなければ純粋」という原則に基づく。
本ドキュメントは、この原則を形式的に記述する。

---

## Capability 公理（Axiom）

**公理 1: 純粋性（Purity）**

```
fn f: A → B（エフェクト宣言なし）⊢ f は参照透明（referentially transparent）
```

`!Effect` を宣言しない関数はいかなる副作用も持たない。
同じ引数で呼び出された場合、常に同じ結果を返す。

**公理 2: 効果の伝播（Effect Propagation）**

```
fn f: A → B !E  かつ  fn g が f を呼び出す  ⟹  g は !E を宣言しなければならない
```

副作用のある関数を呼び出す関数は、その副作用を自身のシグネチャで宣言する義務がある。

**公理 3: 能力の封じ込め（Capability Confinement）**

```
!E を宣言しない関数からは !E エフェクトを発生させることができない
```

これが「capability 引数がなければ純粋」の直接的な表現である。

**公理 4: 合成（Composition）**

```
fn f: A → B !E₁  かつ  fn g: B → C !E₂  ⟹  f |> g : A → C !(E₁ ∪ E₂)
```

パイプラインで合成された関数のエフェクトは各ステップのエフェクトの和集合になる。

---

## 推論規則（Inference Rules）

```
[T-Pure]
  Γ ⊢ e : τ,  effects(e) = ∅
  ────────────────────────────
  Γ ⊢ fn e : τ  （純粋関数）

[T-Effect]
  Γ ⊢ f : A → B !E,  Γ ⊢ g calls f
  ────────────────────────────────────
  Γ ⊢ g must declare !E

[T-Compose]
  Γ ⊢ f : A → B !E₁,  Γ ⊢ g : B → C !E₂
  ──────────────────────────────────────────
  Γ ⊢ f |> g : A → C !(E₁ ∪ E₂)
```

---

## W021 Lint ルールとの対応

公理 2（Effect Propagation）および公理 3（Capability Confinement）の
コード内検証として W021 `pure_fn_calls_effectful` を実装している。

```favnir
fn fetch_data(url: String) -> String !Http { ... }

// W021: pure function `process` calls effectful function `fetch_data`
// — declare the effect or mark `process` as effectful
fn process(url: String) -> String { fetch_data(url) }
```

W021 は `fav lint` によって自動検出される。

---

## エフェクト一覧

| エフェクト | 意味 |
|---|---|
| `!Io` | 標準入出力 |
| `!File` | ファイル読み書き |
| `!Http` | HTTP 通信 |
| `!Db` | データベース（汎用）|
| `!DbRead` | データベース読み取り |
| `!DbWrite` | データベース書き込み |
| `!Network` | 汎用ネットワーク |
| `!Llm` | LLM API 呼び出し |
| `!Snowflake` | Snowflake 操作 |
| `!Gcp` | Google Cloud サービス |
| `!Stream` | Kafka / メッセージストリーム |
| `!Rpc` | gRPC 呼び出し |
| `!Checkpoint` | 増分処理チェックポイント |
| `!Trace` | 分散トレーシング |
| `!PipelineState` | パイプライン分散ステート |

---

## 外部審査（External Audit）依頼事項

本仕様の正式な機械検証（TLA+ / Coq）は v25.0 前後を目標に実施予定。

審査依頼事項:
- **無矛盾性（Consistency）**: 公理間に矛盾がないこと
- **健全性（Soundness）**: 型付け可能なプログラムはランタイムで意図しない副作用を起こさない
- **完全性（Completeness）**: 意図したすべての副作用がエフェクトとして宣言される

---

## v34.x Context 移行との関係

v34.5 以降で `!Effect` アノテーションを廃止し Capability Context（ctx パラメータ）に移行する予定。
ctx 移行後も公理 1〜4 は変形なく成立する:

- ctx フィールドへのアクセスが「capability を保有する」条件に相当
- ctx を持たない関数は引き続き純粋（公理 1 が適用される）
- W021 は ctx ベースの実装に対しても適用可能（v34.5 で更新予定）

v34 セキュリティ審査（v34.4.0）時点では `!Effect` 構文が現役であり、
W021 による形式検証は正常に動作することを確認した。
ctx 移行完了後に本セクションを更新する。
