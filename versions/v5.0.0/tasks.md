# Favnir v5.0.0 タスクリスト — AWS 本番稼働 + リファレンスサイト

作成日: 2026-05-17
完了日: -

実装順序: Phase C（リファレンスサイト）→ Phase B（WASM）→ Phase A（CI/CD）→ Phase D（Dogfooding）

---

## Phase C-0: `fav explain` コマンド

- [ ] `ErrorEntry` 構造体を `fav/src/driver.rs` に追加（`serde::Serialize` derive）
- [ ] `build_error_catalog() -> Vec<ErrorEntry>` を実装（主要エラー 20 件以上）
- [ ] `cmd_explain(code_arg: Option<&str>, all: bool, format: &str)` を実装
  - [ ] `--all --format json` → `serde_json::to_string_pretty` で出力
  - [ ] `--all`（text）→ `code: title` 一覧表示
  - [ ] `<code>` 指定 → 単一エラー詳細表示
  - [ ] 未知コードはエラー終了
- [ ] `fav/src/main.rs` に `Some("explain")` アームを追加（`--all` / `--format` / 位置引数パース）
- [ ] `driver::explain_tests` モジュールを追加 — テスト 5 件
  - [ ] `explain_single_known_code` — E0001 の詳細が返る
  - [ ] `explain_unknown_code_returns_none` — 未知コードは None
  - [ ] `explain_all_returns_nonempty` — catalog が 20 件以上
  - [ ] `explain_all_json_is_valid` — JSON パースが成功する
  - [ ] `explain_all_has_required_fields` — code / title / description が全件存在する
- [ ] `cargo build` が通る
- [ ] `fav explain --all --format json > site/content/errors/catalog.json` が動作する

---

## Phase C-1: `site/` プロジェクト初期化

- [ ] `site/package.json` を作成（mock/package.json をベースに Next.js 16 + MDX 依存追加）
- [ ] `site/next.config.ts` を作成（`output: 'export'`, trailingSlash, images unoptimized）
- [ ] `site/tsconfig.json` を作成
- [ ] `site/tailwind.config.ts` を作成（Tailwind v4）
- [ ] `site/postcss.config.mjs` を作成
- [ ] `site/components.json` を作成（shadcn/ui 設定）
- [ ] `site/app/globals.css` を作成（mock/app/globals.css または mock/styles をベース）
- [ ] `site/app/layout.tsx` を作成（共通レイアウト: フォント・メタデータ・テーマ）
- [ ] `site/public/images/favnir-mascot.png` を配置（mock/ からコピー）
- [ ] shadcn/ui コンポーネントを `site/components/ui/` に配置（mock/components/ui/ からコピー）
- [ ] `npm install` が成功する
- [ ] `npm run dev` が起動する

---

## Phase C-2: コンポーネント移植・更新

- [ ] `site/components/layout/header.tsx` を作成（mock/ の Header を移植、nav 更新）
  - [ ] ナビゲーション: ドキュメント / エラー / Rune / Playground / GitHub
- [ ] `site/components/layout/footer.tsx` を作成（mock/ の Footer を移植）
- [ ] `site/components/layout/sidebar.tsx` を作成（docs サイドバー: カテゴリ + ページリスト）
- [ ] `site/components/landing/hero.tsx` を作成（mock/ の Hero を移植、以下を更新）
  - [ ] コードプレビューを現行 Favnir 構文に書き換え（`pipeline/source/sink` → `import rune / bind / AWS`）
  - [ ] Badge を `v5.0.0` に更新
  - [ ] キャッチコピーを「型安全なデータパイプライン専用言語」に統一
- [ ] `site/components/landing/features.tsx` を作成（mock/ の Features を移植、内容更新）
  - [ ] エフェクト型システム（`!Io / !Aws / !Auth`）
  - [ ] Rune エコシステム（8 Rune）
  - [ ] AWS ネイティブ（LocalStack → 本番切り替え）
  - [ ] 型安全（コンパイル時エラー）
- [ ] `site/components/landing/docs-preview.tsx` を作成（カテゴリリンク更新）
- [ ] `site/components/landing/cta.tsx` を作成
- [ ] `site/app/page.tsx` を作成（全ランディングコンポーネントを組み合わせ）
- [ ] `npm run build` が成功する

