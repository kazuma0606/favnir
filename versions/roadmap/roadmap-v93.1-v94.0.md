# Roadmap v93.1.0 〜 v94.0.0 — SAP Metadata Infer 1.0

Date: 2026-08-25
Status: 完了（v94.0.0 宣言済み・2026-08-30）

マスターロードマップ: [roadmap-v90.1-v95.0.md](roadmap-v90.1-v95.0.md)

---

## 前提

- 直前完了: v93.0.0「SAP QueryBuilder 1.0 宣言」（tests = 4,107）
- 本スプリントは SAP Advanced Era の第 4 スプリント
- 目標: v94.0.0「SAP Metadata Infer 1.0 宣言」（tests = 4,129）

### 着手前チェックリスト

- `versions/current.md` の最新安定版が v93.0.0 になっていることを確認する
- `versions/v90-v95/v93.1.0/` ディレクトリを作成し、spec.md / plan.md / tasks.md を準備すること
- `runes/sap-odata/query_builder.fav` に `QueryBuilder` が含まれることを確認する（v92.1.0 完了済みの証拠）
- `fav/src/driver.rs` に `mod v93000_tests` が存在することを確認する（v93.0.0 完了済みの証拠）

### スプリントの性格

SAP S/4HANA の `$metadata` エンドポイントが返す EDMX XML を解析し、**Favnir 型定義を自動生成**するスプリント。

`fav infer --from sap --metadata <url>` コマンドで型定義ファイルを生成することで、手動型定義の工数をゼロにする。
v10.8.0 の `fav infer --from snowflake --table <name>` と同じ `--from <source>` パターンを SAP に適用する。

A（基盤・パーサー）50% + B（CLI 拡充）30% + C（ドキュメント）20% の構成。

---

## バージョン一覧

| バージョン | 内容 | テスト数（実測ベース） | 状態 |
|---|---|---|---|
| v93.1.0 | EDMX XML パーサー基盤（`parse_edmx`） | 4120 + 2 = 4122 | 完了 |
| v93.2.0 | `EntityType` → Favnir `type` 変換 | 4122 + 2 = 4124 | 未着手 |
| v93.3.0 | `NavigationProperty` → `ExpandClause` フィールド生成 | 4124 + 2 = 4126 | 未着手 |
| v93.4.0 | `EnumType` → Favnir `type E = \| A \| B` 変換 | 4126 + 2 = 4128 | 未着手 |
| v93.5.0 | `fav infer --from sap --metadata <url>` CLI（URL 取得） | 4128 + 2 = 4130 | 未着手 |
| v93.6.0 | `fav infer --from sap --metadata-file <path>` CLI（ファイル読み込み） | 4130 + 2 = 4132 | 未着手 |
| v93.7.0 | 生成コードの `fav fmt` 適用（整形済み出力） | 4132 + 2 = 4134 | 未着手 |
| v93.8.0 | サイトドキュメント更新（`fav infer --sap-metadata` ガイド） | 4134 + 2 = 4136 | 未着手 |
| v93.9.0 | 安定化・コードフリーズ | 4136 + 2 = 4138 | 未着手 |
| v94.0.0 | SAP Metadata Infer 1.0 宣言 ★クリーンアップ | 4138 + 4 = 4142 | 未着手 |

---

## v93.1.0 — EDMX XML パーサー基盤

SAP OData `$metadata` が返す EDMX XML を解析する基盤を実装する。

```
-- $metadata レスポンスの例（EDMX 形式）
<edmx:Edmx Version="4.0">
  <edmx:DataServices>
    <Schema Namespace="API_BUSINESS_PARTNER">
      <EntityType Name="A_BusinessPartnerType">
        <Key><PropertyRef Name="BusinessPartner"/></Key>
        <Property Name="BusinessPartner" Type="Edm.String" MaxLength="10"/>
        <Property Name="BusinessPartnerName" Type="Edm.String" MaxLength="81"/>
        <Property Name="Country" Type="Edm.String" MaxLength="3"/>
      </EntityType>
    </Schema>
  </edmx:DataServices>
</edmx:Edmx>
```

**実装内容:**
- `fav/src/sap_metadata.rs`（新規作成）に `parse_edmx` 関数を実装
- `EdmxSchema` / `EdmxEntityType` / `EdmxProperty` 構造体を定義
- `driver.rs` に `mod v93100_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4107 + 2 = 4109）
- `sap_metadata_file_exists`: `fav/src/sap_metadata.rs` が存在する
- `parse_edmx_function_defined`: `sap_metadata.rs` に `parse_edmx` が含まれる

---

## v93.2.0 — `EntityType` → Favnir `type` 変換

解析した `EntityType` を Favnir の `type` 定義文字列に変換する。

```
-- 入力: EDMX EntityType
EntityType { name: "A_BusinessPartnerType", properties: [...] }

