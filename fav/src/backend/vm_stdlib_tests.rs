// vm_stdlib_tests.rs — VM-based stdlib coverage tests (v0.7.0 parity)
// Replaces the eval.rs-based stdlib tests that were removed when eval.rs was deleted.

use super::VM;
use crate::backend::codegen::codegen_program;
use crate::frontend::parser::Parser;
use crate::middle::compiler::compile_program;
use crate::value::Value;

fn eval(src: &str) -> Value {
    let prog = Parser::parse_str(src, "test").expect("parse error");
    let ir = compile_program(&prog);
    let artifact = codegen_program(&ir);
    let main_idx = artifact.fn_idx_by_name("main").expect("main not found");
    VM::run(&artifact, main_idx, vec![]).expect("runtime error")
}

fn eval_error(src: &str) -> crate::backend::vm::VMError {
    let prog = Parser::parse_str(src, "test").expect("parse error");
    let ir = compile_program(&prog);
    let artifact = codegen_program(&ir);
    let main_idx = artifact.fn_idx_by_name("main").expect("main not found");
    VM::run(&artifact, main_idx, vec![]).expect_err("expected runtime error")
}

// ── List ─────────────────────────────────────────────────────────────────────

#[test]
fn test_list_range() {
    assert_eq!(
        eval("public fn main() -> Int { List.length(List.range(1, 5)) }"),
        Value::Int(4)
    );
    // First element via find
    assert_eq!(
        eval(
            "public fn main() -> Int { Option.unwrap_or(List.find(List.range(0, 3), |x| x == 0), -1) }"
        ),
        Value::Int(0)
    );
    // Last element of range via find
    assert_eq!(
        eval(
            "public fn main() -> Int { Option.unwrap_or(List.find(List.range(0, 3), |x| x == 2), -1) }"
        ),
        Value::Int(2)
    );
}

#[test]
fn test_list_reverse() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(1, 4)
    bind rev <- List.reverse(xs)
    Option.unwrap_or(List.find(rev, |x| x == 3), -1)
}
"#
        ),
        Value::Int(3)
    );
}

#[test]
fn test_list_concat() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind a <- List.range(1, 3)
    bind b <- List.range(3, 5)
    List.length(List.concat(a, b))
}
"#
        ),
        Value::Int(4)
    );
}

#[test]
fn test_list_take_drop() {
    assert_eq!(
        eval("public fn main() -> Int { List.length(List.take(List.range(0, 10), 3)) }"),
        Value::Int(3)
    );
    assert_eq!(
        eval("public fn main() -> Int { List.length(List.drop(List.range(0, 10), 3)) }"),
        Value::Int(7)
    );
}

#[test]
fn test_list_flat_map() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(1, 4)
    bind result <- List.flat_map(xs, |x| List.range(0, x))
    List.length(result)
}
"#
        ),
        Value::Int(6)
    );
}

#[test]
fn test_list_zip() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind a <- List.range(1, 4)
    bind b <- List.range(10, 13)
    bind zipped <- List.zip(a, b)
    List.length(zipped)
}
"#
        ),
        Value::Int(3)
    );
}

#[test]
fn test_list_sort() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(0, 5)
    bind rev <- List.reverse(xs)
    bind sorted <- List.sort(rev, |a, b| a - b)
    Option.unwrap_or(List.find(sorted, |x| x == 0), -1)
}
"#
        ),
        Value::Int(0)
    );
}

#[test]
fn test_list_find_any_all() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Bool {
    bind xs <- List.range(1, 6)
    List.any(xs, |x| x > 3)
}
"#
        ),
        Value::Bool(true)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Bool {
    bind xs <- List.range(1, 6)
    List.all(xs, |x| x > 0)
}
"#
        ),
        Value::Bool(true)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Bool {
    bind xs <- List.range(1, 6)
    List.all(xs, |x| x > 3)
}
"#
        ),
        Value::Bool(false)
    );
}

#[test]
fn test_list_find() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(1, 6)
    bind found <- List.find(xs, |x| x > 3)
    Option.unwrap_or(found, 0)
}
"#
        ),
        Value::Int(4)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(1, 4)
    bind found <- List.find(xs, |x| x > 10)
    Option.unwrap_or(found, 99)
}
"#
        ),
        Value::Int(99)
    );
}

