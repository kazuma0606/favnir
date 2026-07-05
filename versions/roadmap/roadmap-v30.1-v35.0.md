# Favnir Master Roadmap — v30.1 〜 v35.0

Date: 2026-07-01
Status: 計画中（v30.0.0 完了時点）

---

## 背景と方針

v30.0.0「Ecosystem Maturity」の宣言をもって、Favnir は以下を達成した:

```
v26.0 — Rune Foundation       : コア Rune が本当に動く         ✓
v27.0 — Streaming Native      : ストリームが型安全に流れる      ✓
v28.0 — Data Lakehouse        : 現代のデータ基盤に溶け込む      ✓
v29.0 — Observability First   : パイプラインの内側が見える      ✓
v30.0 — Ecosystem Maturity    : コミュニティが Rune を育てる    ✓
```

v25.1〜v30.0 のスプリントでエコシステムを実質化した今、次の問いは
**「実際の案件で Favnir を使って何が起きるか」** である。

ここで重要な発見がある。言語機能の多くはすでに実装済みだ:

```
実装済み（v30.0.0 時点）
  ✓ 文字列リテラル match（Pattern::Lit::Str）
  ✓ alias 型エイリアス
  ✓ List.sort_by / group_by / flat_map / chunk / distinct / zip / unzip
  ✓ String.split / trim / replace / starts_with / ends_with / lines ...
  ✓ DateTime.now / parse / format / format_iso
  ✓ Or-pattern / List-pattern / forall（プロパティテスト）
  ✓ エフェクト推論（v18.1.0）
  ✓ 線形型 -o（v18.5.0）
  ✓ tap / inspect / assert_eq

未実装
  ✗ 境界付きジェネリクス T with Ord（制約の型チェック未実装）
  ✗ 行多相 Row Polymorphism
  ✗ スキーマ型 schema "postgres:table"
  ✗ AOT ネイティブバイナリ（Cranelift は依存に存在するが未接続）
  ✗ インクリメンタルコンパイル
  ✗ WASM 最適化
```

つまり次のフェーズは「機能追加」より **「品質・統合・実証」** が主軸である。

```
v31.0 — Real-World Readiness  : 「実案件で .fav が動く」
v32.0 — Language Polish       : 「書いたとき・デバッグするときが気持ちいい」
v33.0 — Language Power        : 「型で設計できる」（詳細はドッグフード後確定）
v34.0 — Performance & Tooling : 「本番で速い」（詳細はドッグフード後確定）
v35.0 — Production Ready      : 宣言マイルストーン
```

---

## バージョン命名規則

| 種別 | 意味 |
|---|---|
| **x.0.0** — マイルストーン宣言版 | 直前の x-1.1〜x-1.9 の成果を宣言 + **ビルドクリーンアップ実施（必須・例外なし）** |
| **x.1〜x.9** — 実装版 | 機能・品質改善を 1 バージョン 1 テーマで順次実装（クリーンアップ不要） |

---

## クリーンアップ規約

> **ルール: ★クリーンアップは本スプリントの x.0.0 全件（v31.0 / v32.0 / v33.0 / v34.0 / v35.0 の 5 件すべて）で必ず実施する。例外はない。**
>
> x.1〜x.9 の実装版では実施しない。マイルストーン版のみ対象とする。

バージョン一覧の `★クリーンアップ` は上記ルールの視覚的確認用マーカーである。
マーカーがないマイルストーン版は存在しない（もしあればドキュメントのバグ）。

## クリーンアップ手順（各 x.0.0 マイルストーン完了時に必ず実施）

対象: v31.0.0 / v32.0.0 / v33.0.0 / v34.0.0 / v35.0.0 — 計 5 回

マイルストーン版をリリースするたびに、以下のクリーンアップを実施して
CI と同じクリーンな状態を確認する。

```bash
# 1. ビルド生成物を完全削除（target/ は 40GB+ になりやすい）
cd /c/Users/yoshi/favnir/fav
cargo clean

# 2. クリーンビルド（CI と同一環境を確認）
cargo build 2>&1 | tail -3

# 3. テスト全件通過確認
cargo test 2>&1 | grep -E "test result|FAILED"

# 4. ビルドサイズ記録
du -sh target/
# → benchmarks/vXX.0.0.json の "build_size_gb" フィールドに記録

# 5. self-lint + self-fmt 確認
./target/debug/fav lint --deny-warnings --allow W017 --allow W018 --allow W019 self/compiler.fav
./target/debug/fav lint --deny-warnings --allow W012 --allow W017 --allow W018 --allow W019 self/checker.fav
./target/debug/fav fmt --check self/compiler.fav
./target/debug/fav fmt --check self/checker.fav
```

このクリーンアップにより:
- 増分ビルドキャッシュの汚染を防ぐ
- 新スプリントをクリーンな状態で開始できる
- CI との乖離を早期発見できる

