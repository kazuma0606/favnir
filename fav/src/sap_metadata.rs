// fav/src/sap_metadata.rs — SAP $metadata EDMX XML パーサー（v93.1.0〜）
// v93.1.0: 構造体定義 + parse_edmx スタブ（Vec::new() 返し）
// v93.2.0 以降: EntityType → Favnir type 変換を段階的に実装

/// EDMX Property（EntityType の各フィールド）
/// edm_type: "Edm.String" / "Edm.Int32" / "Edm.Boolean" 等の EDM 型名
#[derive(Debug)]
pub struct EdmxProperty {
    pub name: String,
    pub edm_type: String,
}

/// EDMX EntityType（SAP エンティティの型定義）
#[derive(Debug)]
pub struct EdmxEntityType {
    pub name: String,
    pub properties: Vec<EdmxProperty>,
}

/// EDMX Schema（Namespace + EntityType リスト）
#[derive(Debug)]
pub struct EdmxSchema {
    pub namespace: String,
    pub entity_types: Vec<EdmxEntityType>,
}

/// EDMX XML を解析して EdmxSchema リストを返す
/// v93.1.0: スタブ実装（Vec::new() を返す）
/// 完全実装は v93.2.0 以降で段階的に追加する
pub fn parse_edmx(_xml: &str) -> Vec<EdmxSchema> {
    Vec::new()
}

/// EDM 型名を Favnir 型名に変換する（v93.2.0）
pub fn edm_type_to_favnir(edm_type: &str) -> &'static str {
    match edm_type {
        "Edm.String" | "Edm.DateTime" | "Edm.DateTimeOffset" | "Edm.Guid" => "String",
        "Edm.Int16" | "Edm.Int32" | "Edm.Int64" => "Int",
        "Edm.Decimal" | "Edm.Double" | "Edm.Single" => "Float",
        "Edm.Boolean" => "Bool",
        _ => "String",
    }
}

/// EdmxEntityType を Favnir type 定義文字列に変換する（v93.2.0）
/// エンティティ名: 先頭の「1 文字 + _」プレフィックス（A_/I_/C_ 等）と末尾の「Type」を除去
pub fn entity_type_to_favnir(et: &EdmxEntityType) -> String {
    // 先頭の "X_" パターン（ASCII 1文字 + '_'）を除去
    // SAFETY: SAP OData エンティティ名は OData 仕様上 ASCII のみ保証される
    //         (A_/I_/C_/Z_ 等) ため、バイトオフセット 2 でのスライスは安全。
    let name = if et.name.len() > 2 && et.name.as_bytes()[1] == b'_' {
        &et.name[2..]
    } else {
        &et.name
    };
    // 末尾の "Type" を除去
    let name = name.strip_suffix("Type").unwrap_or(name);

    let fields: String = et
        .properties
        .iter()
        .map(|p| format!("    {}: {}", p.name, edm_type_to_favnir(&p.edm_type)))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("type {} = {{\n{}\n}}", name, fields)
}

/// ナビゲーションプロパティ名のリストを Favnir コメント文字列に変換する（v93.3.0）
/// nav_names が空のときは空文字列を返す
/// 出力例: "-- Navigation properties (use with ExpandClause):\n-- \"to_BusinessPartnerAddress\""
pub fn nav_property_to_favnir_comment(nav_names: &[&str]) -> String {
    if nav_names.is_empty() {
        return String::new();
    }
    let mut out = String::from("-- Navigation properties (use with ExpandClause):");
    for name in nav_names {
        out.push_str(&format!("\n-- \"{}\"", name));
    }
    out
}

