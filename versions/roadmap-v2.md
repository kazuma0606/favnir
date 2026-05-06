# Favnir ロードマップ v1.1.0 → v2.0.0

作成日: 2026-05-05

v1.0.0 完了後の進化の方針。
各バージョンは前のバージョンの完了を前提にして順番に進める。

---

## 方針

- **v1.x**: v1.0.0 との後方互換を保ちながら機能を追加する。
  旧キーワード（`trf` / `flw` / `cap`）は引き続き動作するが、段階的に非推奨扱いにする。
- **v2.0.0**: 破壊的変更（キーワードリネーム）を一括適用し、セルフホスト・マイルストーンを達成する。

---

## v1.1.0 — `interface` システム（`cap` の進化）

**テーマ**: 型の抽象化を `interface` キーワードで再設計し、自動合成（本体なし `impl`）を追加する

### 追加するもの

- `interface` キーワード（`cap` の後継。`cap` は非推奨警告を出すが動作は続ける）
- `impl InterfaceName for TypeName { ... }` — 手書き実装（本体あり）
- `impl InterfaceName for TypeName` — 自動合成（本体なし。全フィールドが interface を満たす場合のみ有効）
- `type T with Interface1, Interface2 { ... }` — 型宣言と同時に合成を宣言する糖衣構文
- `Gen` interface の定義（`Stat` ルーンの基盤。v1.5.0 で使用）
- `Field` interface の定義（代数構造の基盤。`fav-algebraic-structures.md` 参照）
- 標準 interface の移行: `Eq`, `Ord`, `Show` を `interface` として再定義
- `interface` の明示的な値渡し（暗黙解決なし）を型検査に組み込む

### 設計ドキュメント

- `dev/post-v1/roadmap/fav-abstraction-system.md`

### 完了条件

- `interface Show { show: Self -> String }` と `impl Show for Int { show = ... }` が動く
- `impl Show, Eq for UserRow`（本体なし）が全フィールドから自動合成される
- `type UserRow with Show, Eq { ... }` が上記のシンタックスシュガーとして機能する
- `cap` で書かれた既存コードに非推奨警告が出るが動作は継続する
- 既存の 321 テストが全て通る

---

## v1.2.0 — `invariant` + `std.states` ルーン

**テーマ**: 型にビジネスルールを埋め込み、バリデーションを型選択に変える

### 追加するもの

- `invariant <expr>` を `type` ブロック内に記述する構文
- コンストラクタ時の invariant 自動検査（違反時は `T!` を返す）
- コンパイル時に静的に証明できる invariant は静的検査に昇格
- `std.states` ルーン（標準 State 型の集合）:
  - 数値: `PosInt`, `NonNegInt`, `Probability`, `PortNumber`
  - 文字列: `Email`, `Url`, `NonEmptyString`, `Slug`
- `fav explain` で type の invariant 一覧を表示
- DB スキーマ出力時に invariant を `CHECK` 制約へ変換（`fav-db-schema-integration.md` 参照）

### 設計ドキュメント

- `dev/post-v1/roadmap/fav-standard-states.md`
- `dev/post-v1/roadmap/fav-db-schema-integration.md`（DB 連携部分）

### 完了条件

- `type Email { value: String; invariant String.contains(value, "@") }` が定義できる
- `Email.new("bad")` が `Err` を返し、`Email.new("a@b.com")` が `Ok(Email {...})` を返す
- `use std.states.PosInt` で `bind age: PosInt <- 25` が動く
- `fav explain` で `Email` の invariant 一覧が表示される

---

## v1.3.0 — `abstract stage` / `abstract seq`

**テーマ**: パイプライン構造そのものを抽象化し、型安全な依存注入を実現する

### 追加するもの

- `abstract trf Name: Input -> Output !Effect` — 実装なしの変換宣言（v2.0.0 で `abstract stage` にリネーム）
- `abstract flw Name<T> { slot: A -> B !Fx; ... }` — スロット付きパイプライン抽象（v2.0.0 で `abstract seq` にリネーム）
- スロット束縛構文: `flw X = Template<T> { slot <- ConcreteImpl }`
- 部分束縛: `PartialFlw<Template, { remaining_slots }>` 型
- 完全束縛後の effect 自動推論（スロットの effect の合成）
- `fav explain` でテンプレートと具体バインディングを表示
- `fav check` で未束縛スロットを警告

