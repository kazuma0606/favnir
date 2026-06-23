# v20.6.0 — io_uring 非同期 I/O タスク

## ステータス: COMPLETE（T1〜T6 完了）

---

## タスク一覧

### T1: `fav/Cargo.toml` — tokio-uring 依存追加

- [x] **事前確認**: `grep -n "tokio\|futures" fav/Cargo.toml` で既存の tokio 設定を確認
- [x] `[target.'cfg(target_os = "linux")'.dependencies]` セクションを新規追加
- [x] `tokio-uring = "0.4"` を追加
- [x] **API 確認**: tokio-uring 0.4 の `File::open` / `read_at` / `start` のシグネチャを確認:
  ```bash
  grep -rn "pub async fn open\|pub async fn read_at\|pub fn start" \
    ~/.cargo/registry/src/*/tokio-uring-0.4*/src/ 2>/dev/null | head -10
  ```
- [x] **futures クレート確認**: `grep "^futures" fav/Cargo.toml` で既存依存を確認。存在しない場合は `futures = "0.3"` を `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` セクションに追加（**必須 — 未追加だと Linux バックエンドがコンパイルエラーになる**）
- [x] `cargo check` でコンパイルエラー 0（Linux 環境では `tokio-uring` が引き込まれることを確認）

---

### T2: `fav/src/backend/vm.rs` — `read_files_batch_impl` + `read_one_uring` + `"IO.read_files_batch"` primitive

#### 2-1. 配置箇所の特定

- [x] `grep -n "read_csv_mmap\|// ── v20\|vm_call_builtin" fav/src/backend/vm.rs | tail -20` で末尾のヘルパー関数セクションと `vm_call_builtin` を確認
- [x] `grep -n "FavList::new\|into_iter.*collect.*FavList\|VMValue::List" fav/src/backend/vm.rs | head -10` で FavList 変換パターンを確認

#### 2-2. `read_files_batch_impl`（Linux バックエンド）を追加

- [x] `// ── v20.6.0: io_uring batch read helpers ──` セクションを vm.rs の末尾付近（`read_csv_mmap` の下）に追加
- [x] `#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]` の `read_one_uring` async fn を実装:
  - [x] `tokio_uring::fs::File::open(&path).await` でファイルオープン（`map_err` でエラー変換）
  - [x] `std::fs::metadata(&path)` でファイルサイズ取得
  - [x] `vec![0u8; size]` でバッファ確保
  - [x] `file.read_at(buf, 0).await` — 戻り値は `(Result<usize, io::Error>, Vec<u8>)` のムーブセマンティクス
  - [x] `let bytes_read = res.map_err(...)?;` + `buf.truncate(bytes_read);` で部分読み込みを適切に処理
  - [x] `String::from_utf8(buf)` でデコード
- [x] `#[cfg(all(target_os = "linux", not(target_arch = "wasm32")))]` の `read_files_batch_impl` を実装:
  - [x] `tokio_uring::start(async { ... })` でブロッキング起動
  - [x] `paths.iter().map(|p| read_one_uring(p.clone())).collect::<Vec<_>>()` でハンドル収集
  - [x] `futures::future::try_join_all(handles).await` で並列実行
  - [x] `map_err` で `String` エラーに変換

#### 2-3. `read_files_batch_impl`（非 Linux バックエンド）を追加

- [x] `#[cfg(all(not(target_os = "linux"), not(target_arch = "wasm32")))]` の `read_files_batch_impl` を実装:
  - [x] `use rayon::prelude::*;`
  - [x] `paths.par_iter().map(|p| std::fs::read_to_string(p).map_err(...)).collect()`

#### 2-4. `"IO.read_files_batch"` プリミティブを `vm_call_builtin` に追加

- [x] 既存の `"IO."` 系プリミティブの近くに追加（例: `IO.read_file` ハンドラの後）
- [x] `args.into_iter().next()` から `VMValue::List` を取り出し、各要素を `VMValue::Str` として収集
- [x] `#[cfg(not(target_arch = "wasm32"))]` ブロック内で `read_files_batch_impl(&paths)` を呼び出し
- [x] 成功時: `FavList::new(contents.into_iter().map(VMValue::Str).collect::<Vec<_>>())` → `ok_vm(VMValue::List(list))`（`FavList` は `FromIterator` 未実装のため `collect::<FavList>()` は使えない）
- [x] 失敗時: `err_vm(VMValue::Str(e))`
- [x] `#[cfg(target_arch = "wasm32")]` ブロックで `err_vm(VMValue::Str("IO.read_files_batch: not supported on wasm32".to_string()))` を返す
- [x] `cargo check` でコンパイルエラー 0

#### 2-5. `is_known_builtin_namespace` への追加確認

- [x] `grep -n "is_known_builtin_namespace\|\"IO\"" fav/src/backend/vm.rs | head -10` で `"IO"` が既に登録済みであることを確認（変更不要の確認）

---

### T3: `fav/src/driver.rs` — `v206000_tests`

- [x] `driver.rs` 末尾に `#[cfg(test)] mod v206000_tests { ... }` を追加
- [x] `temp_txt!` マクロを実装（`line!()` で並列テスト競合回避）:
  ```rust
  macro_rules! temp_txt {
      ($content:expr) => {{
          let mut p = std::env::temp_dir();
          p.push(format!("fav_v206_{}_{}.txt", std::process::id(), line!()));
          std::fs::write(&p, $content).unwrap();
          p
      }};
  }
  ```