#[test]
fn test_list_index_of() {
    // index_of takes a predicate
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(10, 15)
    Option.unwrap_or(List.index_of(xs, |x| x == 12), -1)
}
"#
        ),
        Value::Int(2)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(10, 15)
    Option.unwrap_or(List.index_of(xs, |x| x == 99), -1)
}
"#
        ),
        Value::Int(-1)
    );
}

#[test]
fn test_list_enumerate() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(10, 13)
    bind pairs <- List.enumerate(xs)
    List.length(pairs)
}
"#
        ),
        Value::Int(3)
    );
}

#[test]
fn test_list_join() {
    assert_eq!(
        eval(
            r#"
public fn main() -> String {
    bind xs <- List.map(List.range(1, 4), |x| Debug.show(x))
    List.join(xs, ", ")
}
"#
        ),
        Value::Str("1, 2, 3".into())
    );
}

#[test]
fn test_list_map_filter_fold() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.range(1, 6)
    bind doubled <- List.map(xs, |x| x * 2)
    bind evens <- List.filter(doubled, |x| x > 4)
    List.fold(evens, 0, |acc, x| acc + x)
}
"#
        ),
        Value::Int(6 + 8 + 10)
    );
}

#[test]
fn test_list_unique_flatten_chunk_sum_min_max_count() {
    assert_eq!(
        eval(
            "public fn main() -> Int { List.length(List.unique(List.concat(List.range(1, 4), List.range(1, 3)))) }"
        ),
        Value::Int(3)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind xs <- List.push(List.push(List.range(0, 0), List.range(1, 3)), List.range(3, 4))
    List.length(List.flatten(xs))
}
"#
        ),
        Value::Int(3)
    );
    assert_eq!(
        eval("public fn main() -> Int { List.length(List.chunk(List.range(0, 5), 2)) }"),
        Value::Int(3)
    );
    assert_eq!(
        eval("public fn main() -> Int { List.sum(List.range(1, 4)) }"),
        Value::Int(6)
    );
    // List.sum on empty list = 0
    assert_eq!(
        eval("public fn main() -> Int { List.sum(List.range(0, 0)) }"),
        Value::Int(0)
    );
    assert_eq!(
        eval(
            "public fn main() -> Int { Option.unwrap_or(List.min(List.push(List.range(1, 3), 3)), 0) }"
        ),
        Value::Int(1)
    );
    // List.min on empty list = None
    assert_eq!(
        eval("public fn main() -> Int { Option.unwrap_or(List.min(List.range(0, 0)), -1) }"),
        Value::Int(-1)
    );
    assert_eq!(
        eval(
            "public fn main() -> Int { Option.unwrap_or(List.max(List.push(List.range(1, 3), 3)), 0) }"
        ),
        Value::Int(3)
    );
    assert_eq!(
        eval("public fn main() -> Int { List.count(List.range(1, 5), |x| x > 2) }"),
        Value::Int(2)
    );
}

#[test]
fn test_math_and_new_string_builtins() {
    assert_eq!(
        eval("public fn main() -> Int { Math.abs(-5) }"),
        Value::Int(5)
    );
    assert_eq!(
        eval("public fn main() -> Int { Math.abs(5) }"),
        Value::Int(5)
    );
    assert_eq!(
        eval("public fn main() -> Int { Math.pow(2, 10) }"),
        Value::Int(1024)
    );
    assert_eq!(
        eval("public fn main() -> Int { Math.floor(3.7) }"),
        Value::Int(3)
    );
    assert_eq!(
        eval("public fn main() -> Int { Math.ceil(3.2) }"),
        Value::Int(4)
    );
    assert_eq!(
        eval("public fn main() -> Int { Math.round(3.5) }"),
        Value::Int(4)
    );
    // Math.sqrt
    assert_eq!(
        eval("public fn main() -> Float { Math.sqrt(4.0) }"),
        Value::Float(2.0)
    );
    // Math.clamp
    assert_eq!(
        eval("public fn main() -> Int { Math.clamp(10, 0, 5) }"),
        Value::Int(5)
    );
    assert_eq!(
        eval("public fn main() -> Int { Math.clamp(-1, 0, 5) }"),
        Value::Int(0)
    );
    assert_eq!(
        eval("public fn main() -> Int { Math.clamp(3, 0, 5) }"),
        Value::Int(3)
    );
    // Math.pi / Math.e are Float constants
    match eval("public fn main() -> Float { Math.pi }") {
        Value::Float(f) => assert!((f - std::f64::consts::PI).abs() < 1e-10),
        other => panic!("expected Float for Math.pi, got {:?}", other),
    }
    assert_eq!(
        eval(
            "public fn main() -> Int { Option.unwrap_or(String.index_of(\"hello\", \"ll\"), -1) }"
        ),
        Value::Int(2)
    );
    // String.index_of — not found returns None
    assert_eq!(
        eval(
            "public fn main() -> Int { Option.unwrap_or(String.index_of(\"hello\", \"zz\"), -1) }"
        ),
        Value::Int(-1)
    );
    assert_eq!(
        eval("public fn main() -> String { String.pad_left(\"42\", 5, \"0\") }"),
        Value::Str("00042".into())
    );
    assert_eq!(
        eval("public fn main() -> String { String.pad_right(\"hi\", 5, \".\") }"),
        Value::Str("hi...".into())
    );
    assert_eq!(
        eval("public fn main() -> String { String.reverse(\"abc\") }"),
        Value::Str("cba".into())
    );
    assert_eq!(
        eval("public fn main() -> Int { List.length(String.lines(\"a\\nb\\nc\")) }"),
        Value::Int(3)
    );
    assert_eq!(
        eval("public fn main() -> Int { List.length(String.words(\"  foo  bar  \")) }"),
        Value::Int(2)
    );
}

