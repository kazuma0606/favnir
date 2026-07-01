# Roadmap v34.1.0 〜 v35.0.0 — Production Ready

Date: 2026-07-01
Status: 骨格確定（詳細は v34.0 完了後に更新）

---

## 目標

v34.0「Performance & Tooling」で「本番で速い」を実現した。
このフェーズは **「Production Ready の宣言」** だ。

> **Production Ready の定義（本プロジェクト固有）**
> 「実際のデータエンジニアリング案件で Favnir を選択できる状態。
>  500 行以上の実データパイプラインが end-to-end で動き、
>  エラーが起きたときに原因を特定して修正できる。
>  ドキュメントを読めば新しいエンジニアが 1 日で Favnir を使い始められる」

v35.0 は「新機能を入れる」マイルストーンではない。
v30.1〜v34.9 で積み上げた成果を **「実案件で使える」レベルに確認・安定化する** マイルストーンだ。

---

## ⚠️ 重要：v34.0 完了後に更新が必要

このファイルは **骨格のみ** である。

v34.0 完了後に:
1. v30.1〜v34.9 を通じて積み上がった残課題を洗い出す
2. ドキュメントサイト v4 のコンテンツ要件を確定する
3. ベンチマーク比較対象（Python / Spark / dbt）を確定する
4. v34.5〜v34.9 の具体的な作業を決定する

**更新担当**: v34.0 リリース時

---

## 設計決定事項（暫定）

| 項目 | 暫定決定 | 確定時期 |
|---|---|---|
| 実案件デモの規模 | 500 行以上・複数 Rune・複数ファイル構成 | v34.0 後 |
| ベンチマーク比較対象 | Python pandas / Apache Spark / dbt | v34.0 後 |
| ドキュメントサイト v4 | 既存 Next.js 16 を継続・コンテンツ大幅増強 | v34.0 後 |
| セキュリティ審査 v2 の範囲 | エフェクトシステム形式検証 + OSS ライセンス | v34.0 後 |
| 破壊的変更 | なし | 固定 |

---

## バージョン計画（骨格）

### v34.1 — 実案件デモ実装

**テーマ**: 複数 Rune・複数ファイルを使った実規模パイプラインを `examples/` に追加する。

**対象デモ（暫定）**:

```
examples/real-world-etl/
├── fav.toml
├── src/
│   ├── types.fav           注文データの型定義
│   ├── validators.fav      ビジネスルールのバリデーション
│   ├── stages.fav          ETL ステージ群
│   ├── notifications.fav   Slack / Email 通知
│   └── main.fav            エントリポイント
├── data/
│   └── orders_sample.csv   サンプルデータ 10,000 行
└── README.md               30 分で動かす手順
```

処理フロー:
```
S3 から CSV ダウンロード
    |> バリデーション（欠損値・範囲チェック・重複除去）
    |> Postgres に書き込み
    |> BigQuery に同期
    |> 処理結果を Slack に通知
    |> OTel でトレース記録
```

完了条件:
- `examples/real-world-etl/` が完全な状態で存在する
- README に「30 分で動かす」手順が書かれている
- `fav check` / `fav test` が通る
- Rust テスト 1 件（examples の存在確認）

---

### v34.2 — ドキュメントサイト v4

**テーマ**: 新しいエンジニアが 1 日で Favnir を使い始められるドキュメントを整備する。

**構成（暫定）**:

```
favnir.dev/
├── /                      ランディング（30 秒で何ができるかわかる）
├── /learn/                チュートリアル
│   ├── getting-started    10 分チュートリアル
│   ├── first-pipeline     最初の ETL パイプライン
│   └── rune-guide         Rune の使い方
├── /cookbook/             実用レシピ 50 本以上
│   ├── postgres-etl
│   ├── s3-to-parquet
│   ├── kafka-consumer
│   ├── rag-pipeline
│   └── ...（50 本）
├── /errors/               エラーコードリファレンス（E0001〜）
│   ├── E0001              undefined variable
│   └── ...
├── /runes/                全 Rune ドキュメント（自動生成）
├── /playground/           ブラウザ内実行（WASM）
├── /bench/                ベンチマーク比較グラフ
└── /spec/                 形式的仕様書
```

