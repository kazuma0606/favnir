# Plan: v98.3.0 — SAP Analytics Cloud データプッシュ API（`SacDataset` 型）

## 実装順序

### Step 1: `runes/sap-odata/sac.fav` を新規作成

以下の順序で定義する:

1. `SacDataset` レコード型
   - `model_id: String` — SAC モデル ID
   - `rows: List<String>` — CSV 形式の行データ（ヘッダー行 + データ行）
2. `sac_push_mock(dataset: SacDataset) -> String` — テスト用モックヘルパー
   - `String.concat(["pushed:", dataset.model_id])` を返す
   - 実際の HTTP 通信は行わない

コメントはすべて `--` スタイルを使用する（`//` は Favnir 規約違反）。

### Step 2: `runes/sap-odata/sap_odata.fav` に追記

既存の analytics re-export ブロックの後に追加する:

1. `use sap_odata.sac` を use セクションに追加（analytics の直後）
2. SAC 型の re-export ブロックを追加（v98.2.0 の analytics re-export パターンと同形式）:
   ```favnir
   -- SAC 型 re-export（v98.3.0〜）
   public type SacDataset = sac.SacDataset
   public fn sac_push_mock(dataset: sac.SacDataset) -> String {
       sac.sac_push_mock(dataset)
   }
   ```

### Step 3: `fav/src/driver.rs` に `mod v98300_tests` を追加

`mod v98200_tests` の直後に `#[cfg(test)] mod v98300_tests { ... }` を追加する（2 テスト）:

- `sac_fav_exists`: `std::fs::read_to_string("../runes/sap-odata/sac.fav").expect(...)` でファイル存在を確認
- `sac_fav_has_sac_dataset`: `content.contains("SacDataset")` で型定義の存在を確認

### Step 4: `cargo test` で全 pass 確認

`cargo test -- --test-threads=1 2>&1 | grep "test result"` を実行し、4,241 tests, 0 failures を確認する。

### Step 5: `CHANGELOG.md` に v98.3.0 エントリを追加

`[v98.2.0]` エントリの直前（先頭）に `[v98.3.0]` エントリを追加する。

### Step 6: `versions/current.md` 更新

- `最終更新:` ヘッダーを `v98.3.0` に更新する
- 最新安定版を `v98.3.0` に更新する（テスト数 4,241）

### Step 7: CI 事前確認

`cargo test`（Step 4）実行後、`target/debug/fav` バイナリが存在することを前提とする。

- `cargo clippy --locked -- -D warnings` が pass することを確認する
- `./target/debug/fav fmt --check self/compiler.fav` が pass することを確認する
- `./target/debug/fav fmt --check self/checker.fav` が pass することを確認する