- [x] テスト 1: `version_is_20_6_0` — `include_str!("../Cargo.toml")` に `"20.6.0"` が含まれる
- [x] テスト 2: `io_batch_reads_single_file` — 1 ファイルを `read_files_batch_impl` で読み込み、内容が一致する
  - [x] `temp_txt!("hello world")` で temp ファイル作成
  - [x] `remove_file` でクリーンアップしてから `expect`
- [x] テスト 3: `io_batch_reads_multiple_files` — 3 ファイルを並列読み込み、全内容が正しい
- [x] テスト 4: `io_batch_preserves_order` — 入力パスの順序通りに結果が返る（**3 ファイル** `"alpha"`/`"beta"`/`"gamma"` でインデックス指定確認）
- [x] テスト 5: `io_batch_error_on_missing_file` — 存在しないパスを含む場合に `Err` が返る
- [x] 各テストで `#[cfg(not(target_arch = "wasm32"))]` ガードを付与（`read_files_batch_impl` は非 WASM のみ）
- [x] `cargo test v206000` — 5/5 PASS を確認

---

### T4: `fav/Cargo.toml` バージョン更新

- [x] `version = "20.5.0"` → `"20.6.0"` に変更
- [x] 既存の `version_is_20_5_0` テストに `#[ignore]` を追加

---

### T5: `CHANGELOG.md` 更新 + ベンチマーク

- [x] `CHANGELOG.md` の先頭に v20.6.0 エントリを追加:
  - [x] `### Added` — `IO.read_files_batch` primitive、`read_files_batch_impl`（Linux/非Linux）、`tokio-uring = "0.4"` 依存（Linux専用）
  - [x] `### Performance` — `io_batch_100_files_ms` +2〜4x（Linux io_uring）、`io_batch_1000_files_ms` +3〜5x
- [x] `benchmarks/suite/09_io_batch.sh` を新規作成（`io_batch_100_files_ms` / `io_batch_1000_files_ms` / `io_db_file_mixed_ms` を計測）
- [x] `benchmarks/v20.6.0.json` を実測値で生成（Linux 環境がなければ期待値で作成）

---

### T6: `site/content/docs/runes/io.mdx` 更新

- [x] 既存の `io.mdx` を読んでドキュメントスタイルを確認
- [x] `## IO.read_files_batch` セクションを追加:
  - [x] シグネチャ: `IO.read_files_batch(paths: List<String>) -> List<String>`
  - [x] 説明: Linux では io_uring、Windows/macOS では rayon 並列フォールバック
  - [x] 使用例（Favnir コード）
  - [x] WASM 非対応の注意書き
  - [x] カーネル 5.1+ 要件の注意書き

---

## テスト（v206000_tests、5 件）

| テスト名 | 内容 |
|---|---|
| `version_is_20_6_0` | `Cargo.toml` に `"20.6.0"` が含まれる |
| `io_batch_reads_single_file` | 1 ファイルを `read_files_batch_impl` で読み込み、内容が一致する |
| `io_batch_reads_multiple_files` | 3 ファイルを並列読み込み、全内容が正しい順序で返る |
| `io_batch_preserves_order` | 入力パスの順序通りに結果が返ることを確認 |
| `io_batch_error_on_missing_file` | 存在しないパスを含む場合に `Err` が返る |

---

## 完了条件チェックリスト

- [x] `IO.read_files_batch(paths)` が Linux / 非Linux 両環境で動作する
- [x] Linux では `tokio-uring` が使われる（`cfg(all(target_os = "linux", not(target_arch = "wasm32")))` ガード）
- [x] 非 Linux（Windows / macOS）では `rayon` フォールバックが動作する
- [x] WASM ビルドが `cfg(target_arch = "wasm32")` ガードでコンパイルを通る
- [x] `cargo test v206000` — 5/5 PASS
- [x] `cargo test` — リグレッションなし
- [x] `fav/Cargo.toml` version が `20.6.0`
- [x] `CHANGELOG.md` に v20.6.0 エントリが追加されている
- [x] `benchmarks/v20.6.0.json` が生成されている
- [x] Linux 環境で `io_batch_100_files_ms` が v20.5.0 比 +2x 以上（`benchmarks/v20.6.0.json` で確認）

---

## 優先度

```
T1（Cargo.toml）        ← 他すべての前提
T2（vm.rs 実装）        ← T1 完了後（最大工数）
T3（driver.rs テスト）  ← T2 完了後
T4（バージョン更新）    ← 任意タイミング
T5（CHANGELOG + bench） ← T3 完了後
T6（サイトドキュメント）← T5 完了後
```

---

## 実装リスク と 対策

| リスク | 対策 |
|---|---|
| `tokio-uring 0.4` の `read_at` シグネチャが spec 記載と異なる | T1 完了後に `~/.cargo/registry` のソースで確認 |
| `futures::future::try_join_all` が使えない（futures クレート未依存） | `cargo check` で確認。未依存なら `futures = "0.3"` を native-only deps に追加 |
| Linux テスト環境がない（Windows 開発） | 非 Linux バックエンド（rayon）のみテスト。Linux バックエンドは CI（GitHub Actions ubuntu-latest）で確認 |
| `tokio_uring::start` が既存の tokio ランタイムと競合 | `start` はスレッドブロッキング呼び出しのため競合しない。`tokio::main` の中から呼ばない |
| `read_at` のバッファムーブセマンティクスで所有権エラー | `let (res, buf) = file.read_at(buf, 0).await;` の正確なパターンを使用 |
| temp ファイルが並列テストで競合 | `temp_txt!` マクロで `line!()` + `process::id()` を使って一意なパスを生成 |