---

## 破壊的変更なし原則

v25.0.0 で確定した STABILITY.md ポリシーを継続する。
v30.x〜v35.x（v2.x 相当）では破壊的変更を行わない。
既存コードは常に動き続ける。新機能はすべて **追加のみ** で提供する。

---

## v31.0 — Real-World Readiness

**テーマ**: 「実案件で .fav が動く」
**期間**: v30.1〜v30.9 → v31.0 マイルストーン宣言

### 背景

エコシステムは揃った。次の問いは「実際の案件 .fav プロジェクトが端から端まで動くか」だ。

マルチファイルプロジェクト・Rune import の組み合わせ・fav test のプロジェクト統合——
これらは仕様として実装されているが、実規模での動作検証が不足している。
まずビルドを軽量化し、テンプレートを実用レベルに引き上げ、
実データを使ったドッグフードパイプラインで「詰まるポイント」を記録する。

### 完了条件

1. `cargo build` 後の `target/` が `debug=0` で軽量化されている
2. `fav new --template postgres-etl my-project` で 4 ファイル構成が生成される
3. 生成されたプロジェクトで `fav check` / `fav run` / `fav test` が通る
4. ドッグフード用パイプライン（CSV → Postgres）が実データで動作する
5. `fav new --list` でテンプレート一覧が表示される
6. ドッグフードで発見したバグが修正済み

詳細: [roadmap/roadmap-v30.1-v31.0.md](roadmap-v30.1-v31.0.md)

---

## v32.0 — Language Polish

**テーマ**: 「書いたとき・エラーを見たとき・デバッグするときが気持ちいい」
**期間**: v31.1〜v31.9 → v32.0 マイルストーン宣言

### 背景

機能はある。しかし「初めて使うエンジニアが 30 分以内に動かせるか」という体験品質が
まだ不十分だ。特にエラーメッセージ・REPL・LSP の 3 点が開発体験の鍵を握る。

### 完了条件

1. `[E0001]` 等のエラーコードに `hint:` / `help:` / `note:` が付き、rustc スタイルで表示される
2. typo 候補（Levenshtein 距離）が提示される
3. `fav explain E0001` でエラーコードの説明が表示される
4. REPL で `:doc List.group_by` / `:load file.fav` / `:history` が動作する
5. LSP Inlay Hints（型推論結果のインライン表示）が動作する
6. `fav test --watch` が動作する
7. `fav check --all` でプロジェクト全体のクロスファイルエラーが表示される

詳細: [roadmap/roadmap-v31.1-v32.0.md](roadmap-v31.1-v32.0.md)

---

## v33.0 — Language Power

**テーマ**: 「型で設計できる」
**期間**: v32.1〜v32.9 → v33.0 マイルストーン宣言

> **注意**: v32.1〜v32.9 の詳細は v32.0 完了時点のドッグフード結果を見て確定する。
> 以下は現時点の大枠であり、優先順位は変動する可能性がある。

### 大枠

| 候補機能 | 内容 |
|---|---|
| 境界付きジェネリクス `T with Ord` | `fn max<T with Ord>(a: T, b: T) -> T` — 制約の型チェック実装 |
| 行多相 Row Polymorphism | `fn stamp<R with { id: Int }>(r: R) -> { ...R, ts: String }` |
| `where` 制約（関数引数） | `fn divide(a: Int, b: Int where { b != 0 }) -> Int` |
| スキーマ型 | `type User = schema "postgres:users"` — fav infer と統合 |
| 型駆動コード生成 | `fav generate type --from postgres users` |

詳細: [roadmap/roadmap-v32.1-v33.0.md](roadmap-v32.1-v33.0.md)

---

## v34.0 — Performance & Tooling

**テーマ**: 「本番で速い」
**期間**: v33.1〜v33.9 → v34.0 マイルストーン宣言

> **注意**: v33.1〜v33.9 の詳細は v33.0 完了時点の状況を見て確定する。

### 大枠

| 候補機能 | 内容 |
|---|---|
| AOT ネイティブバイナリ | `fav build --target native`（Cranelift 活用、deps に存在） |
| インクリメンタルコンパイル | 変更ファイルのみ再コンパイル（`~/.fav/cache/`） |
| ストリーミング評価 | `#[streaming(chunk_size=1000)]` でメモリ最小化 |
| Arrow 列指向統合 | stage 出力を Arrow RecordBatch として格納 |
| `fav run --precompiled` | Lambda コールドスタート 100ms 以下 |
| WASM 最適化 | サイズ 50% 削減、wasm-opt 統合 |

詳細: [roadmap/roadmap-v33.1-v34.0.md](roadmap-v33.1-v34.0.md)

---

## v35.0 — Production Ready

**テーマ**: 「Favnir で実案件のデータパイプラインを書く」が当たり前になった宣言
**期間**: v34.1〜v34.9 → v35.0 マイルストーン宣言

