// fav/src/profiler/interactive.rs — v67.7.0 Interactive Profiling

pub const INTERACTIVE_HELP: &str = "\
fav profile --interactive — インタラクティブプロファイリング

使用例:
  fav profile --interactive pipeline.fav

コマンド:
  drill    ホットスポット（hotspot）をコード行レベルにドリルダウン
  next     次のホットスポットに移動
  quit     インタラクティブモードを終了

--interactive モードは各ホットスポットに対して Suggestion（最適化提案）を自動表示します。
";

pub fn cmd_profile_interactive(src: &str) -> String {
    // スタブ実装: 将来フェーズで実際のインタラクティブ REPL に置き換える
    format!(
        "[hotspot] Transform: 847ms (72% of total)\n\
         > drill\n\
           [line 12] collect {{ yield ... }} — 723ms (85% of Transform)\n\
         Suggestion: List.map に変換で 3× 高速化\n\
         \n\
         [hotspot] EmbedText: 1240ms (次のボトルネック)\n\
         > drill\n\
           [API calls] Rune.openai.embed: 1000 回 sequential\n\
         Suggestion: batch_embed(texts, batch_size: 50) で 20× 高速化\n\
         \n\
         (--interactive mode: {})",
        src
    )
}