-- 出力: Favnir 型定義
type BusinessPartner = {
    BusinessPartner:     String,
    BusinessPartnerName: String,
    Country:             String
}
```

**EDM → Favnir 型マッピング:**

| EDM 型 | Favnir 型 |
|---|---|
| `Edm.String` | `String` |
| `Edm.Int32` / `Edm.Int64` | `Int` |
| `Edm.Decimal` | `Float` |
| `Edm.Boolean` | `Bool` |
| `Edm.DateTimeOffset` | `String` |
| `Edm.Guid` | `String` |

**実装内容:**
- `sap_metadata.rs` に `entity_type_to_favnir` 関数を追加
- `edm_type_to_favnir` 型マッピング関数を追加
- `driver.rs` に `mod v93200_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4109 + 2 = 4111）
- `entity_type_to_favnir_defined`: `sap_metadata.rs` に `entity_type_to_favnir` が含まれる
- `edm_type_to_favnir_defined`: `sap_metadata.rs` に `edm_type_to_favnir` が含まれる

---

## v93.3.0 — `NavigationProperty` → `ExpandClause` フィールド生成

`NavigationProperty` を解析し、`ExpandClause` のフィールドリストを生成するコードを追加する。

```
-- 入力: EDMX NavigationProperty
<NavigationProperty Name="to_BusinessPartnerAddress"
    Type="Collection(API_BUSINESS_PARTNER.A_BusinessPartnerAddressType)"/>

-- 出力: 生成コメント付き型定義
type BusinessPartner = {
    BusinessPartner:     String,
    BusinessPartnerName: String,
    -- Navigation properties (use with ExpandClause):
    -- "to_BusinessPartnerAddress"
}

-- 生成される ExpandClause ヘルパー
fn business_partner_expand_address() -> ExpandClause<BusinessPartner> {
    expand_nav<BusinessPartner>(["to_BusinessPartnerAddress"])
}
```

**実装内容:**
- `sap_metadata.rs` に `nav_property_to_favnir_comment` 関数を追加
- 展開ヘルパー関数の生成ロジックを追加
- `driver.rs` に `mod v93300_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4124 + 2 = 4126）
- `nav_property_parser_defined`: `sap_metadata.rs` に `nav_property_to_favnir_comment` が含まれる
- `nav_property_generates_expand_helper`: `sap_metadata.rs` に `nav_to_expand_helper_fn` が含まれる

---

## v93.4.0 — `EnumType` → Favnir `type E = | A | B` 変換

EDMX `EnumType` を Favnir の ADT 型に変換する。

```
-- 入力: EDMX EnumType
<EnumType Name="YY1_BPKIND_CODE">
  <Member Name="1" Value="1"/>
  <Member Name="2" Value="2"/>
  <Member Name="3" Value="3"/>
</EnumType>

-- 出力: Favnir ADT
type Yy1BpkindCode =
    | Val1
    | Val2
    | Val3
```

**実装内容:**
- `sap_metadata.rs` に `enum_type_to_favnir` 関数を追加
- `EdmxEnumType` / `EdmxEnumMember` 構造体を追加
- `driver.rs` に `mod v93400_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4126 + 2 = 4128）
- `enum_type_to_favnir_defined`: `sap_metadata.rs` に `enum_type_to_favnir` が含まれる
- `edmx_enum_type_struct_defined`: `sap_metadata.rs` に `EdmxEnumType` が含まれる

---

## v93.5.0 — `fav infer --from sap --metadata <url>` CLI

`$metadata` エンドポイントを HTTP で取得し、Favnir 型定義を生成する CLI コマンドを実装する。
v10.8.0 の `fav infer --from snowflake --table <name>` と同じ `--from <source>` パターンを採用する。

```
$ fav infer --from sap --metadata https://sandbox.api.sap.com/s4hanacloud/sap/opu/odata/sap/API_BUSINESS_PARTNER/$metadata

-- 生成例（stdout）
-- Generated by: fav infer --from sap --metadata
-- Source: API_BUSINESS_PARTNER
-- NOTE: parse_edmx is a stub — full XML parsing will be added in v93.8.0+

type BusinessPartner = {
    BusinessPartner:     String,
    BusinessPartnerName: String,
    Country:             String
}

type BusinessPartnerAddress = {
    BusinessPartner: String,
    AddressID:       String,
    StreetName:      String,
    CityName:        String
}
```

**実装内容:**
- `fav/src/infer.rs` に `infer_from_sap_metadata_url` 関数を追加
- `cli.fav` に `--from sap` + `--metadata` フラグを追加
- `driver.rs` に `mod v93500_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4128 + 2 = 4130）
- `infer_sap_metadata_url_function_defined`: `infer.rs` に `infer_from_sap_metadata_url` が含まれる
- `cli_fav_has_from_sap_flag`: `cli.fav` に `from sap` が含まれる

---

## v93.6.0 — `fav infer --from sap --metadata-file <path>` CLI

ローカルの EDMX ファイルから型定義を生成するオプションを追加する。オフライン環境や CI での利用を想定する。

```
$ fav infer --from sap --metadata-file ./metadata/API_BUSINESS_PARTNER.xml \
            --output ./runes/sap-odata/generated_types.fav

