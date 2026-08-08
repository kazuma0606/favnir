# v58.6.0 Tasks — マルチ環境設定（dev / staging / prod）

Date: 2026-07-28
Status: COMPLETE（2026-07-28）— 3294 tests passed, 0 failed

---

## T0: 事前確認

- [x] `cargo test` でベースラインが 3292 tests passed, 0 failed であることを確認
- [x] `fav/Cargo.toml` のバージョンが `"58.5.0"` であることを確認
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.6.0 セクションを確認
- [x] `fav/src/driver.rs` に `inject_env_config` がまだ存在しないことを確認
- [x] `fav/src/main.rs` の `Some("run")` アームの先頭付近を確認（挿入点を把握）
- [x] `grep -c '58\.5\.0' fav/src/driver.rs` でローリング関連の文字列件数を確認（12 件）

---

## T1: Cargo.toml バージョン更新

- [x] `fav/Cargo.toml`: `version = "58.5.0"` → `"58.6.0"`

---

## T2: roadmap 更新（スコープ変更ブロック・テスト数補正）

- [x] `roadmap-v58.1-v59.0.md` の v58.6.0 セクションにスコープ変更ブロック追加（spec 作成時点で実施済み）
- [x] v58.6.0 完了条件を 3293 → 3294 に修正（実施済み）
- [x] v58.7.0 ベース・目標数を 3293→3294 / 3295→3296 に修正（実施済み）

---

## T3: driver.rs に関数追加

- [x] `inject_env_config(env_name: &str, pipeline_file: &str) -> i32` を追加
  - dev / staging / prod（その他）の 3 ケース出力
  - `Running <pipeline_file> (env: <env_name>) ...` を出力して 0 を返す

---

## T4: driver.rs テストモジュール追加

- [x] `v58600_tests` モジュールを v58500_tests の直前に挿入
  - [x] `env_config_parsed`: `inject_env_config("staging", "pipeline.fav")` → 0
  - [x] `env_config_injected`: `inject_env_config("prod", "pipeline.fav")` → 0

---

## T5: driver.rs ローリングチェック更新

- [x] assertion 文字列 `version = \"58.5.0\"` → `\"58.6.0\"` に一括更新（5 件）
- [x] failure メッセージ `"Cargo.toml version should be 58.5.0"` → `"58.6.0"` に更新（5 件）

---

## T6: main.rs 拡張

- [x] use imports に `inject_env_config` を追加
- [x] `Some("run")` アームの冒頭（`--debug` チェックより前）に `--env` フラグ検出ロジックを追加
  - `--env` フラグあり・値あり → `inject_env_config` を呼び `return;` で即座に終了
  - `--env` フラグあり・値なし → `eprintln!` + exit(1)
  - pipeline_file は positional 引数検索（`args.iter().skip(2).find(|a| !a.starts_with("--"))`)

---

## T7: テスト実行・確認

- [x] `cargo test -j 8 -- --test-threads=8` を実行
- [x] `env_config_parsed` pass を確認
- [x] `env_config_injected` pass を確認
- [x] 総テスト数 **3294** tests passed, 0 failed を確認

---

## T8: 事後処理

- [x] `CHANGELOG.md` に v58.6.0 エントリを追加
- [x] `versions/current.md` を v58.6.0 / 3294 tests に更新
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.6.0 実績欄を更新
- [x] `versions/roadmap/roadmap-v58.1-v59.0.md` の v58.7.0 ベース数を実績値に修正（code-review で tests 増加した場合）
- [x] このファイル（tasks.md）を COMPLETE ステータスに更新

---

## コードレビュー記録

- [BUG][HIGH] `pipeline_file` の positional search が `--env` の値（env name）を拾う → enumerate + filter で env_idx / env_idx+1 を除外（main.rs）
- [STYLE] `return;` 行に stub コメントがない → `// v58.x stub: ...` コメント追加（main.rs）
- [STYLE] `dev` 環境のテストケースなし → `env_config_dev` テスト追加（driver.rs）
- [STYLE] unknown env name の prod fallback テストなし → `env_config_unknown_falls_back_to_prod` テスト追加（driver.rs）

最終テスト数: 3296 tests passed, 0 failed（code-review 対応で +2）

---

Status: COMPLETE（2026-07-28）— 3296 tests passed, 0 failed
