// fav/src/debug.rs — v67.1.0 fav debug ステップ実行デバッガ / v67.2.0 Time-Travel Debugging

pub const DEBUG_HELP: &str = "\
fav debug — ステップ実行デバッガ v67.1.0

コマンド:
  run                    パイプラインを実行（各ステージ後に自動停止）
  step                   1 ステージ進む
  continue               次のブレークポイントまで実行
  inspect <expr>         レコード / ベクトルの内容を確認
  breakpoint <stage>     特定ステージで停止
  diff <row>             ステージ前後のレコード差分を表示
  quit                   デバッガを終了
";

pub fn cmd_debug(src: &str, _args: &[String]) -> String {
    format!(
        "[fav debug] v67.1.0 — ステップ実行モード\n\
         入力ファイル: {}\n\
         step / inspect / breakpoint / continue / quit が利用可能です。\n\
         'help' でコマンド一覧を表示。",
        src
    )
}

// v67.2.0 — Time-Travel Debugging（記録 & リプレイ）

pub const TIME_TRAVEL_HELP: &str = "\
Time-Travel Debugging:
  fav run pipeline.fav --record session.fav-trace   実行を .fav-trace に記録
  fav debug --replay session.fav-trace              .fav-trace からリプレイ

リプレイコマンド:
  rewind <step>    指定ステップに巻き戻す
  forward          1 ステップ進む
  inspect <expr>   現在ステップのデータを確認
  quit             リプレイを終了
";

#[allow(dead_code)]
pub(crate) fn cmd_debug_replay(trace_path: &str, _args: &[String]) -> String {
    format!(
        "[fav debug --replay] .fav-trace ファイル: {}\n\
         rewind / forward / inspect / quit が利用可能です。",
        trace_path
    )
}