-- 生成後
$ cat ./runes/sap-odata/generated_types.fav
-- Generated by: fav infer --from sap --metadata-file
...
```

**実装内容:**
- `fav/src/infer.rs` に `infer_from_sap_metadata_file` 関数を追加
- `cli.fav` に `--metadata-file` / `--output` フラグを追加（`--from sap` と組み合わせて使用）
- `driver.rs` に `mod v93600_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4130 + 2 = 4132）
- `infer_sap_metadata_file_function_defined`: `infer.rs` に `infer_from_sap_metadata_file` が含まれる
- `cli_fav_has_metadata_file_flag`: `cli.fav` に `metadata-file` が含まれる

---

## v93.7.0 — 生成コードの `fav fmt` 適用

`fav infer --from sap --metadata` が生成するコードを自動フォーマット済みで出力する。

**実装内容:**
- `sap_metadata.rs` の生成出力を `fmt_source_raw` に通すパスを追加
- インデント・改行を Favnir 標準フォーマットに統一
- `driver.rs` に `mod v93700_tests` を追加（2 件）

**完了条件**: Rust テスト 2 件（4132 + 2 = 4134）
- `sap_metadata_generator_applies_fmt`: `sap_metadata.rs` に `fmt_source_raw` が含まれる
- `infer_output_is_formatted`: `sap_metadata.rs` に `formatted` が含まれる

---

## v93.8.0 — サイトドキュメント更新

`site/content/docs/cli/infer.mdx` および `site/content/docs/runes/sap-odata.mdx` を更新する。

**追加セクション:**
- `fav infer --from sap --metadata <url>` の使い方
- `fav infer --from sap --metadata-file <path>` の使い方（CI/オフライン向け）
- EDM 型 → Favnir 型マッピング表
- `NavigationProperty` と `ExpandClause` の対応表

**完了条件**: Rust テスト 2 件（4134 + 2 = 4136）
- `docs_infer_mentions_sap_metadata`: `infer.mdx` に `sap-metadata` が含まれる
- `docs_sap_odata_mentions_metadata_infer`: `sap-odata.mdx` に `metadata` が含まれる

---

## v93.9.0 — 安定化・コードフリーズ

v93.1〜v93.8 の全機能を通しで確認する最終安定化スプリント。

**実装内容:**
- `cargo test` 全 pass 確認（4,136 tests）
- `parse_edmx` → `entity_type_to_favnir` → `fmt_source_raw` の全パスを確認
- バグ修正のみ受け入れ（新機能追加なし）

**完了条件**: Rust テスト 2 件（4136 + 2 = 4138）
- `sap_metadata_smoke_url_and_file_cli`: `cli.fav` に `from sap` と `metadata-file` の両方が含まれる
- `sap_metadata_parser_handles_entity_and_enum`: `sap_metadata.rs` に `entity_type_to_favnir` と `enum_type_to_favnir` の両方が含まれる

---

## v94.0.0 — SAP Metadata Infer 1.0 宣言 ★クリーンアップ

**宣言文**:
> 「`fav infer --sap-metadata <url>` と打てば、SAP の $metadata から Favnir 型定義が自動生成される。
>  EntityType は `type` に、EnumType は ADT に、NavigationProperty は ExpandClause ヘルパーに変換される。
>  それが、Favnir SAP Metadata Infer 1.0 である。」

**クリーンアップ作業:**
- `cargo clean` 実施
- `Cargo.toml` バージョンを `94.0.0` に更新
- `CHANGELOG.md` / `MILESTONE.md` / `README.md` 更新
- `versions/current.md` を v94.0.0 に更新
- driver.rs 内の旧 `cargo_toml_version` テストを `94.0.0` に一括更新

**完了条件**: `v94000_tests` 4 件（4138 + 4 = 4142）
- `cargo_toml_version_is_94_0_0`
- `changelog_has_v94_0_0`
- `milestone_has_sap_metadata_infer`
- `readme_mentions_metadata_infer`

---

## テスト数推移（本スプリント）

| バージョン | テスト数 | 増加 |
|---|---|---|
| v93.0.0（ベース） | 4,120 | — |
| v93.1.0 | 4,122 | +2 |
| v93.2.0 | 4,124 | +2 |
| v93.3.0 | 4,126 | +2 |
| v93.4.0 | 4,128 | +2 |
| v93.5.0 | 4,130 | +2 |
| v93.6.0 | 4,132 | +2 |
| v93.7.0 | 4,134 | +2 |
| v93.8.0 | 4,136 | +2 |
| v93.9.0 | 4,138 | +2 |
| v94.0.0（宣言） | 4,142 | +4 |

**本スプリント合計**: +22 tests
