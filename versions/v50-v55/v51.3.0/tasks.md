# Tasks: v51.3.0 — ストリーミングバックプレッシャー制御

Status: COMPLETE
Date: 2026-07-19

---

## T0 — 事前確認

- [x] `cargo test` 3117 passed, 0 failed を確認（ベース確認）
- [x] `cargo clippy -- -D warnings` クリーンであることを確認
- [x] `toml.rs` の `StreamConfig` に `buffer_size` が**存在しない**ことを確認
- [x] `vm.rs` の VM struct に `stream_buffer_size` が**存在しない**ことを確認
- [x] `vm.rs` の `__streaming_pipeline` ハンドラが `buffer_size` を参照**していない**ことを確認（行 5541〜5543 付近）
- [x] `v51200_tests::cargo_toml_version_is_51_2_0` が存在することを確認（削除対象）
- [x] `parse_fav_toml_pub` が `crate::toml::parse_fav_toml_pub` として accessible であることを確認（`parse_fav_toml` は private）
- [x] `invoke_function` が private であることを確認（`run_with_stream_buffer_size` は `impl VM` 内に定義する）
- [x] 既存 `streaming_pipeline_executes` テスト（行 33940 付近）の stage シグネチャが `List<Int> -> List<Int>` であることを確認（`stream_backpressure_blocks` のテンプレートとして使用）

## T1 — `toml.rs` — `StreamConfig.buffer_size` 追加 + 解析

- [x] `StreamConfig` struct に `buffer_size: Option<usize>` フィールドを追加（コメント付き）
- [x] `parse_fav_toml` の `"stream"` セクション処理（行 842〜857 付近）に `"buffer_size"` キー解析を追加
  - [x] `val.trim_matches('"').parse::<usize>().ok().filter(|&n| n > 0)` でパース（`buffer_size = 0` は `None` 相当）
- [x] `cargo build` が通ることを確認（`StreamConfig` を使う箇所で `..Default::default()` が使われていれば問題なし）

## T2 — `backend/vm.rs` — VM struct + 初期化更新

- [x] VM struct に `stream_buffer_size: Option<usize>` フィールドを追加（`db_path` の後あたり）
- [x] `VM::new_with_db_path` の初期化ブロックに `stream_buffer_size: None` を追加
- [x] `cargo build` が通ることを確認

## T3 — `backend/vm.rs` — `run_with_stream_buffer_size` 静的メソッド追加

- [x] `impl VM` ブロック内（`VM::run` の直後）に `run_with_stream_buffer_size` を追加
  - [x] `VM::new_with_db_path(artifact, None)` で VM を構築
  - [x] `vm.stream_buffer_size = buffer_size` を設定
  - [x] `args.into_iter().map(VMValue::from).collect()` で引数を変換
  - [x] `vm.invoke_function(artifact, fn_idx, args_vm)?` で実行
  - [x] `Ok(Value::from(result))` を返す
- [x] `cargo build` が通ることを確認

## T4 — `backend/vm.rs` — `__streaming_pipeline` バックプレッシャー適用

- [x] `__streaming_pipeline` ハンドラ（行 5541〜5543）の chunk_size 計算を拡張
  - [x] `compiled_chunk_size` として既存の計算を変数に切り出す
  - [x] `self.stream_buffer_size` が `Some(buf)` の場合 `compiled_chunk_size.min(buf)` を使用
  - [x] コメントで「将来 tokio 化時に sync_channel に置換予定」と明記
- [x] `cargo test` で既存の streaming テスト（`streaming_pipeline_executes` 等）が引き続きパスすることを確認（`stream_buffer_size = None` のデフォルトで動作変化なし）

## T5 — `driver.rs` — v51300_tests 追加

- [x] `v51300_tests` モジュールを `v51200_tests` の直前に追加（3 件）:
  - [x] `cargo_toml_version_is_51_3_0`: version が `"51.3.0"` を含むことを assert
  - [x] `stream_buffer_size_config`:
    - [x] `crate::toml::parse_fav_toml_pub` を使用（`parse_fav_toml` は private）
    - [x] `"[stream]\nbuffer_size = 500\n"` をパース
    - [x] `config.stream.as_ref().unwrap().buffer_size == Some(500)` を assert
  - [x] `stream_backpressure_blocks`:
    - [x] `stage double_list: List<Int> -> List<Int> = |xs| { List.map(xs, |x| { x * 2 }) }` を使用（scalar-to-scalar は不可）
    - [x] `#[streaming]` アノテーション付き `seq Pipeline = double_list`
    - [x] `VM::run_with_stream_buffer_size(&artifact, pipeline_idx, vec![input], Some(2))` で実行
    - [x] 戻り値が `Value::List` で長さ 6 であることを assert
    - [x] 各要素が `[2, 4, 6, 8, 10, 12]` であることを assert（chunk_size=2 でも正しく処理される）
- [x] `v51200_tests::cargo_toml_version_is_51_2_0` を削除（他テストは保持）

## T6 — バージョン更新・完了

- [x] `fav/Cargo.toml` version → `"51.3.0"`
- [x] `cargo test` 3119 passed, 0 failed
- [x] `cargo clippy -- -D warnings` クリーン
- [x] `CHANGELOG.md` に v51.3.0 エントリ追加（`[v51.3.0]` 表記で追加）
- [x] `versions/current.md` を v51.3.0（3119 tests）に更新
- [x] `roadmap-v51.1-v52.0.md` の v51.3.0 実績欄を更新
- [x] tasks.md を COMPLETE に更新（T0〜T6 全 `[x]`）
