# v21.6.0 — Playground v2 タスク

## ステータス: COMPLETE

---

## タスク一覧

### T1: `site/lib/share-url.ts` — URL エンコード/デコード ユーティリティ

- [x] **事前確認**: `ls site/lib/` でディレクトリが存在するか確認
- [x] `site/lib/share-url.ts` を新規作成
- [x] `base64urlEncode(bytes: Uint8Array): string` を実装（チャンク 8192 バイトで `btoa` + `+→-`, `/→_`, `=` 除去 — スプレッド展開禁止）
- [x] `base64urlDecode(str: string): Uint8Array` を実装
- [x] `encodeCode(code: string): Promise<string>` を実装
  - `TextEncoder` → `CompressionStream('gzip')` → `base64urlEncode`
  - フォールバック: 非圧縮 base64url（`CompressionStream` 未対応ブラウザ用）
- [x] `decodeCode(encoded: string): Promise<string>` を実装
  - `base64urlDecode` → `DecompressionStream('gzip')` → `TextDecoder`
  - フォールバック: 直接 `TextDecoder` デコード
- [x] `buildShareUrl(encoded: string): string` を実装（`?c=` パラメータ構築）
- [x] `encodeCode`/`decodeCode` の先頭に `if (typeof window === 'undefined') throw new Error('client only')` を追加
- [x] TypeScript 型エラー 0（`tsc --noEmit` で確認）

---

### T2: `site/lib/playground-templates.ts` — 6 テンプレート定義

- [x] `site/lib/playground-templates.ts` を新規作成
- [x] `PlaygroundTemplate { id, name, description, code }` インターフェースを定義
- [x] 6 テンプレートを `PLAYGROUND_TEMPLATES` 配列に実装:
  - `hello-world` — `IO.println("Hello, Favnir!")`
  - `pipeline-basic` — `stage Double |> AddOne |> seq Transform`
  - `list-transform` — `List.filter` + `List.map` + `List.fold`
  - `result-handling` — `Result<T, E>` + `?` 演算子
  - `record-type` — `type User = { name: String, age: Int }` + フィールドアクセス（レコード構築は **改行区切り**、コンマ区切り不可）
  - `fstring-format` — f-string 補間
- [x] 各テンプレートの `code` が Favnir 構文として正しいことを目視確認
- [x] TypeScript 型エラー 0

---

### T3: `site/app/playground/page.tsx` — Share ボタン + テンプレート UI + 実行統計

T1, T2 完了後に着手。

- [x] **事前確認**: 既存 `page.tsx` を読んで変更前の状態を把握
- [x] `share-url.ts` と `playground-templates.ts` を import
- [x] useState 追加:
  - `shareStatus: 'idle' | 'copied'`
  - `execStats: { ms: number; memMB: number | null } | null`
  - `showTemplates: boolean`
  - `fromShare: boolean`
- [x] URL 復元 useEffect:
  - `?c=` パラメータ → `decodeCode` → `setCode`
  - バナー表示用 `setFromShare(true)`
  - パラメータなし → 既存の `EXAMPLE_CODE` 設定
- [x] Share ボタン追加（ヘッダー右）:
  - `encodeCode(code)` → `buildShareUrl(encoded)` → `navigator.clipboard.writeText`
  - ボタンラベル: `idle` → `'📋 共有'`、`copied` → `'✓ コピー済み'`（2秒後に戻す）
- [x] テンプレートドロップダウン実装:
  - 「テンプレート ▼」ボタンで `showTemplates` をトグル
  - `PLAYGROUND_TEMPLATES` をリスト表示
  - 選択時に `setCode(template.code)`, `setShowTemplates(false)`, `setDiagnostics([])`, `setExecStats(null)`
  - ドロップダウン外クリックで閉じる（`useRef` + `document.addEventListener`)
- [x] 実行統計を `handleRun` に追加:
  - `t0 = performance.now()` を `window.__favnirCompile(code)` 呼び出し前に置く（コンパイル + instantiate + 実行を含む体感時間を計測）
  - `(performance as PerformanceWithMemory).memory?.usedJSHeapSize` でメモリ取得
  - `setExecStats({ ms, memMB })`
- [x] 出力パネル下部に実行統計表示: `実行時間: Xms | メモリ: X.XMB`
- [x] フロムシェアバナー表示（エディタ上部）
- [x] ビルドエラーなし（`next build` または `tsc`）

---

### T4: `infra/share/` — Lambda + S3 Terraform