> **注意**: v34.1〜v34.9 の詳細は v34.0 完了時点の状況を見て確定する。

### 大枠

| 候補 | 内容 |
|---|---|
| 実案件デモ | 500 行以上の実データパイプライン（複数 Rune 使用）が end-to-end で動く |
| ドキュメントサイト v4 | Playground + cookbook 50 本 + ベンチマーク比較グラフ |
| ベンチマーク公開 | Python/Spark との実測比較 |
| セキュリティ審査 v2 | エフェクトシステム形式検証 + 外部 OSS ライセンス確認 |
| 安定化 | v30.1〜v34.9 で発見した残存問題の解消 |

詳細: [roadmap/roadmap-v34.1-v35.0.md](roadmap-v34.1-v35.0.md)

---

## バージョン一覧

```
v30.0.0  Ecosystem Maturity 宣言（完了）
│
├── v30.1  ビルド軽量化（debug=0 / split-debuginfo=off）
├── v30.2  postgres-etl テンプレート v2（4ファイル構成）
├── v30.3  マルチファイルプロジェクト E2E 検証
├── v30.4  Rune import マルチファイル動作検証
├── v30.5  ドッグフード用サンプル実装（CSV→Postgres）
├── v30.6  fav test プロジェクト統合
├── v30.7  fav run エラー時スタックトレース改善
├── v30.8  fav new --list コマンド
├── v30.9  ドッグフード発見修正
▼
v31.0.0  Real-World Readiness 宣言 ★クリーンアップ
│
├── v31.1  エラーメッセージ v2（rustc スタイル hint/note/help）
├── v31.2  typo 候補（Levenshtein）+ エラーコード URL
├── v31.3  fav explain E0001 コマンド
├── v31.4  REPL 品質向上（:doc/:load/:history/タブ補完）
├── v31.5  LSP Inlay Hints（型推論結果インライン表示）
├── v31.6  fav test --watch
├── v31.7  fav check --all（クロスファイル型エラー）
├── v31.8  fav scaffold（既存プロジェクトへのコード追加）
├── v31.9  ドッグフード修正 vol.2
▼
v32.0.0  Language Polish 宣言 ★クリーンアップ
│
├── v32.1  境界付きジェネリクス T with Ord（※ドッグフード後確定）
├── v32.2  行多相 Row Polymorphism（※同上）
├── v32.3  where 制約（関数引数）（※同上）
├── v32.4  スキーマ型（※同上）
├── v32.5〜v32.9  ドッグフード結果で決定
▼
v33.0.0  Language Power 宣言 ★クリーンアップ
│
├── v33.1  AOT ネイティブバイナリ（Cranelift）（※状況で確定）
├── v33.2  インクリメンタルコンパイル（※同上）
├── v33.3  ストリーミング評価（#[streaming]）（※同上）
├── v33.4  Arrow 列指向統合（※同上）
├── v33.5〜v33.9  状況で決定
▼
v34.0.0  Performance & Tooling 宣言 ★クリーンアップ
│
├── v34.1  実案件デモ実装（※状況で確定）
├── v34.2  ドキュメントサイト v4（※同上）
├── v34.3  ベンチマーク公開（※同上）
├── v34.4  セキュリティ審査 v2（※同上）
├── v34.5〜v34.9  安定化・最終調整
▼
v35.0.0  Production Ready 宣言 ★クリーンアップ
```

---

## 設計原則（全バージョン共通）

1. **後方互換性**: 既存のパイプラインコードは常に動作する
2. **ドッグフード優先**: 機能追加より「実際に動かす」を先に
3. **クリーンアップ習慣**: 各マイルストーンでビルド生成物をリセット
4. **品質 > 機能数**: 1 機能をきちんと動かすほうが 3 機能を半端に入れるより価値が高い
5. **セルフホスト維持**: `compiler.fav` / `checker.fav` は常に最新の言語機能で書かれた状態を維持する

---

## 参考リンク

| ファイル | 目的 |
|---|---|
| [roadmap-v30.1-v31.0.md](roadmap-v30.1-v31.0.md) | Real-World Readiness 詳細計画 |
| [roadmap-v31.1-v32.0.md](roadmap-v31.1-v32.0.md) | Language Polish 詳細計画 |
| [roadmap-v32.1-v33.0.md](roadmap-v32.1-v33.0.md) | Language Power 詳細計画（ドッグフード後更新） |
| [roadmap-v33.1-v34.0.md](roadmap-v33.1-v34.0.md) | Performance & Tooling 詳細計画（同上） |
| [roadmap-v34.1-v35.0.md](roadmap-v34.1-v35.0.md) | Production Ready 詳細計画（同上） |
| [roadmap-v29.1-v30.0.md](roadmap-v29.1-v30.0.md) | 前スプリント（参照用） |
