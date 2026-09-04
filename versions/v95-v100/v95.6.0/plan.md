# Plan: v95.6.0 — Function Import / Action Import

## Step 1: `runes/sap-odata/rpc.fav` 新規作成

1. `FunctionImportParam` 型エイリアス（`(String, String)` タプル）を定義する
2. `function_import<T>` スタブ関数を定義する
   - シグネチャ: `fn function_import<T>(cfg: SapConfig, function_name: String, params: List<FunctionImportParam>) -> Result<T, String>`
   - 戻り値: `Result.err("not implemented")`
3. `action_import` スタブ関数を定義する
   - シグネチャ: `fn action_import(cfg: SapConfig, action_name: String, params: List<FunctionImportParam>) -> Result<Unit, String>`
   - 戻り値: `Result.err("not implemented")`
4. 各定義に `public` を付与する

## Step 2: `fav/src/driver.rs` にテストを追加

1. `mod v95500_tests` の直後に `#[cfg(test)] mod v95600_tests { ... }` を追加する
2. `rpc_fav_exists` テスト: `../runes/sap-odata/rpc.fav` が存在することを確認
3. `rpc_fav_has_function_import` テスト: `rpc.fav` に `function_import` が含まれることを確認
4. `rpc_fav_has_action_import` テスト: `rpc.fav` に `action_import` が含まれることを確認

## Step 3: `cargo test` で全 pass 確認

- `cargo test 2>&1 | grep "test result"` を実行し、4,177 tests, 0 failures を確認する

## Step 4: CHANGELOG / current.md 更新

1. `CHANGELOG.md` の先頭に `[v95.6.0]` エントリを追加する
2. `versions/current.md` の最新安定版を `v95.6.0` に更新する

## Step 5: tasks.md 更新

- 本バージョンの `tasks.md` を COMPLETE ステータスに更新する
