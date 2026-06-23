# Roadmap v24.1.0 〜 v25.0.0 — Practical Self-Hosting

Date: 2026-06-18

## 目標

v24.0「VM in Favnir」で compiler / checker / CLI / VM すべてが Favnir で実装された。
Favnir は「Rust の力を借りながら、Rust を使わずに Favnir の世界を記述できる」状態に到達した。

この段階で「言語の設計が完成した」と宣言できる。
v25.0 以降は機能追加より**品質・安定性・エコシステム**の時代へ。

> **Practical Self-Hosting の定義（本プロジェクト固有）**
> 「コンパイラ・型チェッカー・CLI・VM 仕様が Favnir で書かれている」状態を指す。
> **VM の実行エンジン（バイトコード dispatch ループ）は Rust で永続維持する**。
> これは設計上の意図であり、制約ではない。

**完了条件（最終テスト）:**

```
# 1. 全 Rust テストが通る
cargo test: 全件 PASS

# 2. compiler.fav: Favnir VM 経由でコンパイルが動く
fav run --vm=self/vm.fav self/compiler.fav -- hello.fav → bytecode OK

# 3. checker.fav: Favnir VM 経由で型チェックが動く（fixture ベース）
fav run --vm=self/vm.fav self/checker.fav -- tests/bootstrap/hello.fav → diagnostics [] OK
fav run --vm=self/vm.fav self/checker.fav -- tests/bootstrap/type_error.fav → [E0001] OK

# 4. cli.fav: Favnir VM 経由で CLI が起動する（fixture ベース）
fav run --vm=self/vm.fav self/cli.fav -- --version → "favnir x.x.x" OK
fav run --vm=self/vm.fav self/cli.fav -- run tests/bootstrap/hello.fav → "Hello" OK

# 5. 4-stage bootstrap 検証（複数 fixture）
bytecode_A == bytecode_B == bytecode_C（6 fixture 全件） ✓
```

---

## 設計決定事項

| 項目 | 決定 |
|---|---|
| `fav spec` 出力形式 | Markdown（`SPEC.md`）/ HTML（`spec/index.html`）|
| 4-stage bootstrap 検証対象 | 6 fixture（hello / arithmetic / pattern_match / list_ops / closures / compiler.fav→hello） |
| bootstrap 比較式 | `bytecode_A == bytecode_B`（Stage 1/3）and `bytecode_B == bytecode_C`（Stage 3/4）|
| ベンチマーク回帰の閾値 | 5%（v24.3）。v20.1 の 10% より厳しく設定 |
| ベンチマーク公開先 | `https://bench.favnir.dev`（pushes ごとに更新） |
| v1.0 互換性ポリシー | v1.x: 後方互換保証。v2.0: 破壊的変更は 2 年前に deprecation warning |
| `--legacy` フラグ | v2.0 まで維持（削除しない） |
| Rune レジストリ目標 | 公式パッケージ 50+（主要クラウド全カバー / データフォーマット / ML 統合）|
| セキュリティ形式検証 | TLA+ / Coq でエフェクトシステムを形式的に証明 |
| テンプレートギャラリー | 4 種類（etl-csv-to-db / api-gateway / lambda-scheduled / distributed-etl）|

---

## バージョン計画

### v24.1 — 形式的仕様書生成（`fav spec`）

**テーマ**: vm.fav が完成したことで、言語のセマンティクスが Favnir コードで表現できる。
これを人間が読める仕様書として出力する。

```bash
fav spec --format markdown > SPEC.md
fav spec --format html > spec/index.html
```

仕様書には以下を含む:
- 型システムの形式的定義（型推論規則）
- opcode の動作仕様（decode → execute の対応表）
- エフェクトシステムの意味論
- パターンマッチの網羅性チェック規則

---

### v24.2 — 4-Stage Bootstrap 検証

**テーマ**: 現状の 3-stage bootstrap を VM も含めた 4-stage に拡張する。

```
Stage 1: Rust VM        + compiler.fav（元）→ hello.fav        → bytecode_A
Stage 2: Rust VM        + compiler.fav（元）→ compiler.fav     → compiler_artifact
Stage 3: Rust VM        + compiler_artifact → hello.fav        → bytecode_B
Stage 4: vm.fav（Favnir）+ compiler_artifact → hello.fav       → bytecode_C

検証:
  bytecode_A == bytecode_B  ← Stage 1/3 の一致（既存 3-stage bootstrap と同等）
  bytecode_B == bytecode_C  ← Stage 3/4 の一致（vm.fav が Rust VM と同じ結果を出せる証明）
  ∴ bytecode_A == bytecode_B == bytecode_C ✓

注: Stage 2 の compiler_artifact は「コンパイラ自身をコンパイルした成果物」であり
    bytecode_* の比較系列には入らない。Stage 3/4 の入力として使われる。
```

#### 検証 fixture 一覧

| fixture | 検証内容 |
|---|---|
| `tests/bootstrap/hello.fav` | 基本出力（文字列・IO） |
| `tests/bootstrap/arithmetic.fav` | 整数・浮動小数演算（opcode 網羅） |
| `tests/bootstrap/pattern_match.fav` | パターンマッチ（条件分岐 / ネスト） |
| `tests/bootstrap/list_ops.fav` | List の生成・map・filter（再帰） |
| `tests/bootstrap/closures.fav` | クロージャ・高階関数 |
| `self/compiler.fav` → `hello.fav` | コンパイラ自身を vm.fav で動かす |