#[test]
fn test_logical_ops_runtime() {
    assert_eq!(
        eval("public fn main() -> Bool { true && true }"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("public fn main() -> Bool { true && false }"),
        Value::Bool(false)
    );
    assert_eq!(
        eval("public fn main() -> Bool { false && true }"),
        Value::Bool(false)
    );
    assert_eq!(
        eval("public fn main() -> Bool { false || true }"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("public fn main() -> Bool { false || false }"),
        Value::Bool(false)
    );
    // Precedence: comparison binds tighter than && / ||
    assert_eq!(
        eval("public fn main() -> Bool { 1 == 1 && 2 == 2 }"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("public fn main() -> Bool { false || 1 == 1 }"),
        Value::Bool(true)
    );
}

#[test]
fn test_io_read_line_suppressed_returns_empty() {
    use crate::backend::vm::SuppressIoGuard;
    let _guard = SuppressIoGuard::new(true);
    let result = eval("public fn main() -> String !Io { IO.read_line() }");
    assert_eq!(result, Value::Str("".into()));
}

// ── String ───────────────────────────────────────────────────────────────────

#[test]
fn test_string_concat() {
    assert_eq!(
        eval(r#"public fn main() -> String { String.concat("hello", " world") }"#),
        Value::Str("hello world".into())
    );
}

#[test]
fn test_string_join() {
    assert_eq!(
        eval(
            r#"
public fn main() -> String {
    bind parts <- List.map(List.range(1, 4), |x| Debug.show(x))
    String.join(parts, "-")
}
"#
        ),
        Value::Str("1-2-3".into())
    );
}

#[test]
fn test_string_replace() {
    assert_eq!(
        eval(r#"public fn main() -> String { String.replace("hello world", "world", "Favnir") }"#),
        Value::Str("hello Favnir".into())
    );
}

#[test]
fn test_string_predicates() {
    assert_eq!(
        eval(r#"public fn main() -> Bool { String.starts_with("hello", "he") }"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { String.ends_with("hello", "lo") }"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { String.contains("hello world", "world") }"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { String.starts_with("hello", "xx") }"#),
        Value::Bool(false)
    );
}

#[test]
fn test_string_slice() {
    assert_eq!(
        eval(r#"public fn main() -> String { String.slice("hello world", 6, 11) }"#),
        Value::Str("world".into())
    );
}

#[test]
fn test_string_repeat() {
    assert_eq!(
        eval(r#"public fn main() -> String { String.repeat("ab", 3) }"#),
        Value::Str("ababab".into())
    );
}

#[test]
fn test_string_char_at() {
    assert_eq!(
        eval(r#"public fn main() -> String { Option.unwrap_or(String.char_at("hello", 1), "?") }"#),
        Value::Str("e".into())
    );
}

#[test]
fn test_string_to_from_int() {
    // String.to_int returns Option (some/none), not Result
    assert_eq!(
        eval(r#"public fn main() -> Int { Option.unwrap_or(String.to_int("42"), 0) }"#),
        Value::Int(42)
    );
    assert_eq!(
        eval(r#"public fn main() -> String { String.from_int(42) }"#),
        Value::Str("42".into())
    );
}

#[test]
fn test_string_to_from_float() {
    assert_eq!(
        eval(r#"public fn main() -> String { String.from_float(3.14) }"#),
        Value::Str("3.14".into())
    );
}

#[test]
fn test_string_length_is_empty() {
    assert_eq!(
        eval(r#"public fn main() -> Int { String.length("hello") }"#),
        Value::Int(5)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { String.is_empty("") }"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { String.is_empty("x") }"#),
        Value::Bool(false)
    );
}

// ── Map ──────────────────────────────────────────────────────────────────────

#[test]
fn test_map_basic() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind m <- Map.set(Map.set((), "a", 1), "b", 2)
    Option.unwrap_or(Map.get(m, "a"), 0)
}
"#
        ),
        Value::Int(1)
    );
}

#[test]
fn test_map_has_key_size_is_empty() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Bool {
    bind m <- Map.set((), "x", 10)
    Map.has_key(m, "x")
}
"#
        ),
        Value::Bool(true)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind m <- Map.set(Map.set((), "a", 1), "b", 2)
    Map.size(m)
}
"#
        ),
        Value::Int(2)
    );
    // Map with 1 entry is not empty
    assert_eq!(
        eval(
            r#"
public fn main() -> Bool {
    bind m <- Map.set((), "k", 1)
    Map.is_empty(m)
}
"#
        ),
        Value::Bool(false)
    );
}

#[test]
fn test_map_merge() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind a <- Map.set((), "x", 1)
    bind b <- Map.set((), "y", 2)
    bind merged <- Map.merge(a, b)
    Map.size(merged)
}
"#
        ),
        Value::Int(2)
    );
}

