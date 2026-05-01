# Favnir v1.0.0 Done Definition

日付: 2026-04-30

## 方針

`v1.0.0` は「すべてを入れる版」ではない。

目的は:

- 言語仕様を固定する
- 実行系を一貫させる
- self-hosting への入口を作る
- tooling の最小完成形を揃える

Veltra や cloud/notebook/product 拡張は、その後の開発で扱う。

---

## v1.0.0 Done Definition

### 1. 言語仕様

以下が固定されていること:

- `type`
- `bind`
- `trf`
- `flw`
- `rune`
- effect
- `T? / T!`
- `chain`
- `cap`
- `async/await`
- `public / internal / private`
- `namespace / use / workspace`

つまり、これ以上コア構文を大きく揺らさない状態。

### 2. 実行系の一貫性

以下が同じ意味論で動くこと:

- `fav run`
- `fav test`
- `fav build`
- `fav exec`
- `fav build --target wasm`

具体的には:

- VM が正規実行系
- `.fvc` artifact が安定
- `.wasm` backend が MVP として安定
- ignored parity test を減らし、実行差異を意識的に管理できる

### 3. Tooling

最低限そろっていること:

- `fav fmt`
- `fav lint`
- `fav test`
- `fav explain`
- `fav exec --info`

加えて:

- structured diagnostics
- structured test result
- structured explain output

の入口があること

### 4. self-hosting 入口

v1.0.0 で完全 self-host は必須ではない。
ただし、入口は必要。

必要条件:

- selfhost 対象の優先順位が決まっている
- subset が定義されている
- bootstrap 方針が決まっている

最低限、次のどれかが `.fav` 側へ移り始めていると強い。

- formatter subset
- explain subset
- parser subset

### 5. Editor / LSP 最小実装

以下の最小セット:

- hover
- diagnostics
- go to definition
- symbol outline

Favnir は type/effect/flow metadata が強みなので、LSP は v1 に入る価値が高い。

### 6. Package / project model

以下が固定されていること:

- `rune`
- `workspace`
- `fav.toml`
- module / namespace 解決
- 可視性の解決順

### 7. Documentation

v1.0.0 時点で必要:

- core spec
- examples
- CLI reference
- roadmap
- migration / design notes

特に「何ができて何がまだできないか」が明確であること。

---

## v1.0.0 に入れすぎないもの

### 言語本体では後回し

- richer constraint system
- macro
- full coroutine model
- field-level visibility
- advanced effect polymorphism
- transport-specific syntax

### 製品化では後回し

- notebook UI
- shared cloud workspace
- BigQuery / GCS first-class connectors
- collaboration
- scheduler
- registry/signing UI
- artifact marketplace

---

## 一言でいうと

`v1.0.0` は:

> Favnir を「使える言語」として締める版
