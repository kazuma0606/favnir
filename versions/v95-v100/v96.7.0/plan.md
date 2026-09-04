# Plan: v96.7.0 — Cross-system 型安全 JOIN（SAP エンティティ × Snowflake テーブル）

## Step 1: `runes/sap-odata/cross_system.fav` を新規作成

`SapSnowflakeJoin<A, B>` ジェネリックレコード型と `CrossSystem.join` スタブ関数を定義する。

```favnir
-- runes/sap-odata/cross_system.fav
-- SAP × Snowflake クロスシステム型安全 JOIN（v96.7.0）

public type SapSnowflakeJoin<A, B> = {
    sap_entity:       A,
    snowflake_record: B,
    join_key:         String
}

public fn CrossSystem.join<A, B>(
    left:      List<A>,
    right:     List<B>,
    left_key:  fn(A) -> String,
    right_key: fn(B) -> String
) -> List<SapSnowflakeJoin<A, B>> {
    List.empty()
}
```

## Step 2: `fav/src/driver.rs` に `mod v96700_tests` を追加

`mod v96600_tests` の直後に追加する。

テスト 1: `cross_system_fav_exists` — `cross_system.fav` に `SapSnowflakeJoin` が含まれることを確認。

テスト 2: `cross_system_fav_has_join_fn` — `cross_system.fav` に `CrossSystem.join` が含まれることを確認。

`runes/` 配下のファイルは `std::fs::read_to_string("../runes/sap-odata/cross_system.fav")` で読む
（`include_str!` ではなく `read_to_string`。他の runes テストと同じパターン）:

```rust
let content = std::fs::read_to_string("../runes/sap-odata/cross_system.fav")
    .expect("runes/sap-odata/cross_system.fav should exist");
```

## Step 3: `cargo test` で 4,205 tests, 0 failures を確認

## Step 4: `CHANGELOG.md` に v96.7.0 エントリを追加

## Step 5: `versions/current.md` を v96.7.0 に更新