#[test]
fn test_map_keys_values() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind m <- Map.set(Map.set((), "a", 1), "b", 2)
    List.length(Map.keys(m))
}
"#
        ),
        Value::Int(2)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind m <- Map.set(Map.set((), "a", 1), "b", 2)
    List.length(Map.values(m))
}
"#
        ),
        Value::Int(2)
    );
}

#[test]
fn test_map_from_list_to_list() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind pairs <- List.zip(
        List.map(List.range(0, 3), |i| String.concat("k", Debug.show(i))),
        List.range(10, 13)
    )
    bind m <- Map.from_list(pairs)
    Map.size(m)
}
"#
        ),
        Value::Int(3)
    );
}

// ── Option ───────────────────────────────────────────────────────────────────

#[test]
fn test_option_and_then() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind result <- Option.and_then(Option.some(5), |x| Option.some(x * 2))
    Option.unwrap_or(result, 0)
}
"#
        ),
        Value::Int(10)
    );
}

#[test]
fn test_option_and_then_none() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind result <- Option.and_then(Option.none(), |x| Option.some(x * 2))
    Option.unwrap_or(result, 99)
}
"#
        ),
        Value::Int(99)
    );
}

#[test]
fn test_option_is_some_is_none() {
    assert_eq!(
        eval(r#"public fn main() -> Bool { Option.is_some(Option.some(1)) }"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { Option.is_none(Option.none()) }"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { Option.is_some(Option.none()) }"#),
        Value::Bool(false)
    );
}

#[test]
fn test_option_or_else() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind result <- Option.or_else(Option.none(), || Option.some(42))
    Option.unwrap_or(result, 0)
}
"#
        ),
        Value::Int(42)
    );
}

#[test]
fn test_option_to_result() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind r <- Option.to_result(Option.some(7), "missing")
    Result.unwrap_or(r, 0)
}
"#
        ),
        Value::Int(7)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind r <- Option.to_result(Option.none(), "missing")
    Result.unwrap_or(r, 99)
}
"#
        ),
        Value::Int(99)
    );
}

// ── Result ───────────────────────────────────────────────────────────────────

#[test]
fn test_result_map_and_then() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind r <- Result.map(Result.ok(5), |x| x * 3)
    Result.unwrap_or(r, 0)
}
"#
        ),
        Value::Int(15)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind r <- Result.and_then(Result.ok(5), |x| Result.ok(x + 1))
    Result.unwrap_or(r, 0)
}
"#
        ),
        Value::Int(6)
    );
}

#[test]
fn test_result_map_err() {
    assert_eq!(
        eval(
            r#"
public fn main() -> String {
    bind r <- Result.map_err(Result.err("oops"), |e| String.concat("err: ", e))
    match r {
        err(e) => e
        ok(_)  => "ok"
    }
}
"#
        ),
        Value::Str("err: oops".into())
    );
}

