/// v39.6.0 — fav audit: 依存 Rune ライセンス一覧 / GPL・CVE 検出

pub fn cmd_audit(check_mode: bool) -> Result<(), String> {
    let runes = collect_rune_deps()?;
    if check_mode {
        // GPL マッチは大文字のみ検索（本実装時に to_uppercase() 等での正規化を追加すること）
        let violations: Vec<&str> = runes.iter()
            .filter(|r| r.contains("GPL"))
            .map(|r| r.as_str())
            .collect();
        if violations.is_empty() {
            println!("audit: OK ({} rune(s) checked)", runes.len());
            Ok(())
        } else {
            for v in &violations {
                eprintln!("audit violation: {}", v);
            }
            // check_mode=true: exit immediately (process::exit) — Ok/Err below is unreachable
            std::process::exit(1);
        }
    } else {
        // check_mode=false: 情報表示のみ（GPL/CVE フィルタは行わず全件リスト）
        for r in &runes {
            println!("{}", r);
        }
        println!("audit: {} rune(s) listed", runes.len());
        Ok(())
    }
}

fn collect_rune_deps() -> Result<Vec<String>, String> {
    // fav.toml の [dependencies] を読み込む（スタブ: 常に空リストを返す）
    // TODO: fav.toml parse 実装時にこの Ok(vec![]) を実際の依存解決ロジックに置き換えること
    // TODO: CVE データソース連携も後続バージョンで実装すること
    Ok(vec![])
}
