/// v38.2.0 — fav generate --from sql: SQL を Favnir パイプラインに変換する

pub fn sql_to_favnir(sql: &str) -> String {
    let up = sql.trim().to_uppercase();
    if up.contains("JOIN") {
        generate_join(sql)
    } else if up.contains("WHERE") {
        generate_filter(sql)
    } else {
        generate_load(sql)
    }
}

fn generate_load(sql: &str) -> String {
    format!(
        "// Generated from SQL\nstage LoadData -> List<String> {{\n    db.query(ctx, {:?})\n}}\n\npipeline main {{\n    LoadData\n}}\n",
        sql
    )
}

fn generate_filter(sql: &str) -> String {
    format!(
        "// Generated from SQL (WHERE \u{2192} List.filter)\nstage LoadAndFilter -> List<String> {{\n    bind rows <- db.query(ctx, {:?})\n    List.filter(rows, |row| True)\n}}\n\npipeline main {{\n    LoadAndFilter\n}}\n",
        sql
    )
}

fn generate_join(sql: &str) -> String {
    // NOTE: JOIN → left_table / right_table はプレースホルダ。
    // 実際のテーブル名は元 SQL（Source SQL 行を参照）に合わせて書き換えてください。
    // WHERE 句が含まれる場合は JoinTables stage に List.filter を追加してください。
    format!(
        "// Generated from SQL (JOIN \u{2192} List.join_on)\n// TODO: replace left_table / right_table with actual table names\n// Source SQL: {:?}\nstage LoadLeft -> List<String> {{ db.query(ctx, \"SELECT * FROM left_table\") }}\nstage LoadRight -> List<String> {{ db.query(ctx, \"SELECT * FROM right_table\") }}\nstage JoinTables(left: List<String>, right: List<String>) -> List<String> {{\n    List.join_on(left, right, |l, r| True)\n}}\n\npipeline main {{\n    LoadLeft, LoadRight |> JoinTables\n}}\n",
        sql
    )
}
