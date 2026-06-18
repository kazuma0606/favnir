# v20.0.0 — Production Performance マイルストーン宣言 実装計画

## 作業順序

### Step 1: Cargo.toml バージョン更新
`fav/Cargo.toml` の `version` を `"19.8.0"` → `"20.0.0"` に変更。

### Step 2: CHANGELOG.md 更新
既存 CHANGELOG.md の先頭（v19.0.0 エントリの上）に v19.1.0〜v19.8.0 の全エントリを挿入。
各エントリ形式: `## [X.Y.Z] - 2026-06-17`

### Step 3: README.md 更新
- 現在のバージョン表記を v20.0.0 に更新
- Production Performance セクションを追加（streaming/AOT/Arrow/precompiled 実績）
- ベンチマーク結果の参考値を記載
- バージョン履歴表を更新

### Step 4: site/content/docs/performance/ 作成
6 つの MDX ファイルを新規作成:
1. `streaming.mdx`
2. `native-build.mdx`
3. `incremental.mdx`
4. `arrow.mdx`
5. `precompiled.mdx`
6. `profiling.mdx`

### Step 5: benchmarks/ ディレクトリ作成
3 ファイルを新規作成:
1. `benchmarks/10gb_csv.fav`
2. `benchmarks/lambda_coldstart.sh`
3. `benchmarks/results.md`

### Step 6: v200000_tests 追加
`fav/src/driver.rs` に `v200000_tests` モジュール（5件）を追加。
`version_is_19_8_0` に `#[ignore]` を追加。

### Step 7: テスト確認
`cargo test v200000` — 5/5 PASS を確認。
`cargo test` — リグレッションなし確認。

---

## リスク・注意点

- CHANGELOG.md は既存エントリを壊さないよう先頭挿入
- README.md はマークダウン構造を維持しつつ追記
- `benchmarks/10gb_csv.fav` は実行環境に 10GB ファイルが不要な参考実装として記述
- `lambda_coldstart.sh` は実際の AWS リソース不要な説明スクリプト
- `performance/*.mdx` は `tools/precompiled.mdx` / `tools/profiling.mdx` とコンテンツが重複しないよう、performance 視点でまとめる