/// PascalCase → snake_case 変換（内部ヘルパー）
fn to_snake_case(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

/// EntityType 名と NavigationProperty 名から ExpandClause ヘルパー関数文字列を生成する（v93.3.0）
/// 例: nav_to_expand_helper_fn("BusinessPartner", "to_BusinessPartnerAddress")
///   → "fn business_partner_expand_business_partner_address() -> ExpandClause<BusinessPartner> {\n    expand_nav<BusinessPartner>([\"to_BusinessPartnerAddress\"])\n}"
/// 関数名 = {snake_entity}_expand_{snake_nav_body}（nav_name から "to_" を除いた部分を snake_case 化）
pub fn nav_to_expand_helper_fn(entity_name: &str, nav_name: &str) -> String {
    let snake_entity = to_snake_case(entity_name);
    // "to_" プレフィックスを除去してから snake_case 化
    let nav_body = nav_name.strip_prefix("to_").unwrap_or(nav_name);
    let snake_nav = to_snake_case(nav_body);
    let fn_name = format!("{}_expand_{}", snake_entity, snake_nav);
    format!(
        "fn {}() -> ExpandClause<{}> {{\n    expand_nav<{}>([\"{}\"])\n}}",
        fn_name, entity_name, entity_name, nav_name
    )
}

/// EDMX EnumType の各メンバー（v93.4.0）
#[derive(Debug)]
pub struct EdmxEnumMember {
    pub name: String,
}

/// EDMX EnumType（列挙型の型定義）（v93.4.0）
#[derive(Debug)]
pub struct EdmxEnumType {
    pub name: String,
    pub members: Vec<EdmxEnumMember>,
}

/// SCREAMING_SNAKE_CASE → PascalCase 変換（内部ヘルパー）
/// SAFETY: SAP OData EnumType 名は OData 仕様上 ASCII のみ保証される。
///         また各トークンの先頭は英字（A-Z）であることを前提とする（数字先頭トークンは非対応）。
fn screaming_snake_to_pascal(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_ascii_uppercase().to_string()
                        + &chars.as_str().to_ascii_lowercase()
                }
            }
        })
        .collect()
}

/// EDMX EnumType を Favnir ADT 型定義文字列に変換する（v93.4.0）
/// 例: EdmxEnumType { name: "YY1_BPKIND_CODE", members: [EdmxEnumMember { name: "1" }, ...] }
///   → "type Yy1BpkindCode =\n    | Val1\n    | Val2\n    | Val3"
pub fn enum_type_to_favnir(et: &EdmxEnumType) -> String {
    // EnumType 名が空の場合は不正な Favnir を生成しないようにガード
    if et.name.is_empty() {
        return String::new();
    }
    let type_name = screaming_snake_to_pascal(&et.name);
    let variants: String = et
        .members
        .iter()
        .map(|m| {
            // 先頭が数字の場合は "Val" プレフィックスを付与
            if m.name.starts_with(|c: char| c.is_ascii_digit()) {
                format!("    | Val{}", m.name)
            } else {
                format!("    | {}", m.name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("type {} =\n{}", type_name, variants)
}

/// カスタム OData サービス名からファイルヘッダーコメントを生成する（v96.5.0）
/// service_name: "ZMY_CUSTOM_SRV" 等のサービス名
/// 生成例: "-- Generated from SAP OData service: ZMY_CUSTOM_SRV\n-- Do not edit manually.\n"
pub fn generate_custom_service_header(service_name: &str) -> String {
    format!(
        "-- Generated from SAP OData service: {}\n-- Do not edit manually.\n",
        service_name
    )
}

// ── Unit tests (v96.5.0) ─────────────────────────────────────────────────────
#[cfg(test)]
mod v96500_sap_metadata_tests {
    use super::*;

    #[test]
    fn generate_custom_service_header_format() {
        let result = generate_custom_service_header("ZMY_CUSTOM_SRV");
        assert_eq!(
            result,
            "-- Generated from SAP OData service: ZMY_CUSTOM_SRV\n-- Do not edit manually.\n"
        );
    }

    #[test]
    fn generate_custom_service_header_empty_name() {
        let result = generate_custom_service_header("");
        assert!(
            result.starts_with("-- Generated from SAP OData service:"),
            "header should start with expected prefix even for empty service name"
        );
        assert!(
            result.contains("-- Do not edit manually."),
            "header should contain do-not-edit line"
        );
    }
}

/// 生成した Favnir ソースを fav fmt に通して標準フォーマットを適用する。
/// VM の `fmt_source_raw` primitive と同じバックエンド（`fmt_source_str`）を使用する。
/// フォーマット失敗時は元の `src` をそのまま返す。
pub fn apply_fmt_to_generated(src: &str) -> String {
    crate::compiler_fav_runner::fmt_source_str(src)
        .unwrap_or_else(|_| src.to_string())
}