- [x] `infra/share/` ディレクトリを新規作成
- [x] `infra/share/handlers/` ディレクトリを新規作成
- [x] `infra/share/handlers/share.js` を新規作成（plan.md 参照）:
  - `POST /share { code }` → slug 生成 → S3 PutObject → `{ slug, url }` を返す
  - `GET /share/{slug}` → S3 GetObject → `{ code }` を返す
  - CORS preflight (OPTIONS) ハンドリング
  - slug バリデーション: `/^[a-z0-9]{6}$/`
  - コードサイズ上限: 32KB
- [x] `infra/share/main.tf` を新規作成:
  - `aws_s3_bucket.shares` — バケット名: `favnir-playground-shares-${var.environment}`
  - `aws_s3_bucket_public_access_block.shares` — 全 block = true（必須）
  - `aws_s3_bucket_server_side_encryption_configuration.shares` — AES256（必須）
  - `aws_s3_bucket_lifecycle_configuration.shares_ttl` — 90日 TTL
- [x] `infra/share/handlers/package.json` を新規作成 — `{ "dependencies": { "@aws-sdk/client-s3": "^3" } }`
- [x] `npm install` を `infra/share/handlers/` で実行して `node_modules` を生成（zip バンドル用）
- [x] `infra/share/lambda.tf` を新規作成:
  - `aws_lambda_function.share` — Runtime: `nodejs20.x`, Handler: `share.handler`, `filename = "${path.module}/handlers/share.zip"`
  - IAM ロール（S3 `GetObject` / `PutObject` + `ListBucket` 権限）
  - API Gateway HTTP API + ルーティング (`POST /share`, `GET /share/{slug}`)
  - CORS 設定（`*` オリジン許可）
- [x] `infra/share/providers.tf` — AWS provider 設定（`LocalStack` 対応 `endpoint` 変数）
- [x] `infra/share/variables.tf` — `region`, `environment` 変数
- [x] `infra/share/outputs.tf` — `api_url` 出力
- [x] `terraform validate` でエラーなし

---

### T5: `site/app/playground/share-api.ts` — Lambda クライアント

T4 完了後に着手（API URL が確定してから）。

- [x] `site/app/playground/share-api.ts` を新規作成
- [x] `SHARE_API = process.env.NEXT_PUBLIC_SHARE_API_URL ?? ''` を設定
- [x] `shareCode(code: string): Promise<ShareResult | null>` を実装
  - `POST /share` を呼び出し
  - API URL 未設定の場合は `null` を返す（graceful degradation）
  - fetch エラー時も `null` を返す
- [x] `loadSharedCode(slug: string): Promise<string | null>` を実装
  - `GET /share/{slug}` を呼び出し
  - 404 や エラー時は `null` を返す
- [x] 設定手順を確認: `terraform output -raw api_url` の値を `NEXT_PUBLIC_SHARE_API_URL` として `site/.env.local` に設定
- [x] TypeScript 型エラー 0

---

### T6: page.tsx に Lambda 短縮 URL 統合

T3, T5 完了後に着手。

- [x] `share-api.ts` から `shareCode`, `loadSharedCode` を import
- [x] `handleShare` を更新:
  - Lambda API が利用可能なら短縮 URL（`/s/<slug>`）を優先
  - フォールバック: サーバーレス URL（`?c=`）
- [x] URL 復元 useEffect に `?s=<slug>` 対応を追加:
  - `params.get('s')` → `loadSharedCode(slug)` → `setCode`
  - Lambda 呼び出し失敗時は `EXAMPLE_CODE` を表示
- [x] ビルドエラーなし

---

### T7: Cargo.toml バージョン更新

- [x] `version = "21.5.0"` → `"21.6.0"` に変更
- [x] `v215000_tests::version_is_21_5_0` に `#[ignore]` を追加（忘れると `cargo test` が失敗）

---

### T8: `fav/src/driver.rs` — v216000_tests 追加

- [x] **事前確認**: `grep -n "mod v215000_tests" fav/src/driver.rs | head -3` で追加位置を確認
- [x] `v215000_tests` の後に `v216000_tests` モジュールを追加
- [x] 8 件のテストを実装（実装方針: `Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(...)` でファイル存在確認）:
  - `version_is_21_6_0` — `Cargo.toml` に `"21.6.0"` が含まれる
  - `playground_share_url_file_exists` — `site/lib/share-url.ts` が存在する
  - `playground_templates_file_exists` — `site/lib/playground-templates.ts` が存在する
  - `playground_share_api_file_exists` — `site/app/playground/share-api.ts` が存在する
  - `infra_share_main_tf_exists` — `infra/share/main.tf` が存在する
  - `infra_share_lambda_handler_exists` — `infra/share/handlers/share.js` が存在する
  - `playground_templates_count` — `playground-templates.ts` の中身に `"id":` または `id:` が 6 個以上含まれる
  - `changelog_has_v21_6_0` — `CHANGELOG.md` に `[v21.6.0]` が含まれる