---

## Phase C-3: MDX ドキュメントコンテンツ

- [ ] `site/lib/docs.ts` を作成（`getAllDocs()` / `getDocBySlug()` / `buildSidebar()`）
- [ ] `site/components/docs/mdx-components.tsx` を作成（MDX レンダラー）
- [ ] `site/components/docs/code-block.tsx` を作成（shiki でシンタックスハイライト）
  - [ ] Favnir 構文用カスタムグラマー（`favnir` lang として登録）
  - [ ] トークン定義: キーワード（import/fn/type/match）・エフェクト（!Io/!AWS 等）・文字列・数値・コメント・名前空間
  - [ ] MDXRemote に `components={{ pre: CodeBlock }}` を渡す
  - [ ] ビルド後に再デプロイ（`scripts/deploy-site.sh`）
- [ ] `site/components/docs/toc.tsx` を作成（ページ内目次）
- [ ] `site/app/docs/layout.tsx` を作成（サイドバー付きドキュメントレイアウト）
- [ ] `site/app/docs/[...slug]/page.tsx` を作成（`generateStaticParams` + MDX 表示）
- [ ] MDX ファイルを作成（最低 15 ページ）
  - [ ] `content/docs/introduction.mdx`
  - [ ] `content/docs/installation.mdx`
  - [ ] `content/docs/quickstart.mdx`
  - [ ] `content/docs/language/types.mdx`
  - [ ] `content/docs/language/effects.mdx`
  - [ ] `content/docs/language/pattern-matching.mdx`
  - [ ] `content/docs/language/runes.mdx`
  - [ ] `content/docs/stdlib/io.mdx`
  - [ ] `content/docs/stdlib/list.mdx`
  - [ ] `content/docs/stdlib/map.mdx`
  - [ ] `content/docs/stdlib/string.mdx`
  - [ ] `content/docs/stdlib/result.mdx`
  - [ ] `content/docs/stdlib/option.mdx`
  - [ ] `content/docs/runes/db.mdx`
  - [ ] `content/docs/runes/aws.mdx`
- [ ] ドキュメントページが `npm run build` で静的生成される

---

## Phase C-4: エラーカタログページ

- [ ] `fav explain --all --format json > site/content/errors/catalog.json` を実行してファイルを生成
- [ ] `site/lib/errors.ts` を作成（`getAllErrors()` / `getErrorByCode()`）
- [ ] `site/app/errors/page.tsx` を作成（エラー一覧: コード + タイトル）
- [ ] `site/app/errors/[code]/page.tsx` を作成（エラー詳細: 説明 + 例 + Fix）
  - [ ] `generateStaticParams` で全エラーコードのページを生成
- [ ] `npm run build` でエラーカタログページが生成される

---

## Phase C-5: Rune カタログページ

- [ ] `site/content/docs/runes/` に全 8 Rune の MDX を作成
  - [ ] `http.mdx` / `duckdb.mdx` / `auth.mdx` / `log.mdx` / `env.mdx` / `gen.mdx`（C-3 の db.mdx / aws.mdx に加え）
- [ ] `site/app/runes/page.tsx` を作成（全 Rune カード一覧）
- [ ] `site/app/runes/[name]/page.tsx` を作成（Rune API リファレンス）

---

## Phase C-6: Terraform + デプロイ

- [ ] `infra/site/providers.tf` を作成（aws + us-east-1 alias）
- [ ] `infra/site/variables.tf` を作成（`aws_region` / `site_bucket_name`）
- [ ] `infra/site/main.tf` を作成
  - [ ] `aws_s3_bucket` + `aws_s3_bucket_public_access_block`
  - [ ] `aws_cloudfront_origin_access_control`
  - [ ] `aws_cloudfront_distribution`（default + WASM ordered キャッシュ + カスタムエラーページ）
  - [ ] `aws_s3_bucket_policy`（OAC のみ許可）
- [ ] `infra/site/outputs.tf` を作成（`cloudfront_domain` / `site_bucket` / `distribution_id`）
- [ ] `terraform init` が成功する
- [ ] `terraform plan` でリソースが正しく表示される
- [ ] `terraform apply` で S3 + CloudFront が作成される
- [ ] `npm run build` → `aws s3 sync out/ s3://<bucket> --delete` でファイルが配信される
- [ ] CloudFront ドメインでサイトが HTTPS アクセスできる