最後の `compiler.fav → hello.fav` が通ること = vm.fav が「最小セルフホスト」を達成した証明。
全 fixture で `bytecode(Rust VM) == bytecode(vm.fav)` を CI で自動検証する。

> **実装時注記（v24.2.0）**: Stage 4（vm.fav + compiler_artifact → bytecode_C）は
> vm.fav Phase 6（ユーザー定義関数ディスパッチ）が未実装のためスコープ外とした。
> v24.2.0 では Stage 1–3 検証 infrastructure と 5 fixture の作成のみを実施。
> Stage 4 は Phase 6 完了後（v25.x 以降）に追加予定。

---

### v24.3 — 継続的パフォーマンス回帰検知

**テーマ**: v20.1 で整備したベンチマーク基盤を本格稼働させる。

```yaml
# GitHub Actions（毎 merge で実行）
- name: Benchmark regression check
  run: |
    fav run benchmarks/compare.fav \
      --baseline benchmarks/v24.0.0.json \
      --current benchmarks/latest.json \
      --threshold 5    # 5% 以上の劣化で CI fail
```

全ベンチマークの推移グラフを `https://bench.favnir.dev` で公開。
JSON が正本であり、`results.md` は `compare.fav --emit-md` が自動更新する。

---

### v24.4 — `v1.0` 後方互換性ポリシー確定

**テーマ**: v25.0 = v1.0 リリース候補。破壊的変更ポリシーを確定する。

```
v1.x: 後方互換性を保証（マイナーバージョンで破壊的変更なし）
v2.0: 破壊的変更は 2 年前に deprecation warning を出してから
SemVer: 完全準拠
--legacy: v2.0 まで維持
```

成果物:
- `STABILITY.md`（互換性ポリシーの文書化）
- deprecation warning 機能の実装（`@deprecated` アノテーション）
- semver lint（`fav lint` で v1.x 互換性違反を検出）

---

### v24.5 — Rune レジストリ成熟（公式パッケージ 50+）

**テーマ**: OSS コミュニティが Rune を公開できるエコシステムを整える。

```bash
fav search "bigquery"      # レジストリ検索
fav install bigquery       # インストール
fav publish my-rune        # 公開
```

公式パッケージ目標:
- 主要クラウドサービス全カバー（AWS / Azure / GCP / Snowflake）
- データフォーマット（Avro / ORC / Excel / XML）
- ML 統合（scikit-learn / HuggingFace API）

---

### v24.6 — セキュリティ審査（エフェクトシステム形式検証）

**テーマ**: `capability 引数がなければ純粋` を形式的に証明する。

- エフェクトシステムの形式的仕様（TLA+ / Coq）
- 外部審査（言語設計の専門家によるレビュー）
- CVE 対応プロセスの確立（`security@favnir.dev` + 90日 responsible disclosure）

---

### v24.7 — ドキュメントサイト v2

**テーマ**: 現状のサイト（site/）を完全リニューアル。

```
favnir.dev/
  docs/          言語リファレンス（自動生成）
  learn/         チュートリアル（入門〜応用）
  cookbook/      レシピ集（実際のユースケース）
  spec/          形式的仕様書（v24.1 の出力）
  bench/         ベンチマーク推移グラフ（v24.3 の出力）
  playground/    Playground v2（v21.6）
  packages/      Rune レジストリ（v24.5）
```

---

### v24.8 — `fav new` テンプレートギャラリー

**テーマ**: よくあるユースケースのテンプレートをワンコマンドで生成。

```bash
fav new --template etl-csv-to-db    myproject
fav new --template api-gateway      myapi
fav new --template lambda-scheduled myjob
fav new --template distributed-etl  mybigproject
```

各テンプレートには:
- サンプル `.fav` ファイル（動作する）
- `fav.toml`（適切なデフォルト設定）
- `README.md`（セットアップ手順）
- GitHub Actions CI 設定

---

## v25.0 — Practical Self-Hosting マイルストーン宣言

**完了条件:**

| コンポーネント | 実装 |
|---|---|
| コンパイラ（compiler.fav） | Favnir ✓ |
| 型チェッカー（checker.fav） | Favnir ✓ |
| CLI（cli.fav） | Favnir ✓ |
| **VM（vm.fav）** | **Favnir ✓（v24.0 達成）** |
| VM エンジン（実行基盤） | Rust（永続・設計上） |

> 「Favnir は Rust の力を借りながら、Rust を使わずに Favnir の世界を記述できる」

**最終テスト（全件 PASS が完了条件）:**

```bash
# 1. 全 Rust テストが通る
cargo test

# 2. compiler.fav: Favnir VM 経由でコンパイルが動く
fav run --vm=self/vm.fav self/compiler.fav -- hello.fav

# 3. checker.fav: fixture ベースで型チェックが動く
fav run --vm=self/vm.fav self/checker.fav -- tests/bootstrap/hello.fav
fav run --vm=self/vm.fav self/checker.fav -- tests/bootstrap/type_error.fav

# 4. cli.fav: fixture ベースで CLI が起動する
fav run --vm=self/vm.fav self/cli.fav -- --version
fav run --vm=self/vm.fav self/cli.fav -- run tests/bootstrap/hello.fav

# 5. 4-stage bootstrap（6 fixture 全件）
bytecode_A == bytecode_B == bytecode_C
```

---

## 参考リンク

- 前フェーズ: `versions/roadmap/roadmap-v23.1-v24.0.md`
- マスタースケジュール: `versions/roadmap-v20.1-v25.0.md`
