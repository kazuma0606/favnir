use crate::lsp::document_store::{CheckedDoc, DocumentStore};
use crate::lsp::hover::{position_to_char_offset, span_contains};
use crate::lsp::protocol::{CompletionItem, MarkupContent, Position, completion_kind};
use crate::middle::checker::{SymbolKind, Type};

const KEYWORDS: &[&str] = &[
    "fn",
    "type",
    "stage",
    "seq",
    "interface",
    "impl",
    "match",
    "if",
    "else",
    "bind",
    "chain",
    "collect",
    "yield",
    "public",
    "async",
    "for",
    "in",
    "where",
    "bench",
    "test",
];

// ── Builtin function table ────────────────────────────────────────────────────

pub struct BuiltinFn {
    pub namespace: &'static str,
    pub name: &'static str,
    /// Display signature, e.g. "(xs: List<'a>, f: ('a -> 'b)) -> List<'b>"
    pub signature: &'static str,
    /// Individual parameter labels for signature help
    pub params: &'static [&'static str],
}

pub const BUILTIN_NAMESPACES: &[&str] = &[
    "List", "String", "Map", "Result", "Option", "IO", "Json", "Csv", "Gen", "Http", "Llm",
    "DB", "AWS", "Env", "Debug", "Float", "Int", "T", "Schema",
];

