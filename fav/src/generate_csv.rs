/// v38.3.0 — fav generate --from csv: CSV から Favnir type + schema + expect を生成する

pub fn csv_to_favnir(csv_path: &str) -> Result<String, String> {
    // パス traversal ガード（v38.1.0 suggest.rs と同パターン）
    if csv_path.contains("..") {
        return Err(format!("invalid path (must not contain '..'): {}", csv_path));
    }
    let content = std::fs::read_to_string(csv_path)
        .map_err(|e| format!("cannot read {}: {}", csv_path, e))?;
    let headers = parse_headers(&content)?;
    Ok(generate_from_headers(&headers))
}

/// テスト用: ファイルパスではなく CSV 文字列から直接生成する
/// `pub(crate)` — binary crate 内のテスト専用（外部公開不要、generate_sql との対称性）
pub(crate) fn csv_to_favnir_from_str(csv_str: &str) -> Result<String, String> {
    let headers = parse_headers(csv_str)?;
    Ok(generate_from_headers(&headers))
}

fn parse_headers(csv: &str) -> Result<Vec<String>, String> {
    let first_line = csv.lines().next().ok_or("CSV is empty")?;
    Ok(first_line.split(',').map(|h| h.trim().to_string()).collect())
}

fn generate_from_headers(headers: &[String]) -> String {
    let fields = headers
        .iter()
        .map(|h| format!("    {}: String", h))
        .collect::<Vec<_>>()
        .join("\n");
    let first_col = headers.first().map(|s| s.as_str()).unwrap_or("id");
    // `fields` は `type Row` と `schema Row` の両ブロックで同じフィールド列を共有するため 2 回使用
    format!(
        "// Generated from CSV\ntype Row = {{\n{}\n}}\n\nschema Row {{\n{}\n}}\n\nexpect {{\n    all rows: Row -> rows.{} != \"\"\n}}\n",
        fields, fields, first_col
    )
}
