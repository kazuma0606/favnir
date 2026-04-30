# Favnir v1 Gap Analysis

日付: 2026-04-30

## 前提

Favnir は Forge の置き換えではない。

- Forge:
  - application DSL
  - notebook / GraphQL / gRPC など周辺統合が強い
  - Rust ホスト依存が濃い
- Favnir:
  - explainable data-flow language
  - `type / bind / trf / flw / rune / effect`
  - artifact / VM / WASM / explain を中核にする

したがって、Forge の全機能を移植するのではなく、

1. v1.0.0 前に足りない基盤
2. Forge を参考にしつつ Favnir で別解を出すべき領域
3. 他言語と差別化するために前に出すべき領域

を分けて考える。

---

## 1. v1.0.0 前に足りない基盤

### 1-1. 型と実行系の完全な整合

まだ弱い点:

- VM parity が未完
  - 旧 `eval.rs` 由来の ignored test が残っている
- WASM backend は MVP は通るが、subset が狭い
- `exec --info` と `explain` は強いが、structured API としてはまだ薄い

v1 までに必要:

- VM を唯一の正規実行系にする
- ignored parity test をゼロにする
- `fav run`, `fav test`, `fav exec` が同じ意味論に揃う
- `explain` の出力を CLI 向け文字列だけでなく構造化 JSON でも返せるようにする

### 1-2. self-hosting の入口

今あるもの:

- language/runtime の分離方針
- `language/` と `selfhost/` の分離構想

足りないもの:

- selfhost する対象の優先順位
- subset の固定
- bootstrap の実行計画

必要:

- parser subset
- checker subset
- pretty-printer / formatter subset
- explain JSON generator

をどの順で `.fav` に移すか固定すること

### 1-3. LSP / editor 基盤

Forge は VS Code / notebook 寄りの統合が強かった。
Favnir は v1 で最低限ここが必要。

- hover
- diagnostics
- go to definition
- `trf/flw` symbol outline
- effect / type hover

特に Favnir は type/effect/flow の metadata が強いので、LSP は差別化に直結する。

### 1-4. package / rune / workspace の安定化

今は概念としては良いが、v1 ではルールの固定が必要。

- `namespace`
- `use`
- `public/internal/private`
- `rune`
- `workspace`
- `fav.toml`

が揺れていると、Veltra や registry 側が作りづらい。

### 1-5. structured testing / diagnostics

Forge の反省として、ホスト言語の test 実行に巻き込まれないことが重要。

v1 までに必要:

- `fav test` の結果を JSON で出せる
- `--jobs`, `--max-memory`, `--filter`, `--shard` の安定化
- trace / emit / effect 情報を test failure に結び付ける

---

## 2. Forge を参考にしつつ Favnir で別解を出す領域

### 2-1. Notebook

Forge の参考点:

- `.fnb`
- Markdown-first
- `.fnb.out.json`
- stdio kernel
- VS Code notebook 統合

Favnir / Veltra の別解:

- `.vnb`
- explain / trace / artifact を第一級出力にする
- notebook は製品名 `Veltra` 側の責務に寄せる
- language 側は notebook kernel protocol と structured explain/trace を出す

結論:

- notebook 体験は重要
- ただし Favnir 本体に UI を抱え込まない
- Forge の `.fnb` 思想はかなり参考になる

### 2-2. API integration

Forge の参考点:

- GraphQL
- gRPC

Favnir でそのままやる必要はまだない。
先に必要なのは:

- artifact 実行 API
- explain API
- notebook kernel API

その上で将来、

- `rune` を HTTP/GraphQL/gRPC の handler 資産として公開する

方向はあり得る。

つまり:

- Forge = transport first
- Favnir = flow/artifact first

### 2-3. Product packaging

Forge の参考点:

- notebook package
- surrounding packages (`forge-grpc`, `forge-graphql`)

Favnir の別解:

- `rune` を package 単位にする
- `.fvc` / `.wasm` を portable artifact にする
- registry / signing / policy check に寄せる

---

## 3. 他言語と比べたときの不足

### 3-1. Haskell / OCaml / F# 系に比べて不足

- pattern / ADT はあるが、抽象化の完成度はまだ低い
- cap/constraint system がまだ小さい
- stdlib の algebraic consistency はこれから
- error messages の完成度

必要:

- `Option/Result/List/Map` の combinator をさらに law-aware に揃える
- cap の運用例を増やす
- pattern ergonomics を安定化する

### 3-2. Rust に比べて不足

- package/tooling の完成度
- editor support
- diagnostics の強さ
- release engineering

ただし、ownership/lifetime を表面に出さないのは Favnir の正しい差別化。

### 3-3. TypeScript / Python に比べて不足

- onboarding の軽さ
- ecosystem の広さ
- notebook / REPL の親しみやすさ

補う方向:

- Veltra notebook
- stronger examples
- `fav explain`
- local UX の軽さ

### 3-4. Databricks / dbt / data tooling に比べて不足

- data source connectors
- team workflow
- scheduling
- lineage UI
- deployment story

これは言語本体ではなく Veltra 側で補う領域。

---

## 4. 差別化として強く押し出すべきもの

### 4-1. Explainable Flow Language

これは一番強い。

- `trf`
- `flw`
- effect
- artifact info
- explain
- trace

を一貫して持っている言語は珍しい。

打ち出し方:

- explainable
- inspectable
- safe to compose
- AI-friendly

### 4-2. Artifact-first developer experience

普通の言語は build artifact が opaque になりがち。
Favnir は逆に、

- `.fvc`
- `.wasm`
- `exec --info`

を前に出せる。

これは security / audit / review / AI generation に効く。

### 4-3. AI-native language metadata

Favnir では次を static metadata として持てる。

- type kind
- effect kind
- `trf/flw` dependencies
- entry reachability
- emitted events
- closures / variant ctors

この metadata は LSP にも Veltra にも AI 補完にも使える。

### 4-4. Notebook + artifact + explain の一体化

Databricks 風 notebook は他にもある。
Favnir/Veltra の独自性は、

- notebook
- explain
- artifact inspect
- typed flow

が一体であること。

### 4-5. Safe composition over clever abstraction

他の関数型言語が abstraction の強さへ寄るところで、
Favnir は

- 読める
- 説明できる
- effect が見える
- AI が壊しにくい

を優先する。

ここは明確な思想差になる。

---

## 5. v1.0.0 前に優先度が高い整理

### Must

- VM parity 完了
- selfhost subset 計画
- LSP 最小実装
- structured explain JSON
- structured test output
- rune/workspace/package の固定

### Strongly Recommended

- `exec --info` を JSON でも出せるようにする
- capability/cap と runtime effect の役割を docs で固定
- notebook kernel protocol を Favnir 側の責務として切り出す

### Later / Veltra side

- notebook UI
- BigQuery / GCS connectors
- hosted runner
- collaboration
- registry / signing / policy UI

---

## 6. 一言でいうと

Forge を参考にして Favnir が伸ばすべきなのは、

- notebook の思想
- product integration の分離
- surrounding tools の大切さ

であって、Forge の DSL そのものではない。

Favnir が v1.0.0 前に本当に強化すべきなのは:

- language/runtime の一貫性
- explain/trace/artifact の構造化
- editor/kernel/test の基盤

差別化として前に出すべきなのは:

- explainable data flow
- artifact-first tooling
- AI-native metadata
- Veltra へつながる notebook/runtime story