### 設計ドキュメント

- `dev/post-v1/roadmap/fav-abstract-flw.md`
- `dev/post-v1/roadmap/fav-abstraction-system.md`（`abstract stage` / `abstract seq` セクション）

### 完了条件

- `abstract flw DataPipeline<Row> { parse: String -> List<Row>!; save: List<Row> -> Int !Db }` が定義できる
- `flw UserImport = DataPipeline<UserRow> { parse <- ParseCsv; save <- SaveUsers }` が動く
- スロット型不一致でコンパイルエラーが出る
- 部分束縛中の `PartialFlw<...>` が `fav run` / `fav build` で実行不可としてエラーになる
- `fav explain` にテンプレート名と具体バインディングが表示される

---

## v1.4.0 — `fav explain --format json` + `fav bundle`

**テーマ**: コードの意味を機械可読な JSON で出力し、最小実行 artifact を生成する

### 追加するもの

- `fav explain --format json` — explain.json スキーマ出力:
  - `fns` / `stages` / `seqs` / `types` / `effects_used` / `emits` / `runes_used`
  - `reachable_from_entry` フラグ
  - `--focus stage` / `--focus seq` / `--focus types` オプション
- `fav bundle <file> [-o <out.fvc>]` — 到達可能性解析に基づく最小 artifact 生成
  - `--manifest` フラグで `manifest.json` を生成
  - `--explain` フラグで `explain.json` を同時生成（共通解析パスを使用）
- artifact の `.fvc` フォーマットに explain metadata セクションを追加
- `fav explain dist/app.fvc` — artifact からの explain 出力
- CI 連携: `fav explain diff` の基盤（スキーマの安定化が目的）

### 設計ドキュメント

- `dev/post-v1/roadmap/fav-explain-bundle.md`
- `dev/post-v1/roadmap/favnir-graph-explain.md`（Data Lineage 拡張）

### 完了条件

- `fav explain main.fav --format json` が有効な JSON を出力する
- `fav bundle main.fav -o dist/app.fvc --explain` が `.fvc` + `explain.json` を生成する
- `included` / `excluded` が正確に到達可能性を反映している
- `effects_required` が実行環境の capability チェックに使える

---

## v1.5.0 — `Stat` ルーン

**テーマ**: 型駆動のデータ生成と統計的推論を一つのルーンに統合する

### 追加するもの

- **プリミティブ生成**（旧 `random` 相当）:
  `Stat.int`, `Stat.float`, `Stat.bool`, `Stat.string`, `Stat.choice`, `Stat.generator`
- **分布駆動生成**: `Stat.normal`, `Stat.uniform`
- **型駆動生成**（`Gen` interface 依存、v1.1.0 が前提）:
  `Stat.one<T>`, `Stat.list<T>`, `Stat.rows<T>`
- **シミュレーション**: `Stat.simulate<T>` (noise パラメータで異常値混入)
- **統計的推論**: `Stat.profile<T>`, `Stat.drift<T>`
- **サンプリング**: `Stat.sample`, `Stat.sample_outliers`, `Stat.sample_edges`
- `fav check --sample N` との統合（実データの前提確認）
- `impl Gen for T`（本体なし）の自動合成ロジック（全フィールドが Gen を持つ型のみ）

### 設計ドキュメント

- `dev/post-v1/roadmap/stat-rune-architecture.md`
- `dev/post-v1/roadmap/validate-stat-integration.md`（`validate` との連携パターン）

### 完了条件

- `bind user <- Stat.one<UserRow>(seed: 42)` が `invariant` を満たす値を生成する
- `bind users <- Stat.list<UserRow>(1000, seed: 42)` が deterministic に動く
- `Stat.profile<UserRow>(real_data)` が `ProfileReport`（invariant 適合率を含む）を返す
- `fav check --sample 100` が実データの invariant 違反率を報告する

---

## v1.6.0 — `validate` ルーンファミリー

**テーマ**: バリデーションをパイプライン資産として表現し、テスト・開発・本番で再利用する

### 追加するもの

- `validate` 共通型: `ValidationError { path, code, message }`
- `validate.field` — フィールドレベル検証ルール:
  `Field.required`, `Field.min_len`, `Field.max_len`, `Field.range`, `Field.email`
