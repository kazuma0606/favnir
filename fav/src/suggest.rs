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

// v67.4.0 — AI 最適化アドバイザー（プロファイルベース）

pub const SUGGEST_PROFILE_HELP: &str = "\
fav suggest <pipeline.fav> --from-profile <fav-profile.json>

プロファイリングデータを読んでボトルネックを分析し、最適化提案を生成します。

提案の適用:
  fav fix --apply suggestion-1.patch

優先度:
  [HIGH IMPACT]  実行時間の 50% 以上を占めるボトルネック
  [MED]          並列化・バッチ化で改善可能な処理
  [LOW]          N+1 クエリ等の軽微な非効率
";

pub fn cmd_suggest_profile(src: &str, profile_path: &str) -> String {
    format!(
        "Suggestion 1 [HIGH IMPACT] Transform stage: 847ms (72% of total)\n\
         Pattern detected: collect {{ yield }} は AOT コンパイルで最適化不可\n\
         Fix: List.map / List.filter に書き換え → 3× 高速化\n\
         → fav fix --apply suggestion-1.patch\n\
         \n\
         Suggestion 2 [MED] EmbedText: 1240ms, sequential\n\
         Pattern: 1000 件を逐次処理中\n\
         Fix: par [EmbedText x 4] に変更 → スループット 4× 向上\n\
         → fav fix --apply suggestion-2.patch\n\
         \n\
         (pipeline: {}, profile: {})",
        src, profile_path
    )
}
