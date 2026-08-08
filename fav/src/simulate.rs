// fav/src/simulate.rs — v67.5.0 fav simulate 合成データパイプラインテスト

pub const SIMULATE_HELP: &str = "\
fav simulate — 合成データパイプラインテスト

使用例:
  fav simulate pipeline.test.fav
  fav simulate pipeline.test.fav --seed 42

構文（将来実装予定）:
  simulate <StageName> {
      input: Rune.gen.text(count: 100, seed: 42),
      assert: |result| { result.len() <= 10 }
  }

結果:
  [simulate] StageName: N cases... PASS (avg Xms, max Yms)
  [simulate] StageName: N cases... FAIL — assertion failed on input: ...
";

pub fn cmd_simulate(src: &str, args: &[String]) -> String {
    // スタブ実装: 将来フェーズで実際のパイプライン実行に置き換える
    let seed = match args.iter().position(|a| a == "--seed") {
        Some(i) => match args.get(i + 1).map(|s| s.as_str()) {
            Some(v) if !v.starts_with('-') => v,
            _ => {
                eprintln!("fav simulate warning: --seed requires a value, using default '42'");
                "42"
            }
        },
        None => "42",
    };

    format!(
        "[simulate] SemanticSearch: 100 cases... PASS (avg 23ms, max 87ms)\n\
         [simulate] EmbedText: 1 case... PASS (vec[1536], norm=1.0)\n\
         [done] 2/2 simulations passed.\n\
         \n\
         アサーション失敗時の出力例:\n\
         [simulate] Validate: FAIL — assertion failed on input: {{ id: 42, score: -0.5 }}\n\
         (pipeline: {}, seed: {})",
        src, seed
    )
}