---

## Phase B: `@favnir/wasm`

- [ ] `crates/favnir-wasm/Cargo.toml` を作成（crate-type cdylib + wasm-bindgen）
- [ ] `fav/src/lib.rs` を作成（WASM 向け公開 API）
  - [ ] `check_source(source: &str) -> CheckResult`
  - [ ] `run_source_sandboxed(source: &str) -> RunResult`（`!Io` のみ許可、stdout キャプチャ）
- [ ] `crates/favnir-wasm/src/lib.rs` を作成
  - [ ] `#[wasm_bindgen] pub fn fav_check(source: &str) -> JsValue`
  - [ ] `#[wasm_bindgen] pub fn fav_run(source: &str) -> JsValue`
- [ ] `scripts/build-wasm.sh` を作成
- [ ] `wasm-pack build` が成功し `site/public/wasm/` に出力される
- [ ] `site/app/playground/page.tsx` を作成
  - [ ] `@favnir/wasm` を動的 import（`useEffect(() => init(), [])`）
  - [ ] コードエディタ（textarea）+ Run / Check ボタン
  - [ ] 出力エリア（stdout / エラー一覧）
  - [ ] デフォルトサンプルコード（Point / distance の例）
- [ ] Playground が `npm run dev` でブラウザ内実行できる
- [ ] `npm run build` に Playground が含まれる

---

## Phase A: CI/CD

- [ ] `.github/workflows/ci.yml` を作成
  - [ ] `cargo build` ステップ
  - [ ] `cargo test` ステップ
  - [ ] `cd site && npm ci && npm run build` ステップ
- [ ] `.github/workflows/deploy.yml` を作成
  - [ ] `aws-actions/configure-aws-credentials`（OIDC）
  - [ ] ECR login → docker build → push
  - [ ] `fav deploy --env prod`
  - [ ] `scripts/build-wasm.sh`
  - [ ] `cd site && npm run build`
  - [ ] `aws s3 sync` + CloudFront キャッシュ無効化
- [ ] CI が PR ごとに実行される
- [ ] deploy が main merge 時に実行される

---

## Phase D: Dogfooding

- [ ] `rune-registry/fav.toml` を作成
- [ ] `rune-registry/src/main.fav` を作成（HTTP サーバー起動）
- [ ] `rune-registry/src/handlers.fav` を作成（list / info / publish エンドポイント）
- [ ] `rune-registry/src/storage.fav` を作成（S3 + DynamoDB 操作）
- [ ] `fav deploy --env prod` で ECS にデプロイされる
- [ ] `fav install csv` が AWS 上の Registry から動作する

---

## 完了条件

- [ ] `cargo build` が通る
- [ ] 既存テスト（937 件）が全て pass
- [ ] 新規テスト 5 件（`fav explain`）が pass
- [ ] `npm run build` が成功する
- [ ] CloudFront でサイトが HTTPS 配信される
- [ ] Playground でブラウザ内 Favnir 実行が動く
- [ ] CI が PR ごとに動く
- [ ] deploy が main merge 時に動く

---

## 実装メモ

- **Phase C が最優先**: サイトを先に公開し、その後 WASM Playground を追加する
- **mock/ はテンプレートとして参照**: `site/` に移植・更新。mock/ 自体は削除しない
- **`next/image` は unoptimized**: S3 静的配信では image optimization API が使えない
- **wasm-pack は事前インストール必要**: `cargo install wasm-pack`
- **WASM MIME タイプ**: S3 + CloudFront では `application/wasm` が自動設定される
- **Favnir カスタム lang**: shiki に `favnir` 言語定義を追加することでコードブロックのハイライトが改善する
- **`fav explain` のエラーカタログ**: checker.rs の定数を手動でリスト化する（リフレクション不可）。主要 20〜30 件を優先
- **Terraform state バケット**: `favnir-terraform-state` S3 バケットは手動で事前作成が必要
- **OIDC IAM ロール**: GitHub Actions → AWS の OIDC 連携は Terraform で管理