pub const BUILTIN_FNS: &[BuiltinFn] = &[
    // ── List ──────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "List",
        name: "map",
        signature: "(xs: List<'a>, f: ('a -> 'b)) -> List<'b>",
        params: &["xs: List<'a>", "f: ('a -> 'b)"],
    },
    BuiltinFn {
        namespace: "List",
        name: "filter",
        signature: "(xs: List<'a>, f: ('a -> Bool)) -> List<'a>",
        params: &["xs: List<'a>", "f: ('a -> Bool)"],
    },
    BuiltinFn {
        namespace: "List",
        name: "fold",
        signature: "(xs: List<'a>, init: 'b, f: ('b, 'a) -> 'b) -> 'b",
        params: &["xs: List<'a>", "init: 'b", "f: ('b, 'a) -> 'b"],
    },
    BuiltinFn {
        namespace: "List",
        name: "length",
        signature: "(xs: List<'a>) -> Int",
        params: &["xs: List<'a>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "first",
        signature: "(xs: List<'a>) -> Option<'a>",
        params: &["xs: List<'a>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "last",
        signature: "(xs: List<'a>) -> Option<'a>",
        params: &["xs: List<'a>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "push",
        signature: "(xs: List<'a>, x: 'a) -> List<'a>",
        params: &["xs: List<'a>", "x: 'a"],
    },
    BuiltinFn {
        namespace: "List",
        name: "append",
        signature: "(xs: List<'a>, ys: List<'a>) -> List<'a>",
        params: &["xs: List<'a>", "ys: List<'a>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "concat",
        signature: "(xss: List<List<'a>>) -> List<'a>",
        params: &["xss: List<List<'a>>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "contains",
        signature: "(xs: List<'a>, x: 'a) -> Bool",
        params: &["xs: List<'a>", "x: 'a"],
    },
    BuiltinFn {
        namespace: "List",
        name: "find",
        signature: "(xs: List<'a>, f: ('a -> Bool)) -> Option<'a>",
        params: &["xs: List<'a>", "f: ('a -> Bool)"],
    },
    BuiltinFn {
        namespace: "List",
        name: "partition",
        signature: "(xs: List<'a>, f: ('a -> Bool)) -> (List<'a>, List<'a>)",
        params: &["xs: List<'a>", "f: ('a -> Bool)"],
    },
    BuiltinFn {
        namespace: "List",
        name: "zip_with",
        signature: "(xs: List<'a>, ys: List<'b>, f: ('a, 'b) -> 'c) -> List<'c>",
        params: &["xs: List<'a>", "ys: List<'b>", "f: ('a, 'b) -> 'c"],
    },
    BuiltinFn {
        namespace: "List",
        name: "flat_map",
        signature: "(xs: List<'a>, f: ('a -> List<'b>)) -> List<'b>",
        params: &["xs: List<'a>", "f: ('a -> List<'b>)"],
    },
    BuiltinFn {
        namespace: "List",
        name: "chunk",
        signature: "(xs: List<'a>, n: Int) -> List<List<'a>>",
        params: &["xs: List<'a>", "n: Int"],
    },
    BuiltinFn {
        namespace: "List",
        name: "take_while",
        signature: "(xs: List<'a>, f: ('a -> Bool)) -> List<'a>",
        params: &["xs: List<'a>", "f: ('a -> Bool)"],
    },
    BuiltinFn {
        namespace: "List",
        name: "drop_while",
        signature: "(xs: List<'a>, f: ('a -> Bool)) -> List<'a>",
        params: &["xs: List<'a>", "f: ('a -> Bool)"],
    },
    BuiltinFn {
        namespace: "List",
        name: "unique",
        signature: "(xs: List<'a>) -> List<'a>",
        params: &["xs: List<'a>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "sort",
        signature: "(xs: List<'a>, f: ('a, 'a) -> Int) -> List<'a>",
        params: &["xs: List<'a>", "f: ('a, 'a) -> Int"],
    },
    BuiltinFn {
        namespace: "List",
        name: "reverse",
        signature: "(xs: List<'a>) -> List<'a>",
        params: &["xs: List<'a>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "sum",
        signature: "(xs: List<Int>) -> Int",
        params: &["xs: List<Int>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "count",
        signature: "(xs: List<'a>, f: ('a -> Bool)) -> Int",
        params: &["xs: List<'a>", "f: ('a -> Bool)"],
    },
    BuiltinFn {
        namespace: "List",
        name: "min",
        signature: "(xs: List<Int>) -> Option<Int>",
        params: &["xs: List<Int>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "max",
        signature: "(xs: List<Int>) -> Option<Int>",
        params: &["xs: List<Int>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "empty",
        signature: "() -> List<'a>",
        params: &[],
    },
    BuiltinFn {
        namespace: "List",
        name: "singleton",
        signature: "(x: 'a) -> List<'a>",
        params: &["x: 'a"],
    },
    BuiltinFn {
        namespace: "List",
        name: "is_empty",
        signature: "(xs: List<'a>) -> Bool",
        params: &["xs: List<'a>"],
    },
    BuiltinFn {
        namespace: "List",
        name: "range",
        signature: "(start: Int, end: Int) -> List<Int>",
        params: &["start: Int", "end: Int"],
    },
    // ── String ────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "String",
        name: "length",
        signature: "(s: String) -> Int",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "split",
        signature: "(s: String, sep: String) -> List<String>",
        params: &["s: String", "sep: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "join",
        signature: "(xs: List<String>, sep: String) -> String",
        params: &["xs: List<String>", "sep: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "contains",
        signature: "(s: String, sub: String) -> Bool",
        params: &["s: String", "sub: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "trim",
        signature: "(s: String) -> String",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "trim_start",
        signature: "(s: String) -> String",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "trim_end",
        signature: "(s: String) -> String",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "starts_with",
        signature: "(s: String, prefix: String) -> Bool",
        params: &["s: String", "prefix: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "ends_with",
        signature: "(s: String, suffix: String) -> Bool",
        params: &["s: String", "suffix: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "replace",
        signature: "(s: String, from: String, to: String) -> String",
        params: &["s: String", "from: String", "to: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "to_upper",
        signature: "(s: String) -> String",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "to_lower",
        signature: "(s: String) -> String",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "concat",
        signature: "(s1: String, s2: String) -> String",
        params: &["s1: String", "s2: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "slice",
        signature: "(s: String, start: Int, end: Int) -> String",
        params: &["s: String", "start: Int", "end: Int"],
    },
    BuiltinFn {
        namespace: "String",
        name: "index_of",
        signature: "(s: String, sub: String) -> Option<Int>",
        params: &["s: String", "sub: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "pad_left",
        signature: "(s: String, n: Int, ch: String) -> String",
        params: &["s: String", "n: Int", "ch: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "pad_right",
        signature: "(s: String, n: Int, ch: String) -> String",
        params: &["s: String", "n: Int", "ch: String"],
    },
    BuiltinFn {
        namespace: "String",
        name: "repeat",
        signature: "(s: String, n: Int) -> String",
        params: &["s: String", "n: Int"],
    },
    BuiltinFn {
        namespace: "String",
        name: "is_empty",
        signature: "(s: String) -> Bool",
        params: &["s: String"],
    },
    // ── Map ───────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Map",
        name: "empty",
        signature: "() -> Map<'k, 'v>",
        params: &[],
    },
    BuiltinFn {
        namespace: "Map",
        name: "insert",
        signature: "(m: Map<'k, 'v>, key: 'k, val: 'v) -> Map<'k, 'v>",
        params: &["m: Map<'k, 'v>", "key: 'k", "val: 'v"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "get",
        signature: "(m: Map<'k, 'v>, key: 'k) -> Option<'v>",
        params: &["m: Map<'k, 'v>", "key: 'k"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "remove",
        signature: "(m: Map<'k, 'v>, key: 'k) -> Map<'k, 'v>",
        params: &["m: Map<'k, 'v>", "key: 'k"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "keys",
        signature: "(m: Map<'k, 'v>) -> List<'k>",
        params: &["m: Map<'k, 'v>"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "values",
        signature: "(m: Map<'k, 'v>) -> List<'v>",
        params: &["m: Map<'k, 'v>"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "contains_key",
        signature: "(m: Map<'k, 'v>, key: 'k) -> Bool",
        params: &["m: Map<'k, 'v>", "key: 'k"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "size",
        signature: "(m: Map<'k, 'v>) -> Int",
        params: &["m: Map<'k, 'v>"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "map_values",
        signature: "(m: Map<'k, 'a>, f: ('a -> 'b)) -> Map<'k, 'b>",
        params: &["m: Map<'k, 'a>", "f: ('a -> 'b)"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "from_list",
        signature: "(pairs: List<('k, 'v)>) -> Map<'k, 'v>",
        params: &["pairs: List<('k, 'v)>"],
    },
    BuiltinFn {
        namespace: "Map",
        name: "to_list",
        signature: "(m: Map<'k, 'v>) -> List<('k, 'v)>",
        params: &["m: Map<'k, 'v>"],
    },
    // ── Result ────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Result",
        name: "ok",
        signature: "(value: 'a) -> Result<'a, 'e>",
        params: &["value: 'a"],
    },
    BuiltinFn {
        namespace: "Result",
        name: "err",
        signature: "(error: 'e) -> Result<'a, 'e>",
        params: &["error: 'e"],
    },
    BuiltinFn {
        namespace: "Result",
        name: "map",
        signature: "(r: Result<'a, 'e>, f: ('a -> 'b)) -> Result<'b, 'e>",
        params: &["r: Result<'a, 'e>", "f: ('a -> 'b)"],
    },
    BuiltinFn {
        namespace: "Result",
        name: "map_err",
        signature: "(r: Result<'a, 'e>, f: ('e -> 'f)) -> Result<'a, 'f>",
        params: &["r: Result<'a, 'e>", "f: ('e -> 'f)"],
    },
    BuiltinFn {
        namespace: "Result",
        name: "and_then",
        signature: "(r: Result<'a, 'e>, f: ('a -> Result<'b, 'e>)) -> Result<'b, 'e>",
        params: &["r: Result<'a, 'e>", "f: ('a -> Result<'b, 'e>)"],
    },
    BuiltinFn {
        namespace: "Result",
        name: "is_ok",
        signature: "(r: Result<'a, 'e>) -> Bool",
        params: &["r: Result<'a, 'e>"],
    },
    BuiltinFn {
        namespace: "Result",
        name: "is_err",
        signature: "(r: Result<'a, 'e>) -> Bool",
        params: &["r: Result<'a, 'e>"],
    },
    BuiltinFn {
        namespace: "Result",
        name: "unwrap_or",
        signature: "(r: Result<'a, 'e>, default: 'a) -> 'a",
        params: &["r: Result<'a, 'e>", "default: 'a"],
    },
    // ── Option ────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Option",
        name: "some",
        signature: "(value: 'a) -> Option<'a>",
        params: &["value: 'a"],
    },
    BuiltinFn {
        namespace: "Option",
        name: "none",
        signature: "() -> Option<'a>",
        params: &[],
    },
    BuiltinFn {
        namespace: "Option",
        name: "map",
        signature: "(opt: Option<'a>, f: ('a -> 'b)) -> Option<'b>",
        params: &["opt: Option<'a>", "f: ('a -> 'b)"],
    },
    BuiltinFn {
        namespace: "Option",
        name: "and_then",
        signature: "(opt: Option<'a>, f: ('a -> Option<'b>)) -> Option<'b>",
        params: &["opt: Option<'a>", "f: ('a -> Option<'b>)"],
    },
    BuiltinFn {
        namespace: "Option",
        name: "unwrap_or",
        signature: "(opt: Option<'a>, default: 'a) -> 'a",
        params: &["opt: Option<'a>", "default: 'a"],
    },
    BuiltinFn {
        namespace: "Option",
        name: "is_some",
        signature: "(opt: Option<'a>) -> Bool",
        params: &["opt: Option<'a>"],
    },
    BuiltinFn {
        namespace: "Option",
        name: "is_none",
        signature: "(opt: Option<'a>) -> Bool",
        params: &["opt: Option<'a>"],
    },
    // ── IO ────────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "IO",
        name: "println",
        signature: "(s: String) -> Unit !Io",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "IO",
        name: "print",
        signature: "(s: String) -> Unit !Io",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "IO",
        name: "read_line",
        signature: "() -> String !Io",
        params: &[],
    },
    BuiltinFn {
        namespace: "IO",
        name: "read_file",
        signature: "(path: String) -> Result<String, String> !Io",
        params: &["path: String"],
    },
    BuiltinFn {
        namespace: "IO",
        name: "write_file",
        signature: "(path: String, content: String) -> Result<Unit, String> !Io",
        params: &["path: String", "content: String"],
    },
    BuiltinFn {
        namespace: "IO",
        name: "append_file",
        signature: "(path: String, content: String) -> Result<Unit, String> !Io",
        params: &["path: String", "content: String"],
    },
    BuiltinFn {
        namespace: "IO",
        name: "file_exists",
        signature: "(path: String) -> Bool !Io",
        params: &["path: String"],
    },
    BuiltinFn {
        namespace: "IO",
        name: "now_ms",
        signature: "() -> Int !Io",
        params: &[],
    },
    // ── Json ──────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Json",
        name: "encode",
        signature: "(value: 'a) -> String",
        params: &["value: 'a"],
    },
    BuiltinFn {
        namespace: "Json",
        name: "decode",
        signature: "(s: String) -> Result<'a, String>",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "Json",
        name: "pretty",
        signature: "(s: String) -> String",
        params: &["s: String"],
    },
    // ── Csv ───────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Csv",
        name: "read",
        signature: "(path: String) -> Result<List<'a>, String> !Io",
        params: &["path: String"],
    },
    BuiltinFn {
        namespace: "Csv",
        name: "write_file",
        signature: "(path: String, rows: List<'a>) -> Result<Unit, String> !Io",
        params: &["path: String", "rows: List<'a>"],
    },
    BuiltinFn {
        namespace: "Csv",
        name: "parse",
        signature: "(s: String) -> Result<List<'a>, String>",
        params: &["s: String"],
    },
    // ── Gen ───────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Gen",
        name: "uuid",
        signature: "() -> String !Gen",
        params: &[],
    },
    BuiltinFn {
        namespace: "Gen",
        name: "uuid_v7",
        signature: "() -> String !Gen",
        params: &[],
    },
    BuiltinFn {
        namespace: "Gen",
        name: "nano_id",
        signature: "(n: Int) -> String !Gen",
        params: &["n: Int"],
    },
    // ── Http ──────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Http",
        name: "get",
        signature: "(url: String) -> Result<String, String> !Http",
        params: &["url: String"],
    },
    BuiltinFn {
        namespace: "Http",
        name: "get_json",
        signature: "(url: String) -> Result<'a, String> !Http",
        params: &["url: String"],
    },
    BuiltinFn {
        namespace: "Http",
        name: "post",
        signature: "(url: String, body: String) -> Result<String, String> !Http",
        params: &["url: String", "body: String"],
    },
    BuiltinFn {
        namespace: "Http",
        name: "post_json",
        signature: "(url: String, body: 'a) -> Result<'b, String> !Http",
        params: &["url: String", "body: 'a"],
    },
    // ── Llm ───────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Llm",
        name: "complete",
        signature: "(prompt: String) -> Result<String, String> !Llm",
        params: &["prompt: String"],
    },
    BuiltinFn {
        namespace: "Llm",
        name: "chat",
        signature: "(messages: List<{role: String, content: String}>) -> Result<String, String> !Llm",
        params: &["messages: List<{role: String, content: String}>"],
    },
    BuiltinFn {
        namespace: "Llm",
        name: "extract",
        signature: "(prompt: String, data: String) -> Result<'a, String> !Llm",
        params: &["prompt: String", "data: String"],
    },
    // ── DB ────────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "DB",
        name: "query",
        signature: "(sql: String) -> Result<List<'a>, String> !Db",
        params: &["sql: String"],
    },
    BuiltinFn {
        namespace: "DB",
        name: "exec",
        signature: "(sql: String) -> Result<Unit, String> !Db",
        params: &["sql: String"],
    },
    BuiltinFn {
        namespace: "DB",
        name: "query_one",
        signature: "(sql: String) -> Result<'a, String> !Db",
        params: &["sql: String"],
    },
    // ── Env ───────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Env",
        name: "get_var",
        signature: "(name: String) -> Option<String>",
        params: &["name: String"],
    },
    BuiltinFn {
        namespace: "Env",
        name: "now_ms",
        signature: "() -> Int",
        params: &[],
    },
    BuiltinFn {
        namespace: "Env",
        name: "sleep_ms",
        signature: "(ms: Int) -> Unit !Io",
        params: &["ms: Int"],
    },
    // ── Debug ─────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Debug",
        name: "show_raw",
        signature: "(value: 'a) -> String",
        params: &["value: 'a"],
    },
    // ── Float ─────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Float",
        name: "parse",
        signature: "(s: String) -> Result<Float, String>",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "Float",
        name: "to_string",
        signature: "(n: Float) -> String",
        params: &["n: Float"],
    },
    BuiltinFn {
        namespace: "Float",
        name: "round",
        signature: "(n: Float) -> Int",
        params: &["n: Float"],
    },
    BuiltinFn {
        namespace: "Float",
        name: "floor",
        signature: "(n: Float) -> Int",
        params: &["n: Float"],
    },
    BuiltinFn {
        namespace: "Float",
        name: "ceil",
        signature: "(n: Float) -> Int",
        params: &["n: Float"],
    },
    BuiltinFn {
        namespace: "Float",
        name: "abs",
        signature: "(n: Float) -> Float",
        params: &["n: Float"],
    },
    BuiltinFn {
        namespace: "Float",
        name: "sqrt",
        signature: "(n: Float) -> Float",
        params: &["n: Float"],
    },
    // ── Int ───────────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Int",
        name: "parse",
        signature: "(s: String) -> Result<Int, String>",
        params: &["s: String"],
    },
    BuiltinFn {
        namespace: "Int",
        name: "to_string",
        signature: "(n: Int) -> String",
        params: &["n: Int"],
    },
    BuiltinFn {
        namespace: "Int",
        name: "abs",
        signature: "(n: Int) -> Int",
        params: &["n: Int"],
    },
    BuiltinFn {
        namespace: "Int",
        name: "to_float",
        signature: "(n: Int) -> Float",
        params: &["n: Int"],
    },
    // ── Schema / T ────────────────────────────────────────────────────────────
    BuiltinFn {
        namespace: "Schema",
        name: "adapt_one",
        signature: "(json: String) -> Result<'a, String>",
        params: &["json: String"],
    },
    BuiltinFn {
        namespace: "Schema",
        name: "adapt_list",
        signature: "(json: String) -> Result<List<'a>, String>",
        params: &["json: String"],
    },
    BuiltinFn {
        namespace: "T",
        name: "adapt_one",
        signature: "(json: String) -> Result<'a, String>",
        params: &["json: String"],
    },
    BuiltinFn {
        namespace: "T",
        name: "adapt_list",
        signature: "(json: String) -> Result<List<'a>, String>",
        params: &["json: String"],
    },
];

pub const KNOWN_RUNES: &[(&str, &str)] = &[
    ("aws", "AWS S3/SQS/DynamoDB !AWS"),
    ("cache", "Cache operations !Cache"),
    ("csv", "CSV read/write !Io"),
    ("db", "SQL database !Db"),
    ("email", "Email sending !Io"),
    ("fs", "Filesystem operations !Io"),
    ("gen", "UUID/NanoId generation !Gen"),
    ("graphql", "GraphQL client !Http"),
    ("grpc", "gRPC client !Http"),
    ("http", "HTTP client !Http"),
    ("json", "JSON encode/decode"),
    ("llm", "LLM (Claude/OpenAI) !Llm"),
    ("queue", "Message queue !Queue"),
    ("slack", "Slack messaging !Io"),
    ("sql", "SQL query builder !Db"),
];

// ── Completion handler ────────────────────────────────────────────────────────

pub fn handle_completion(
    store: &DocumentStore,
    uri: &str,
    pos: Position,
    trigger_char: Option<String>,
) -> Vec<CompletionItem> {
    let Some(doc) = store.get(uri) else {
        return Vec::new();
    };
    let Some(offset) = position_to_char_offset(&doc.source, pos) else {
        return Vec::new();
    };

    if trigger_char.as_deref() == Some(".") {
        // Check if preceding token is a builtin namespace → module completion
        if let Some(ns) = namespace_before_dot(&doc.source, offset) {
            if BUILTIN_NAMESPACES.contains(&ns.as_str()) {
                return module_completions(&ns);
            }
        }
        return field_completions(doc, offset);
    }

    // Rune import context → suggest known rune names
    if is_rune_import_context(&doc.source, offset) {
        return rune_completions();
    }

    let mut items = Vec::new();
    items.extend(global_completions(doc));
    items.extend(keyword_completions());
    items.extend(snippet_completions());
    items
}

// ── Module completion ─────────────────────────────────────────────────────────

pub fn module_completions(ns: &str) -> Vec<CompletionItem> {
    BUILTIN_FNS
        .iter()
        .filter(|f| f.namespace == ns)
        .map(|f| CompletionItem {
            label: f.name.to_string(),
            kind: completion_kind::FUNCTION,
            detail: Some(format!("{}.{}{}", f.namespace, f.name, f.signature)),
            insert_text: None,
            insert_text_format: None,
            documentation: None,
        })
        .collect()
}

/// Extract the identifier just before the `.` trigger character.
/// `offset` is the byte offset right after the `.`.
fn namespace_before_dot(src: &str, offset: usize) -> Option<String> {
    let dot_byte = offset.checked_sub(1)?; // byte offset of '.'
    let before_dot = src.get(..dot_byte)?;
    let ns: String = before_dot
        .chars()
        .rev()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    if ns.is_empty() { None } else { Some(ns) }
}

// ── Rune completion ───────────────────────────────────────────────────────────

fn rune_completions() -> Vec<CompletionItem> {
    KNOWN_RUNES
        .iter()
        .map(|(name, desc)| CompletionItem {
            label: name.to_string(),
            kind: completion_kind::MODULE,
            detail: Some(desc.to_string()),
            insert_text: None,
            insert_text_format: None,
            documentation: None,
        })
        .collect()
}

/// Returns true when the text before `offset` on the current line looks like
/// `import rune "` (possibly with leading whitespace), indicating the user is
/// typing a rune name inside the string literal.
fn is_rune_import_context(src: &str, offset: usize) -> bool {
    let before = &src[..offset.min(src.len())];
    let line_start = before.rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_so_far = &before[line_start..];
    let trimmed = line_so_far.trim_start();
    trimmed.starts_with("import rune \"")
}

// ── Existing helpers ──────────────────────────────────────────────────────────

fn field_completions(doc: &CheckedDoc, offset: usize) -> Vec<CompletionItem> {
    let Some(dot_offset) = offset.checked_sub(1) else {
        return Vec::new();
    };
    let Some(ty) = doc
        .type_at
        .iter()
        .filter(|(span, _)| span_contains(span, dot_offset.saturating_sub(1)))
        .min_by_key(|(span, _)| span.end.saturating_sub(span.start))
        .map(|(_, ty)| ty)
    else {
        return Vec::new();
    };
    let Some(type_name) = named_record_type(ty) else {
        return Vec::new();
    };
    let Some(fields) = doc.record_fields.get(type_name) else {
        return Vec::new();
    };

    fields
        .iter()
        .map(|(name, ty)| CompletionItem {
            label: name.clone(),
            kind: completion_kind::FIELD,
            detail: Some(ty.display()),
            insert_text: None,
            insert_text_format: None,
            documentation: None,
        })
        .collect()
}

fn named_record_type(ty: &Type) -> Option<&str> {
    match ty {
        Type::Named(name, _) => Some(name.as_str()),
        _ => None,
    }
}

fn global_completions(doc: &CheckedDoc) -> Vec<CompletionItem> {
    doc.symbols
        .iter()
        .map(|symbol| CompletionItem {
            label: symbol.name.clone(),
            kind: match symbol.kind {
                SymbolKind::Function => completion_kind::FUNCTION,
                SymbolKind::Type | SymbolKind::Stage | SymbolKind::Seq | SymbolKind::Interface => {
                    completion_kind::CLASS
                }
            },
            detail: Some(symbol.detail.clone()),
            insert_text: None,
            insert_text_format: None,
            documentation: doc
                .doc_comments
                .get(&symbol.name)
                .map(|text| MarkupContent {
                    kind: "markdown".to_string(),
                    value: text.clone(),
                }),
        })
        .collect()
}

fn keyword_completions() -> Vec<CompletionItem> {
    KEYWORDS
        .iter()
        .map(|keyword| CompletionItem {
            label: (*keyword).to_string(),
            kind: completion_kind::KEYWORD,
            detail: None,
            insert_text: None,
            insert_text_format: None,
            documentation: None,
        })
        .collect()
}

fn snippet_completions() -> Vec<CompletionItem> {
    [
        (
            "fn",
            "fn ${1:name}(${2:param}: ${3:Type}) -> ${4:RetType} {\n    $0\n}",
        ),
        ("type", "type ${1:Name} = { ${2:field}: ${3:Type} }"),
        (
            "interface",
            "interface ${1:Name} {\n    ${2:method}: ${3:Type}\n}",
        ),
        ("match", "match ${1:expr} {\n    ${2:pattern} => $0\n}"),
    ]
    .into_iter()
    .map(|(label, insert_text)| CompletionItem {
        label: label.to_string(),
        kind: completion_kind::SNIPPET,
        detail: None,
        insert_text: Some(insert_text.to_string()),
        insert_text_format: Some(2),
        documentation: None,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::document_store::DocumentStore;
    use crate::lsp::protocol::Position;

    #[test]
    fn completion_returns_field_items_on_dot_trigger() {
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///main.fav",
            "type User = { name: String age: Int }\nfn get(user: User) -> String = user.name"
                .to_string(),
        );
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position {
                line: 1,
                character: 36,
            },
            Some(".".to_string()),
        );
        assert!(items.iter().any(|item| item.label == "name"));
        assert!(items.iter().any(|item| item.label == "age"));
    }

    #[test]
    fn completion_returns_global_fn_name() {
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///main.fav",
            "fn double(n: Int) -> Int = n * 2\nfn main() -> Int = do".to_string(),
        );
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position {
                line: 1,
                character: 20,
            },
            None,
        );
        assert!(items.iter().any(|item| item.label == "double"));
    }

    #[test]
    fn completion_includes_keywords() {
        let mut store = DocumentStore::new();
        store.open_or_change("file:///main.fav", "fn main() -> Int = 1".to_string());
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position {
                line: 0,
                character: 10,
            },
            None,
        );
        assert!(items.iter().any(|item| item.label == "match"));
        assert!(items.iter().any(|item| item.label == "bind"));
    }

    #[test]
    fn completion_includes_snippets() {
        let mut store = DocumentStore::new();
        store.open_or_change("file:///main.fav", "fn main() -> Int = 1".to_string());
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position {
                line: 0,
                character: 10,
            },
            None,
        );
        assert!(
            items
                .iter()
                .any(|item| item.label == "fn" && item.insert_text_format == Some(2))
        );
    }

    #[test]
    fn module_completion_list_returns_map_and_filter() {
        let items = module_completions("List");
        assert!(items.iter().any(|item| item.label == "map"), "expected 'map' in List completions");
        assert!(items.iter().any(|item| item.label == "filter"), "expected 'filter' in List completions");
        assert!(items.iter().any(|item| item.label == "length"), "expected 'length' in List completions");
    }

    #[test]
    fn module_completion_string_returns_split_and_trim() {
        let items = module_completions("String");
        assert!(items.iter().any(|item| item.label == "split"), "expected 'split' in String completions");
        assert!(items.iter().any(|item| item.label == "trim"), "expected 'trim' in String completions");
    }

    #[test]
    fn handle_completion_dot_on_list_returns_module_items() {
        let mut store = DocumentStore::new();
        // "List." at end of line → position character 5
        store.open_or_change("file:///main.fav", "List.".to_string());
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position { line: 0, character: 5 },
            Some(".".to_string()),
        );
        assert!(items.iter().any(|item| item.label == "map"), "expected List.map in completions");
        assert!(items.iter().any(|item| item.label == "filter"), "expected List.filter in completions");
    }

    #[test]
    fn handle_completion_rune_import_returns_rune_names() {
        let mut store = DocumentStore::new();
        store.open_or_change(
            "file:///main.fav",
            "import rune \"".to_string(),
        );
        let items = handle_completion(
            &store,
            "file:///main.fav",
            Position { line: 0, character: 13 },
            None,
        );
        assert!(items.iter().any(|item| item.label == "http"), "expected 'http' in rune completions");
        assert!(items.iter().any(|item| item.label == "csv"), "expected 'csv' in rune completions");
        assert!(items.iter().any(|item| item.label == "json"), "expected 'json' in rune completions");
    }

    #[test]
    fn rune_completions_includes_all_known_runes() {
        let items = rune_completions();
        assert!(items.iter().any(|item| item.label == "http"));
        assert!(items.iter().any(|item| item.label == "aws"));
        assert!(items.iter().any(|item| item.label == "llm"));
    }
}