- `validate.flow` — パイプライン・ドメイン検証:
  `Flow.validator<T>`, `Flow.field`, `Flow.cross`, `Flow.nested`, `Flow.each`, `Flow.when`,
  `Flow.validate`, `Flow.validate_all`
- `validate.db` — DB 行・CSV 行検証:
  `DbValidate.field`, `DbValidate.record`, `DbValidate.validate_row`
- `validate.flow` が `stage` として自然にパイプラインに組み込める設計

### 設計ドキュメント

- `dev/post-v1/roadmap/validate-rune-architecture.md`
- `dev/post-v1/roadmap/validate-stat-integration.md`
- `dev/post-v1/roadmap/validate-stat-favnir-style-examples.md`

### 完了条件

- `Flow.validator<Signup>() |> Flow.field(...) |> Flow.cross(...)` でバリデータが組み立てられる
- `stage ValidateSignup: Signup -> Signup! = |form| { Flow.validate(form, SignupValidator) }` が動く
- `DbValidate.record([...]) |> DbValidate.validate_row(row, ...)` が動く
- `Stat.list<Signup>(50)` で生成したデータが `Flow.validate_all` で検証できる

---

## v1.7.0 — `Task<T>` 非同期モデル

**テーマ**: `await` キーワードなしで非同期を `bind` に統合する

### 追加するもの

- `Task<T>` 型の定義（非同期計算の標準型）
- `async fn` / `async trf` / `async flw` 宣言（戻り値が `Task<T>` に変わる）
- `async` スコープ内での `bind` による `Task<T>` 自動解除（`await` 不要）
- `chain` による `Task<T>!` 一括処理（Task 解除 + Result 伝播）
- `Task.run` — 同期コンテキストからの明示実行
- `Task.all` / `Task.race` / `Task.timeout` — 並列実行 API
- `async fn main()` のランタイムサポート
- Effect システムとの統合（`async stage` は `!Network` 等と組み合わせる）

### 設計ドキュメント

- `dev/post-v1/roadmap/favnir-async.md`
- `dev/post-v1/roadmap/favnir-concurrency.md`

> **⚠ 専用メモ未作成**: 以下は `favnir-async.md` に設計意図はあるが、CLI・ランタイムの具体仕様が未定義。
> spec.md 作成時に補完が必要。
> - `async fn main()` のランタイム起動フロー（`Task.run` との関係）
> - `fav run` / `fav build` での async エントリポイント検出方法
> - `Task.timeout` のデフォルト値ポリシー

### 完了条件

- `async trf FetchText: Url -> String !Network = |url| { bind body <- IO.http_get(url); body }` が動く
- `bind (a, b) <- Task.all(FetchText(url1), FetchText(url2))` が並列実行される
- `async fn main()` でランタイムが Task を実行する
- `await` キーワードは存在しない（型エラーになる）
- `chain body <- IO.http_get(url)` で `Task<String>!` が一括処理される

---

## v2.0.0 — キーワードリネーム + セルフホスト・マイルストーン

**テーマ**: 破壊的変更を一括適用し、言語の最終形に近づける

### 追加・変更するもの

**言語の破壊的変更（v1.x との非互換）**:
- `trf` → `stage` への完全移行（`trf` は削除）
- `flw` → `seq` への完全移行（`flw` は削除）
- `cap` → 削除（v1.1.0 から非推奨化した `cap` を完全除去。`interface` のみ）
- `abstract trf` → `abstract stage`、`abstract flw` → `abstract seq`
- エラーコード体系の刷新: `E001`–`E040` → `E0100`–`E0999`（セルフホスト拡張向け）
- explain JSON の `kind` 値も `"trf"`→`"stage"`、`"flw"`→`"seq"` へ移行

**セルフホスト・マイルストーン**:
- パーサーの Favnir 移植（Rust VM 上で Favnir 製パーサーを実行）
- 型チェッカーの一部 Favnir 移植
- `fav explain compiler` — コンパイル工程の可視化

**ツールチェーン**:
- v1.x → v2.0.0 移行ガイド（`trf`/`flw`/`cap` を `stage`/`seq`/`interface` に変換する `fav migrate` コマンド）
- `langspec.md` v2.0.0 の全面改訂（SSS アーキテクチャ、Task<T>、invariant 等を正式記載）
- RELEASE_NOTES.md v2.0.0