新規追加コンテンツ:
- `/errors/` — `fav explain` コマンドと同内容を Web で閲覧できる
- cookbook を 30 本 → 50 本に増強
- ベンチマーク比較グラフ（Python / Spark との実測比較）

---

### v34.3 — ベンチマーク公開

**テーマ**: 実測ベンチマークを `bench/` ページで公開する。

**比較対象**（暫定）:
- Python pandas（CSV 読み込み・変換・Postgres 書き込み）
- Apache Spark（同上、大規模データ）
- dbt（SQL 変換パイプライン）

**計測項目**:
- 処理速度（行数/秒）
- メモリ使用量（ピーク）
- Lambda コールドスタート時間
- コンパイル時間

```
benchmarks/real-world/
├── python_pandas.json
├── apache_spark.json
└── favnir.json
```

---

### v34.4 — セキュリティ審査 v2

**テーマ**: v24.6.0（セキュリティ審査 v1）を更新し、Production Ready を確認する。

**審査対象**:
1. エフェクトシステムの形式的検証（`pure_fn_calls_effectful` W021 lint が機能しているか）
2. OSS 依存ライセンス確認（Cargo.toml の全依存が MIT / Apache-2.0 互換か）
3. Rune の認証情報の扱い（環境変数経由のみか、コードに埋め込めないか）
4. `fav run` の実行サンドボックス確認

---

### v34.5〜v34.9 — 安定化・最終調整

v34.0 完了後のドッグフード・審査結果で以下から選択・実施:

- v30.1〜v34.4 で積み上がった残課題の解消
- パフォーマンスチューニング（実測値に基づく）
- テストカバレッジの向上
- CHANGELOG / MIGRATION ガイドの整備
- `fav upgrade`（古いプロジェクトの移行支援）

---

## v35.0 — Production Ready マイルストーン宣言

**暫定完了条件（v34.0 完了後に確定）:**

| コンポーネント | 暫定完了基準 |
|---|---|
| 実案件デモ | `examples/real-world-etl/` が end-to-end で動作する |
| ドキュメント | `/errors/` ページ・cookbook 50 本・ベンチマーク比較が公開済み |
| ベンチマーク | Python pandas との比較で速度優位が示されている |
| セキュリティ | v2 審査で問題なし |
| 安定性 | テスト数 3000+、既知バグゼロ |
| 後方互換性 | v30.0.0 時点の .fav コードが v35.0.0 でも動作する |

**最終宣言文（暫定）:**

> 「`fav new --template postgres-etl my-pipeline` で始め、
>  `fav check` で型安全性を確認し、
>  `fav build --target native` でネイティブバイナリを生成し、
>  Lambda にデプロイして実データを処理できる。
>  エラーが起きれば `fav explain` で原因がわかり、
>  `fav test --watch` でリグレッションを防げる。
>
>  これが Favnir v35.0 — Production Ready の姿である。」

**★ クリーンアップ実施（v35.0 リリース時 — 最終クリーンアップ）:**

```bash
cd /c/Users/yoshi/favnir/fav
cargo clean
cargo build
cargo test 2>&1 | grep "test result"
cargo clippy --locked -- -D warnings
./target/debug/fav lint --deny-warnings --allow W017 --allow W018 --allow W019 self/compiler.fav
./target/debug/fav lint --deny-warnings --allow W012 --allow W017 --allow W018 --allow W019 self/checker.fav
du -sh target/
echo "=== v35.0.0 Production Ready クリーンアップ完了 ==="
```

---

## 参考リンク

- マスタースケジュール: `versions/roadmap/roadmap-v30.1-v35.0.md`
- 前フェーズ: `versions/roadmap/roadmap-v33.1-v34.0.md`
- 達成宣言: `MILESTONE.md`
