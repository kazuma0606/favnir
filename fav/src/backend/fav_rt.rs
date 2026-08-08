//! v62.2.0: Favnir runtime stub — C プリミティブの最小セット定義。
//!
//! `fav build --link` で生成したネイティブバイナリにリンクされる
//! ランタイム関数のスタブソースを提供する。

/// ランタイムスタブのバージョン。
pub const FAV_RT_VERSION: &str = "0.1.0";

/// サポートするプリミティブ関数名（カンマ区切り）。
pub const FAV_RT_PRIMITIVES: &str = "fav_io_print,fav_io_panic";

/// C ランタイムスタブのソース文字列を返す。
///
/// `fav build --link` 時にこのソースを一時ファイルとして書き出し、
/// `cc` でオブジェクトと一緒にリンクすることで IO プリミティブを提供する。
pub fn fav_rt_stub_src() -> &'static str {
    r#"// Favnir runtime stub v0.1.0
#include <stdio.h>
#include <stdlib.h>

void fav_io_print(const char* s) { puts(s); }
void fav_io_panic(const char* msg) { fprintf(stderr, "panic: %s\n", msg); exit(1); }
"#
}