> **⚠ 専用メモ未作成**: `fav migrate` の具体仕様が未定義。spec.md 作成時に補完が必要。
> - 変換対象: `trf`→`stage`、`flw`→`seq`、`cap`→`interface`、`abstract trf`→`abstract stage` など
> - 実行方法: `fav migrate <file>` / `fav migrate --in-place` / `fav migrate --dry-run`
> - エラー処理: 自動変換不可パターンの検出と手動対応ガイド出力
> - 移行ガイドドキュメント（`versions/v2.0.0/migration-guide.md`）の作成

### 設計ドキュメント

- `dev/post-v1/roadmap/fav-sss-architecture.md`（リネームの根拠）
- `dev/post-v1/roadmap/favnir-selfhost-plan.md`（セルフホスト戦略）
- `dev/post-v1/roadmap/fav-error-code-system.md`（エラーコード体系）
- `dev/post-v1/roadmap/favnir-post1-roadmap.md`（製品フェーズ全体像）

### 完了条件

- `stage` / `seq` / `interface` キーワードで書かれた `.fav` コードが動く
- `trf` / `flw` / `cap` キーワードはコンパイルエラーになる
- `fav migrate` で v1.x コードが v2.0.0 構文に自動変換される
- パーサーの一部が `.fav` で書かれ、Rust VM 上で動く
- 言語仕様書が新構文を完全に記述している

---

## バージョンと機能の対応表

| バージョン | テーマ | 主な追加 |
|---|---|---|
| v1.0.0 | 安定版 | LSP, WASM closure, rune install |
| v1.1.0 | `interface` システム | `interface`, `impl`(自動合成), `with` |
| v1.2.0 | `invariant` + `std.states` | 型制約, Email/PosInt 等 |
| v1.3.0 | `abstract stage/seq` | パイプライン抽象化, 依存注入 |
| v1.4.0 | explain JSON + bundle | `fav explain --format json`, `fav bundle` |
| v1.5.0 | `Stat` ルーン | 型駆動生成, 統計的推論 |
| v1.6.0 | `validate` ルーン | validate.field/flow/db |
| v1.7.0 | `Task<T>` 非同期 | `await` なし非同期, `Task.all/race` |
| v2.0.0 | 破壊的変更 + selfhost | `stage/seq/interface` リネーム, パーサー移植 |

---

## 実装順序の依存関係

```
v1.1.0 (interface)
  └─ v1.2.0 (invariant)         -- Gen interface が前提
       └─ v1.5.0 (Stat)         -- Gen interface が前提
            └─ v1.6.0 (validate)-- Stat との統合

v1.1.0 (interface)
  └─ v1.3.0 (abstract stage/seq)-- Field interface が前提

v1.3.0 (abstract stage/seq)
  └─ v1.4.0 (explain + bundle)  -- seq 構造の解析が前提

v1.7.0 (Task<T>)               -- 独立して開発可能（v1.3.0 以降推奨）

v1.x 全体
  └─ v2.0.0 (リネーム + selfhost)
```

---

## 補助メモ（ロードマップ未確定）

`dev/post-v1/` 配下にあるが、現時点でロードマップに未確定のドキュメント。
将来のバージョンへの昇格候補として管理する。

### アイデア候補（`dev/post-v1/ideas/`）

| ファイル | 内容 | 想定昇格先 |
|---|---|---|
| `fav-safe-cast.md` | 安全なキャスト設計 | 未定 |
| `fav-null-safety-and-option-ergonomics.md` | Option 利便性・`if bind` 糖衣構文 | v1.2.0 以降 |
| `fav-type-inference.md` | 型推論の方針 | v2.0.0 以降 |
| `favnir-open-questions.md` | 未決定設計問題 | 随時参照 |
| `favnir-next-candidates.md` | 次バージョン機能候補 | 随時参照 |
| `forge-syntax.md` | Forge との構文比較 | 参考資料 |

### ビジョン・規約（`dev/post-v1/vision/`）

| ファイル | 内容 |
|---|---|
| `fav-manifesto.md` | Favnir の設計原則 |
| `fav-coc-vision.md` | CoC ビジョン（Veltra との接続） |
| `fav-project-management.md` | プロジェクト管理方針 |
| `fav-directory-convention.md` | ディレクトリ構成規約 |
