# Favnir v10.0.0 Tasks

Date: 2026-06-03
Theme: OSS 公開準備完了（CI / `fav new` / CONTRIBUTING / GitHub Public 化）

---

## 積み残し確認

- [x] Z-1: W004 lint ルール（TooManyArgs）の実装確認
  - v9.4.0 COMPLETE ログに "+ W004 lint ルール" とあるが、`memory/MEMORY.md` の「未実装タスク」欄に残存している
  - `fav/self/compiler.fav` の lint セクションを確認し、`lint_fn_w004` が存在すれば完了済みとして記録
  - 未実装であれば `lint_fn_w004` を実装する（`stage` の入力型がタプル 4 個以上 → W004）
  - 完了後、`memory/MEMORY.md` の「未実装タスク」欄から削除
- [x] Z-2: compiler.fav par stack overflow（既知制限）の文書化
  - v9.13.0 F-1e では compiler.fav pipeline が par を含む seq をコンパイルするとスタックオーバーフロー
  - 修正は今バージョンのスコープ外。`versions/v10.0.0/known-limitations.md` に記録して積み残しを明示する

---

## Phase A: `fav new` スキャフォールディングコマンド

`cli.fav` に `fn cmd_new(name: String) -> Unit !Io` を追加する。
Rust 変更なし（`IO.write_file_raw` / `IO.make_dir_raw` で実装）。

- [x] A-1: `cli.fav` に `cmd_new` を実装
  ```favnir
  fn cmd_new(name: String) -> Unit !Io = {
    // ディレクトリ構造を生成
    // <name>/fav.toml
    // <name>/src/main.fav
    // <name>/.gitignore
  }
  ```
  - `fav.toml` テンプレート: `[project]\nname = "<name>"\nversion = "0.1.0"\nsrc = "src"`
  - `src/main.fav` テンプレート: 最小 stage + seq の例（Order パイプライン）
  - `.gitignore` テンプレート: `*.fvc\n.fav_cache/`
- [x] A-2: `cmd_new` を `cli.fav` の `main` ディスパッチに追加（`"new"` コマンド）
- [x] A-3: `v10_tests` — `fav_new_creates_project_structure` テスト
  - tempdir に `fav new myproject` を実行
  - `myproject/fav.toml`, `myproject/src/main.fav`, `myproject/.gitignore` が存在することを確認
- [x] A-4: `fav new myproject` で生成されたプロジェクトが `fav run src/main.fav` で正常実行できること
  - テスト: `fav_new_generated_project_runs`

---

## Phase B: GitHub Actions CI

`.github/workflows/ci.yml` を作成する。
Rust 変更なし。

- [x] B-1: `.github/workflows/ci.yml` を作成
  ```yaml
  name: CI
  on:
    push:
      branches: [master]
    pull_request:
  jobs:
    test:
      runs-on: ubuntu-latest
      steps:
        - uses: actions/checkout@v4
        - uses: dtolnay/rust-toolchain@stable
        - run: cargo build --release
        - run: cargo test
        - run: ./target/release/fav check fav/self/compiler.fav
        - run: ./target/release/fav check fav/self/checker.fav
        - run: ./target/release/fav check fav/self/cli.fav
        - run: ./target/release/fav lint fav/self/compiler.fav
        - run: ./target/release/fav lint fav/self/checker.fav
        - run: ./target/release/fav fmt --check fav/self/compiler.fav
        - run: ./target/release/fav fmt --check fav/self/checker.fav
  ```
- [x] B-2: CI が main ブランチでローカル実行できることを `act` またはワークフロー構文確認
  - YAML 構文エラーがないことを確認（`python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"` 等）

---

## Phase C: CONTRIBUTING.md + CHANGELOG.md

- [x] C-1: `CONTRIBUTING.md` を作成（リポジトリルート）
  内容:
  - 開発環境セットアップ（Rust stable、`cargo build`）
  - `cargo test` でのテスト実行
  - `fav check fav/self/` での self-check
  - PR ガイドライン（小さい単位・テスト必須・bootstrap 維持）
  - コードスタイル（`fav fmt` 適用済みであること）
  - bootstrap 検証の説明（`cargo test bootstrap`）