- [x] `cargo test v216000` — 8/8 PASS を確認
- [x] `cargo test` — リグレッションなし（1817 件以上合格）

---

### T9: CHANGELOG + `site/content/docs/tools/playground.mdx`

- [x] `CHANGELOG.md` の先頭に v21.6.0 エントリを追加
- [x] `site/content/docs/tools/playground.mdx` を新規作成:
  - Playground v2 機能一覧（共有・テンプレート・実行統計）
  - 共有 URL の使い方
  - テンプレートギャラリーの紹介
  - フォーク（共有 URL からの読み込み）

---

## テスト一覧（v216000_tests、8件）

| テスト名 | 内容 |
|----------|------|
| `version_is_21_6_0` | Cargo.toml に `"21.6.0"` が含まれる |
| `playground_share_url_file_exists` | `site/lib/share-url.ts` が存在する |
| `playground_templates_file_exists` | `site/lib/playground-templates.ts` が存在する |
| `playground_share_api_file_exists` | `site/app/playground/share-api.ts` が存在する |
| `infra_share_main_tf_exists` | `infra/share/main.tf` が存在する |
| `infra_share_lambda_handler_exists` | `infra/share/handlers/share.js` が存在する |
| `playground_templates_count` | テンプレートが 6 個以上定義されている |
| `changelog_has_v21_6_0` | `CHANGELOG.md` に v21.6.0 エントリがある |

---

## テストカバレッジに関する注記

`encodeCode`/`decodeCode` のラウンドトリップ、slug バリデーション、32KB 上限チェックはブラウザ API（`CompressionStream`）に依存するため Rust テストでは検証不能。以下の手動確認で代替する:

1. `?c=` パラメータ経由で 1000 文字コードを共有→復元してエディタ内容が一致することを確認
2. 32KB 超のコードを投稿して 400 エラーが返ることを確認（Lambda 直接 curl）
3. 存在しない slug を `GET /share/xxxxxx` で叩いて 404 が返ることを確認

---

## 完了条件チェックリスト

- [x] `cargo test v216000` — 8/8 PASS
- [x] `cargo test` — リグレッションなし（1817 件以上合格）
- [x] `?c=` パラメータでコードのラウンドトリップが正確（手動確認）
- [x] テンプレート 6 種が Playground UI で選択可能（手動確認）
- [x] 実行時間が出力パネルに表示される（手動確認）
- [x] `terraform validate` — infra/share/ でエラーなし
- [x] `CHANGELOG.md` に v21.6.0 エントリ
- [x] `fav/Cargo.toml` version が `21.6.0`
- [x] `site/content/docs/tools/playground.mdx` 作成済み

---

## 優先度

```
T1（share-url.ts）     ← 最初。T3, T6 の基盤
T2（templates.ts）     ← T1 と並列可
T4（infra/share/）     ← T1/T2 と並列可（インフラ作業）
T3（page.tsx 更新）    ← T1, T2 後
T5（share-api.ts）     ← T4 後（API URL 確定後）
T6（Lambda 統合）      ← T3, T5 後
T7（Cargo.toml）       ← いつでも
T8（driver.rs tests）  ← T1〜T6 完了後（ファイルが存在してからテスト）
T9（CHANGELOG + MDX）  ← 最後
```

---

## 実装リスクと対策

| リスク | 対策 |
|--------|------|
| `CompressionStream` が SSR で使えない | `'use client'` 限定で呼び出す。`typeof window === 'undefined'` ガードを追加 |
| base64url のデコードで URL 特殊文字が壊れる | base64url は URL-safe（`+→-`, `/→_`）なので `encodeURIComponent` は不要 |
| Lambda cold start で共有 URL 生成が遅い | サーバーレス URL（`?c=`）をデフォルトとし、Lambda はオプション強化 |
| S3 slug 衝突 | `GetObject` で存在確認し、衝突時は再生成（確率 ≈ 2×10^-9 / 件） |
| `performance.memory` が undefined（Chrome 以外） | `?.` オプショナルチェーンで安全に参照 |
| `NEXT_PUBLIC_SHARE_API_URL` が未設定 | `null` を返す分岐で graceful degradation |
| LocalStack で S3 Lifecycle ルールが動かない | Lifecycle は本番確認のみ。ローカルでは TTL テストをスキップ |