#[test]
fn test_result_is_ok_is_err() {
    assert_eq!(
        eval(r#"public fn main() -> Bool { Result.is_ok(Result.ok(1)) }"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { Result.is_err(Result.err("x")) }"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"public fn main() -> Bool { Result.is_ok(Result.err("x")) }"#),
        Value::Bool(false)
    );
}

#[test]
fn test_result_to_option() {
    assert_eq!(
        eval(
            r#"
public fn main() -> Int {
    bind opt <- Result.to_option(Result.ok(42))
    Option.unwrap_or(opt, 0)
}
"#
        ),
        Value::Int(42)
    );
    assert_eq!(
        eval(
            r#"
public fn main() -> Bool {
    bind opt <- Result.to_option(Result.err("no"))
    Option.is_none(opt)
}
"#
        ),
        Value::Bool(true)
    );
}

// ── 基本言語機能 ──────────────────────────────────────────────────────────────

#[test]
fn test_arithmetic() {
    assert_eq!(
        eval("public fn main() -> Int { 3 + 4 * 2 }"),
        Value::Int(11)
    );
    assert_eq!(eval("public fn main() -> Int { 10 - 3 }"), Value::Int(7));
    assert_eq!(eval("public fn main() -> Int { 10 / 2 }"), Value::Int(5));
}

#[test]
fn test_comparison() {
    assert_eq!(
        eval("public fn main() -> Bool { 1 < 2 }"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("public fn main() -> Bool { 2 > 3 }"),
        Value::Bool(false)
    );
    assert_eq!(
        eval("public fn main() -> Bool { 2 == 2 }"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("public fn main() -> Bool { 2 != 3 }"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("public fn main() -> Bool { 3 >= 3 }"),
        Value::Bool(true)
    );
    assert_eq!(
        eval("public fn main() -> Bool { 2 <= 3 }"),
        Value::Bool(true)
    );
}

#[test]
fn test_if_else() {
    assert_eq!(
        eval("public fn main() -> Int { if true { 1 } else { 2 } }"),
        Value::Int(1)
    );
    assert_eq!(
        eval("public fn main() -> Int { if false { 1 } else { 2 } }"),
        Value::Int(2)
    );
}

#[test]
fn test_match_variant_with_payload() {
    let src = r#"
type Shape = | Circle(Int) | Square(Int)
public fn main() -> Int {
    bind s <- Circle(7)
    match s {
        Circle(r) => r * 2
        Square(w) => w * w
    }
}
"#;
    assert_eq!(eval(src), Value::Int(14));
}

#[test]
fn test_match_guard() {
    let src = r#"
public fn main() -> String {
    bind x <- 5
    match x {
        n where n > 3 => "big"
        _             => "small"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("big".into()));
}

#[test]
fn test_destructure_bind_basic() {
    let src = r#"
type Point = { x: Int y: Int }
public fn main() -> Int {
    bind pt <- Point { x: 3 y: 4 }
    bind { x, y } <- pt
    x + y
}
"#;
    assert_eq!(eval(src), Value::Int(7));
}

#[test]
fn test_destructure_bind_alias() {
    let src = r#"
type User = { name: String age: Int }
public fn main() -> Int {
    bind user <- User { name: "Mio" age: 20 }
    bind { age: user_age } <- user
    user_age
}
"#;
    assert_eq!(eval(src), Value::Int(20));
}

#[test]
fn test_destructure_bind_wildcard() {
    let src = r#"
type User = { name: String age: Int }
public fn main() -> String {
    bind user <- User { name: "Mio" age: 20 }
    bind { name, _ } <- user
    name
}
"#;
    assert_eq!(eval(src), Value::Str("Mio".into()));
}

#[test]
fn test_pipe_match_ok() {
    let src = r#"
public fn main() -> Int {
    bind result <- Result.ok(5)
    result |> match {
        Ok(v)  => v
        Err(_) => 0
    }
}
"#;
    assert_eq!(eval(src), Value::Int(5));
}

#[test]
fn test_pipe_match_err() {
    let src = r#"
public fn main() -> Int {
    bind result <- Result.err("oops")
    result |> match {
        Ok(v)  => v
        Err(_) => -1
    }
}
"#;
    assert_eq!(eval(src), Value::Int(-1));
}

#[test]
fn test_pipe_match_option_some() {
    let src = r#"
public fn main() -> Int {
    bind opt <- Option.some(42)
    opt |> match {
        Some(v) => v
        None    => 0
    }
}
"#;
    assert_eq!(eval(src), Value::Int(42));
}

#[test]
fn test_pipe_match_option_none() {
    let src = r#"
public fn main() -> Int {
    bind opt <- Option.none()
    opt |> match {
        Some(v) => v
        None    => -1
    }
}
"#;
    assert_eq!(eval(src), Value::Int(-1));
}

#[test]
fn test_return_type_inference_int() {
    let src = r#"
fn double(n: Int) = n * 2
public fn main() -> Int { double(5) }
"#;
    assert_eq!(eval(src), Value::Int(10));
}

#[test]
fn test_return_type_inference_string() {
    let src = r#"
fn greet(name: String) = $"Hello {name}!"
public fn main() -> String { greet("Mio") }
"#;
    assert_eq!(eval(src), Value::Str("Hello Mio!".into()));
}

#[test]
fn test_return_type_inference_bool() {
    let src = r#"
fn is_adult(age: Int) = age >= 18
public fn main() -> Bool { is_adult(20) }
"#;
    assert_eq!(eval(src), Value::Bool(true));
}

#[test]
fn test_runtime_error_shows_stack_trace() {
    let src = r#"
fn divide(n: Int) -> Int { n / 0 }
fn process(n: Int) -> Int { divide(n) }
public fn main() -> Int { process(10) }
"#;
    let err = eval_error(src);
    assert_eq!(err.message, "division by zero");
    assert_eq!(err.stack_trace[0].fn_name, "divide");
    assert_eq!(err.stack_trace[2].fn_name, "main");
}

#[test]
fn test_stack_trace_depth() {
    let src = r#"
fn a() -> Int { b() }
fn b() -> Int { c() }
fn c() -> Int { 1 / 0 }
public fn main() -> Int { a() }
"#;
    let err = eval_error(src);
    assert_eq!(err.stack_trace.len(), 4);
}

#[test]
fn test_pipe_match_chained() {
    let src = r#"
fn double(n: Int) -> Int {
    Result.ok(n * 2)
}

public fn main() -> Int {
    double(7) |> match {
        Ok(v)  => v
        Err(_) => 0
    }
}
"#;
    assert_eq!(eval(src), Value::Int(14));
}

#[test]
fn test_pattern_guard_fallthrough() {
    let src = r#"
public fn main() -> String {
    match 15 {
        n where n > 20 => "big"
        n where n > 10 => "medium"
        _              => "small"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("medium".into()));
}

#[test]
fn test_pattern_guard_all_fail() {
    let src = r#"
public fn main() -> String {
    match 5 {
        n where n > 20 => "big"
        n where n > 10 => "medium"
        _              => "small"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("small".into()));
}

#[test]
fn test_pattern_guard_record() {
    let src = r#"
type User = { name: String age: Int }

public fn main() -> String {
    bind u <- User { name: "Alice", age: 20 }
    match u {
        { age } where age >= 18 => "adult"
        _                       => "minor"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("adult".into()));
}

#[test]
fn test_pattern_guard_record_minor() {
    let src = r#"
type User = { name: String age: Int }

public fn main() -> String {
    bind u <- User { name: "Bob", age: 15 }
    match u {
        { age } where age >= 18 => "adult"
        _                       => "minor"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("minor".into()));
}

#[test]
fn test_pattern_guard_compound_and() {
    let src = r#"
public fn main() -> String {
    match 25 {
        n where n >= 18 && n < 65 => "working-age"
        n where n >= 65           => "senior"
        _                         => "youth"
    }
}
"#;
    assert_eq!(eval(src), Value::Str("working-age".into()));
}

#[test]
fn test_chain_ok_propagation() {
    let src = r#"
fn safe_div(a: Int, b: Int) -> Int {
    if b == 0 { Result.err("zero") } else { Result.ok(a / b) }
}
public fn main() -> Int {
    chain x <- safe_div(10, 2)
    Result.ok(x + 1)
}
"#;
    assert_eq!(
        eval(src),
        Value::Variant("ok".into(), Some(Box::new(Value::Int(6))))
    );
}

#[test]
fn test_chain_err_short_circuits() {
    let src = r#"
fn safe_div(a: Int, b: Int) -> Int {
    if b == 0 { Result.err("zero") } else { Result.ok(a / b) }
}
public fn main() -> Int {
    chain x <- safe_div(10, 0)
    Result.ok(x + 1)
}
"#;
    assert_eq!(
        eval(src),
        Value::Variant("err".into(), Some(Box::new(Value::Str("zero".into()))))
    );
}

#[test]
fn test_collect_yield() {
    let src = r#"
public fn main() -> Int {
    bind result <- collect {
        yield 1;
        yield 2;
        yield 3;
        ()
    }
    List.length(result)
}
"#;
    assert_eq!(eval(src), Value::Int(3));
}

#[test]
fn test_pipeline_trf_flw() {
    let src = r#"
stage Double: Int -> Int = |n| { n * 2 }
stage AddOne: Int -> Int = |n| { n + 1 }
seq DoubleAndAdd = Double |> AddOne
public fn main() -> Int {
    5 |> DoubleAndAdd
}
"#;
    assert_eq!(eval(src), Value::Int(11));
}

#[test]
fn test_record_construct_access() {
    let src = r#"
type Point = { x: Int y: Int }
public fn main() -> Int {
    bind p <- Point { x: 3, y: 4 }
    p.x + p.y
}
"#;
    assert_eq!(eval(src), Value::Int(7));
}

#[test]
fn test_generic_fn() {
    let src = r#"
fn identity<T>(x: T) -> T { x }
public fn main() -> Int { identity(42) }
"#;
    assert_eq!(eval(src), Value::Int(42));
}

#[test]
fn test_int_show_ord_eq() {
    // Int.show is a cap instance; use Debug.show for direct string conversion
    assert_eq!(
        eval(r#"public fn main() -> String { Debug.show(42) }"#),
        Value::Str("42".into())
    );
    // Int.eq.equals returns Bool
    assert_eq!(
        eval(r#"public fn main() -> Bool { Int.eq.equals(3, 3) }"#),
        Value::Bool(true)
    );
    // Int.ord.compare returns Int (-1, 0, 1)
    assert_eq!(
        eval(r#"public fn main() -> Int { Int.ord.compare(2, 3) }"#),
        Value::Int(-1)
    );
}

#[test]
fn test_debug_show() {
    assert_eq!(
        eval(r#"public fn main() -> String { Debug.show(42) }"#),
        Value::Str("42".into())
    );
    assert_eq!(
        eval(r#"public fn main() -> String { Debug.show(true) }"#),
        Value::Str("true".into())
    );
}

// ── v1.9.0: for-in expression ─────────────────────────────────────────────────

#[test]
fn test_for_in_runs_body() {
    // for loop is side-effecting; count iterations via List.fold
    let result = eval(
        r#"
public fn main() -> Int {
    bind nums <- collect { yield 1; yield 2; yield 3; }
    bind count <- List.fold(nums, 0, |acc, ignored| acc + 1)
    count
}
"#,
    );
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_for_in_returns_unit() {
    // for-in itself evaluates to Unit; surrounding fn returns 42
    let result = eval(
        r#"
public fn main() -> Int !Io {
    bind nums <- collect { yield 10; yield 20; }
    for n in nums {
        IO.println_int(n)
    }
    42
}
"#,
    );
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_for_in_captures_outer_var() {
    // for body can reference outer-scope variable
    let result = eval(
        r#"
public fn main() -> Int {
    bind nums <- collect { yield 1; yield 2; yield 3; }
    bind total <- List.fold(nums, 0, |acc, x| acc + x)
    total
}
"#,
    );
    assert_eq!(result, Value::Int(6));
}

// ── v1.9.0: ?? null-coalesce operator ────────────────────────────────────────

#[test]
fn test_null_coalesce_some() {
    let result = eval(
        r#"
public fn main() -> Int {
    bind x: Option<Int> <- Option.some(5)
    x ?? 99
}
"#,
    );
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_null_coalesce_none() {
    let result = eval(
        r#"
public fn main() -> Int {
    bind x: Option<Int> <- Option.none()
    x ?? 99
}
"#,
    );
    assert_eq!(result, Value::Int(99));
}

#[test]
fn test_null_coalesce_chained() {
    let result = eval(
        r#"
public fn main() -> Int {
    bind a: Option<Int> <- Option.none()
    bind b: Option<Int> <- Option.some(7)
    bind av <- a ?? 0
    bind bv <- b ?? 0
    av + bv
}
"#,
    );
    assert_eq!(result, Value::Int(7));
}
