# Spec: v96.7.0 — Cross-system 型安全 JOIN（SAP エンティティ × Snowflake テーブル）

## Background

v96.6.0 まで SAP エンティティ（`BusinessPartner` 等）と Snowflake テーブルレコードは
別々の型として存在し、pipeline 内でそれらを結合する型安全な手段がなかった。

v96.7.0 では `SapSnowflakeJoin<A, B>` ジェネリック型と `CrossSystem.join` 関数を
`runes/sap-odata/cross_system.fav` に追加し、SAP ×  Snowflake のクロスシステム JOIN を
型レベルで表現できるようにする。

## Goals

1. `runes/sap-odata/cross_system.fav` を新規作成する
   - `SapSnowflakeJoin<A, B>` ジェネリックレコード型を定義する
   - `CrossSystem.join` 関数スタブを定義する（v96.7.0 は `List.empty()` 返し）
2. `fav/src/driver.rs` に `mod v96700_tests`（2 テスト）を追加する

## Favnir コード仕様

```favnir
-- runes/sap-odata/cross_system.fav
-- SAP × Snowflake クロスシステム型安全 JOIN（v96.7.0）

public type SapSnowflakeJoin<A, B> = {
    sap_entity:       A,
    snowflake_record: B,
    join_key:         String
}

-- SAP エンティティリストと Snowflake レコードリストをキー関数で JOIN する（v96.7.0 スタブ）
-- left_key:  SAP エンティティからキー文字列を取り出す関数
-- right_key: Snowflake レコードからキー文字列を取り出す関数
-- 戻り値: キーが一致したペアのリスト（完全実装は将来バージョン）
public fn CrossSystem.join<A, B>(
    left:      List<A>,
    right:     List<B>,
    left_key:  fn(A) -> String,
    right_key: fn(B) -> String
) -> List<SapSnowflakeJoin<A, B>> {
    List.empty()
}
```

## 使用例

```favnir
-- SAP BusinessPartner × Snowflake CRM テーブルを partner_id で JOIN
-- 注: Favnir は型推論により型パラメータを省略可能。
--     ロードマップ参考形: CrossSystem.join<BusinessPartner, CrmRecord>(...)
bind joined <- CrossSystem.join(
    bps, crm_records,
    fn(bp)  { bp.partner_id },
    fn(crm) { crm.sap_id }
)
```

## Success Criteria

- `runes/sap-odata/cross_system.fav` が存在し `SapSnowflakeJoin` を含む
- `runes/sap-odata/cross_system.fav` が `CrossSystem.join` を含む
- `cargo test` で 4,205 tests, 0 failures

## Error Codes

新規エラーコードなし。

## Files to Modify

| ファイル | 変更内容 |
|---|---|
| `runes/sap-odata/cross_system.fav` | 新規作成（`SapSnowflakeJoin<A,B>` 型 + `CrossSystem.join` スタブ） |
| `fav/src/driver.rs` | `mod v96700_tests`（2 テスト）を追加 |
