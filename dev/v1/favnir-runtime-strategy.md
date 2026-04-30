# Favnir Runtime Strategy

更新日: 2026-04-26

## 前提

Favnir は Rust に思想的に依存する必要はない。

ただし、一人で開発する以上、次の点は別途担保する必要がある。

- メモリ安全
- sandbox 性
- 配布性
- 実行時の権限制御
- 小さい trusted core

したがって、Favnir は「Rust の上の言語」である必要はないが、「何らかの安全なホスト基盤の上で動く言語」である方がよい。

## 基本方針

最も現実的な構成は次の 4 層。

1. Rust host
2. Favnir frontend
3. WASM backend
4. capability runtime

これに加えて、全体を通して trusted core を小さく保つ。

## 1. Rust Host

Rust は Favnir の意味を持つ場所ではなく、起動と境界処理を担うホストとする。

役割:

- CLI
- ファイル読み込み
- module / package 解決
- コンパイル起動
- 実行環境初期化
- capability 注入
- sandbox 設定

重要なのは、Rust を「言語本体」ではなく「launcher / host」と位置付けること。

つまり:

- Rust = host
- Favnir = language core

## 2. Favnir Frontend

ここが Favnir 本体。

役割:

- parser
- AST
- 型チェック
- effect チェック
- `stage / flow` 合成検査
- ADT / pattern match 解決
- typed IR 生成

この層は将来的にセルフホスト対象にしやすいよう、最初から Rust の型やランタイム都合と切り離して設計する。

## 3. WASM Backend

実行ターゲットは、最初から WASM を強く意識する。

役割:

- typed IR から WASM への lowering
- pure stage の実行単位化
- effect stage の host 呼び出し接続
- module 単位の安全な実行

利点:

- sandbox しやすい
- 実行環境を閉じ込めやすい
- Rust 以外の host にも載せやすい
- browser / edge / server へ広げやすい

Favnir の本質は Rust 上にあるのではなく、WASM へ安全に落とせる typed pipeline 言語として成立させることにある。

## 4. Capability Runtime

副作用は直接開放せず、capability 経由でのみ使えるようにする。

想定 capability:

- `Db`
- `Io`
- `File`
- `Network`
- `Emit<Event>`
- `Clock`
- `Random`

Favnir コードは OS や外部資源を直接触らない。  
代わりに、

- この stage は `Db` を要求する
- この flow は `Emit<UserCreated>` を行う
- この処理は `File` を読む

と宣言する。

ホスト側はその要求を見て、明示的に capability を注入する。

## メモリ管理の方針

Favnir では GC は入れない前提で考える。

理由:

- 個人開発で GC 実装まで抱えるのは重い
- 言語の価値は GC そのものではなく、typed pipeline と effect にある
- runtime を小さく保ちたい

ただし、C/C++ 的にユーザーへ直接メモリ管理を委ねる方向にも行かない。

方針は次の通り。

### 1. ユーザーにはメモリ管理を見せない

Favnir の表面構文には:

- ownership
- lifetime
- manual free
- pointer arithmetic

を出さない。

ユーザーが考えるべきなのは:

- immutable
- effect
- capability 境界

までに留める。

### 2. compiler / frontend 内部は arena を使う

次のような中間データは arena / region で扱うのがよい。

- AST
- typed IR
- constraint graph
- diagnostic 補助情報

理由:

- 実装が比較的単純
- checker との相性が良い
- まとめて解放しやすい

### 3. 実行時値は immutable 中心で扱う

pure value は immutable を前提にする。

候補:

- cheap copy
- shared immutable value
- 必要に応じて refcount

ただし、初期実装ではまず値の種類を絞り、小さい trusted runtime で扱える範囲に保つ方がよい。

### 4. 外部資源は host ownership に寄せる

重い資源は Favnir 側で所有しない方がよい。

対象:

- DB connection
- file handle
- network client
- clock / random source

Favnir 側は capability や handle を通して触るだけにする。

### 5. 実行単位ごとに region cleanup する

`flow` や `job` の実行単位ごとに確保したメモリをまとめて破棄できる設計がよい。

これにより:

- GC を持たずに済む
- ライフタイムを表面へ出さずに済む
- cleanup の見通しが良くなる

## ownership / lifetime の扱い

Rust では ownership / lifetime が安全性に大きく寄与しているが、Favnir ではこれを表面構文に露出させない方がよい。

Favnir の安全性は、次の組み合わせで作る。

- immutable data
- lexical scope
- immutable closure capture
- capability 境界
- region / arena cleanup
- host-managed resources

つまり、所有権の概念は runtime / compiler の内部で吸収し、ユーザーには露出しない。

## closure capture の runtime 方針

クロージャは lexical scope から値を capture できるが、初期段階では制限を強くしておく。

方針:

- capture は immutable のみ
- mutable capture はなし
- capability や重い資源の暗黙 capture は避ける
- closure environment は小さく保つ

これにより、GC なしでも実装しやすくなる。

## Trusted Core

信頼するべき実装はできるだけ小さくする。

含めるべき範囲:

- parser / checker の最小正当性
- typed IR から実行単位への変換
- capability 境界
- runtime の effect dispatch
- sandbox 制御

含めない方がよいもの:

- 巨大な標準ライブラリ
- 高レベルな補助機能
- app 固有の業務ロジック

方針:

- trusted core は小さく
- 上位機能は library / userland へ逃がす
- 境界だけを厳密に守る

## 安全性の説明

Favnir の安全性は、単に「Rust 製だから安全」と説明するべきではない。

説明軸は 3 つに分ける。

### 1. 言語安全性

- immutable
- 再代入禁止
- effect 明示
- typed composition

### 2. runtime 安全性

- capability-based execution
- sandboxed host calls
- 明示的な権限制御

### 3. 実装基盤安全性

- Rust や WASM のメモリ安全性
- 小さい trusted core

この 3 層で説明すれば、「一人開発なのにどう安全性を担保するのか」という問いに答えやすい。

## 全体像

実行の流れは次のイメージ。

```text
Favnir source
  -> parser / checker / typed IR
  -> WASM lowering
  -> sandboxed execution
  -> host capabilities (Db, File, Network, Emit...)
```

Rust は主に最後の host と周辺ツールを担当する。  
言語意味はそれより前段に閉じるべき。

## 推奨実装順

最初の実装順は次の通り。

1. Rust host で CLI を作る
2. Favnir frontend を独立設計する
3. typed IR を定義する
4. pure subset を WASM に落とす
5. capability runtime を最小実装する
6. `Emit<Event>`、`Db`、`File` などを順次追加する

最初から全部やる必要はない。  
まずは `Pure` と `Emit<Event>` を中心に回せれば十分。

## 結論

Favnir は Rust に思想的に依存しない。

ただし、安全性と現実的な開発速度のために、何らかの小さく信頼できるホスト基盤には依存した方がよい。

現時点で最も自然な形は次の構成。

- Rust host
- Favnir frontend
- WASM backend
- capability runtime
- small trusted core

この構成なら、言語としての独立性と、実装としての現実性を両立できる。
