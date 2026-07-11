/// v38.5.0 — fav explain --verbose: コンテキスト付き LLM 拡張説明

pub fn explain_verbose(error_code: &str, location: &str) -> String {
    let base = base_explanation(error_code);
    // コントロール文字を除去してターミナルインジェクション（ANSI エスケープ等）を防ぐ
    let safe_location: String = location.chars().filter(|c| !c.is_control()).collect();
    let context_note = if safe_location.is_empty() {
        String::new()
    } else {
        format!("\n\nContext ({}): [LLM stub — v38.7.0 で本実装予定]", safe_location)
    };
    format!("{}{}\n\nFix suggestion: [LLM stub — v38.7.0 で本実装予定]\n", base, context_note)
}

fn base_explanation(error_code: &str) -> String {
    match error_code {
        "E0001" => "E0001: Undefined variable. Check for typos or missing definitions.".to_string(),
        "E0007" => "E0007: Undefined function. Ensure the function is declared before use.".to_string(),
        "E0008" => "E0008: Wrong number of arguments. Check the function signature.".to_string(),
        _ => format!("{}: No built-in explanation available. Use a valid error code (e.g., E0001, E0007, E0008).", error_code),
    }
}