- [x] C-2: `CHANGELOG.md` を作成（リポジトリルート）
  - v4.0.0〜v9.13.0 の主要機能を簡潔にまとめる
  - 各バージョンに 1〜3 行の説明（詳細は `versions/` を参照）
  - フォーマット: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) 準拠
- [x] C-3: `LICENSE` ファイルを確認
  - リポジトリに MIT ライセンスが配置されているか確認
  - なければ作成（Copyright 2026 Yoshi, MIT License）

---

## Phase D: テスト + self-check + バージョン更新 + commit

- [x] D-1: `v10_tests` モジュールを `src/driver.rs` に追加
  - `fav_new_creates_project_structure`
  - `fav_new_generated_project_runs`
- [x] D-2: `cargo test v10` — 2 件通過
- [x] D-3: `cargo test checker_fav_wire_self_check` — 通過
- [x] D-4: `cargo test bootstrap` — 通過
- [x] D-5: `cargo test` — 全件通過
- [x] D-6: `fav/Cargo.toml` version → `"10.0.0"`
- [x] D-7: `fav/self/cli.fav` の `run_version` → `"10.0.0"`
- [x] D-8: 本ファイル完了チェック
- [x] D-9: `memory/MEMORY.md` に v10.0.0 完了を記録
- [x] D-10: commit

---

## 完了条件

| 条件 | 確認 |
|---|---|
| `fav new <name>` がプロジェクト構造を生成する | ✓ |
| 生成プロジェクトが `fav run src/main.fav` で実行できる | ✓ |
| `.github/workflows/ci.yml` が正しい YAML 構文である | ✓ |
| CI ワークフローに `cargo test` + `fav check/lint/fmt` が含まれる | ✓ |
| `CONTRIBUTING.md` がリポジトリルートに存在する | ✓ |
| `CHANGELOG.md` がリポジトリルートに存在する | ✓ |
| `LICENSE`（MIT）がリポジトリルートに存在する | ✓ |
| W004 の実装状況が確認・整理されている | ✓ |
| `cargo test` 全件通過 | ✓ |
| `cargo test checker_fav_wire_self_check` 通過 | ✓ |
| `cargo test bootstrap` 維持 | ✓ |

---

## 実装メモ

### IO.make_dir_raw
- `IO.make_dir_raw(path: String) -> Unit !Io` — ディレクトリ作成 primitive
- 既存か否かに関わらず成功（`create_dir_all` 相当）
- `vm.rs` に追加が必要な場合は確認する

### fav new のテンプレート設計
```
<name>/
  fav.toml          # [project] name/version/src
  src/
    main.fav        # 最小動作例（Order パイプライン）
  .gitignore        # *.fvc / .fav_cache/
```

`src/main.fav` テンプレート内容:
```favnir
type Order = { id: Int  item: String  amount: Float }

stage ParseOrder: String -> Order = |s| {
  Order { id: 1  item: s  amount: 0.0 }
}

stage FormatOrder: Order -> String = |o| {
  "Order#" + Int.to_string(o.id) + ": " + o.item
}

seq ProcessOrder = ParseOrder |> FormatOrder
```

### CI ワークフロー設計方針
- `ubuntu-latest` + Rust stable（最新安定版）
- リリースビルドで `fav` を生成してから self-check/lint/fmt を実行
- Windows/macOS のクロスプラットフォームビルドは将来版で対応（v10.0.0 はスコープ外）

### GitHub Public 化（手動作業）
本タスクファイルの完了後、以下を手動で実施:
1. GitHub リポジトリ設定 → Visibility を Public に変更
2. README.md が最新状態であることを確認
3. 発表準備（ブログ・SNS）

### スコープ外
- macOS / Windows CI
- Playground（WASM）更新
- サイトドキュメント更新（v10.0.0 では CLI/ドキュメントのみ、サイト大改修は v10.1.0 以降）
- compiler.fav par stack overflow の根本修正（既知制限として文書化）
