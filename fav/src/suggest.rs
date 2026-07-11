/// v38.1.0 — fav suggest: エラーコードから修正案を生成する

pub fn cmd_suggest(error_code: &str, location: &str) -> Result<(), String> {
    // location が空の場合はファイル読み込みをスキップして組み込みヒントを返す
    let hint = if location.is_empty() {
        builtin_hint(error_code)
    } else {
        let source = read_source(location)?;
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            llm_suggest(&key, error_code, &source)
        } else {
            builtin_hint(error_code)
        }
    };
    println!("{}", hint);
    Ok(())
}

fn read_source(location: &str) -> Result<String, String> {
    let path = location.split(':').next().unwrap_or(location);
    // パス traversal ガード: ".." を含むパスは拒否する
    if path.contains("..") {
        return Err(format!("invalid path (must not contain '..'): {}", path));
    }
    std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read {}: {}", path, e))
}

fn builtin_hint(error_code: &str) -> String {
    match error_code {
        "E0001" => "Suggestion: Check for typos in variable names. Use `fav check` to see all defined variables.".to_string(),
        "E0007" => "Suggestion: The function may not be imported. Add the correct `import` statement at the top.".to_string(),
        "E0008" => "Suggestion: Check the number of arguments. Use `fav doc` to see function signatures.".to_string(),
        _ => format!("No built-in suggestion for {}. Set ANTHROPIC_API_KEY for LLM suggestions.", error_code),
    }
}

fn llm_suggest(_api_key: &str, error_code: &str, _source: &str) -> String {
    // 現在スタブ実装。真の HTTP 呼び出しは v39.x で本実装予定。
    format!(
        "[LLM stub] {}",
        builtin_hint(error_code)
    )
}
