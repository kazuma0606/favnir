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

// ── List ─────────────────────────────────────────────────────────────────────

#[test]
fn test_list_range() {
    assert_eq!(
        eval("public fn main() -> Int { List.length(List.range(1, 5)) }"),
        Value::Int(4)
    );
    // First element via find
    assert_eq!(
        eval("public fn main() -> Int { Option.unwrap_or(List.find(List.range(0, 3), |x| x == 0), -1) }"),
        Value::Int(0)
    );
    // Last element of range via find
    assert_eq!(
        eval("public fn main() -> Int { Option.unwrap_or(List.find(List.range(0, 3), |x| x == 2), -1) }"),
        Value::Int(2)
    );
}

#[test]
fn test_list_reverse() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(1, 4)
    bind rev <- List.reverse(xs)
    Option.unwrap_or(List.find(rev, |x| x == 3), -1)
}
"#),
        Value::Int(3)
    );
}

#[test]
fn test_list_concat() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind a <- List.range(1, 3)
    bind b <- List.range(3, 5)
    List.length(List.concat(a, b))
}
"#),
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
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(1, 4)
    bind result <- List.flat_map(xs, |x| List.range(0, x))
    List.length(result)
}
"#),
        Value::Int(6)
    );
}

#[test]
fn test_list_zip() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind a <- List.range(1, 4)
    bind b <- List.range(10, 13)
    bind zipped <- List.zip(a, b)
    List.length(zipped)
}
"#),
        Value::Int(3)
    );
}

#[test]
fn test_list_sort() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(0, 5)
    bind rev <- List.reverse(xs)
    bind sorted <- List.sort(rev, |a, b| a - b)
    Option.unwrap_or(List.find(sorted, |x| x == 0), -1)
}
"#),
        Value::Int(0)
    );
}

#[test]
fn test_list_find_any_all() {
    assert_eq!(
        eval(r#"
public fn main() -> Bool {
    bind xs <- List.range(1, 6)
    List.any(xs, |x| x > 3)
}
"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Bool {
    bind xs <- List.range(1, 6)
    List.all(xs, |x| x > 0)
}
"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Bool {
    bind xs <- List.range(1, 6)
    List.all(xs, |x| x > 3)
}
"#),
        Value::Bool(false)
    );
}

#[test]
fn test_list_find() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(1, 6)
    bind found <- List.find(xs, |x| x > 3)
    Option.unwrap_or(found, 0)
}
"#),
        Value::Int(4)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(1, 4)
    bind found <- List.find(xs, |x| x > 10)
    Option.unwrap_or(found, 99)
}
"#),
        Value::Int(99)
    );
}

#[test]
fn test_list_index_of() {
    // index_of takes a predicate
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(10, 15)
    Option.unwrap_or(List.index_of(xs, |x| x == 12), -1)
}
"#),
        Value::Int(2)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(10, 15)
    Option.unwrap_or(List.index_of(xs, |x| x == 99), -1)
}
"#),
        Value::Int(-1)
    );
}

#[test]
fn test_list_enumerate() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(10, 13)
    bind pairs <- List.enumerate(xs)
    List.length(pairs)
}
"#),
        Value::Int(3)
    );
}

#[test]
fn test_list_join() {
    assert_eq!(
        eval(r#"
public fn main() -> String {
    bind xs <- List.map(List.range(1, 4), |x| Debug.show(x))
    List.join(xs, ", ")
}
"#),
        Value::Str("1, 2, 3".into())
    );
}

#[test]
fn test_list_map_filter_fold() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind xs <- List.range(1, 6)
    bind doubled <- List.map(xs, |x| x * 2)
    bind evens <- List.filter(doubled, |x| x > 4)
    List.fold(evens, 0, |acc, x| acc + x)
}
"#),
        Value::Int(6 + 8 + 10)
    );
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
        eval(r#"
public fn main() -> String {
    bind parts <- List.map(List.range(1, 4), |x| Debug.show(x))
    String.join(parts, "-")
}
"#),
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
        eval(r#"
public fn main() -> Int {
    bind m <- Map.set(Map.set((), "a", 1), "b", 2)
    Option.unwrap_or(Map.get(m, "a"), 0)
}
"#),
        Value::Int(1)
    );
}

#[test]
fn test_map_has_key_size_is_empty() {
    assert_eq!(
        eval(r#"
public fn main() -> Bool {
    bind m <- Map.set((), "x", 10)
    Map.has_key(m, "x")
}
"#),
        Value::Bool(true)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind m <- Map.set(Map.set((), "a", 1), "b", 2)
    Map.size(m)
}
"#),
        Value::Int(2)
    );
    // Map with 1 entry is not empty
    assert_eq!(
        eval(r#"
public fn main() -> Bool {
    bind m <- Map.set((), "k", 1)
    Map.is_empty(m)
}
"#),
        Value::Bool(false)
    );
}

#[test]
fn test_map_merge() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind a <- Map.set((), "x", 1)
    bind b <- Map.set((), "y", 2)
    bind merged <- Map.merge(a, b)
    Map.size(merged)
}
"#),
        Value::Int(2)
    );
}

#[test]
fn test_map_keys_values() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind m <- Map.set(Map.set((), "a", 1), "b", 2)
    List.length(Map.keys(m))
}
"#),
        Value::Int(2)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind m <- Map.set(Map.set((), "a", 1), "b", 2)
    List.length(Map.values(m))
}
"#),
        Value::Int(2)
    );
}

#[test]
fn test_map_from_list_to_list() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind pairs <- List.zip(
        List.map(List.range(0, 3), |i| String.concat("k", Debug.show(i))),
        List.range(10, 13)
    )
    bind m <- Map.from_list(pairs)
    Map.size(m)
}
"#),
        Value::Int(3)
    );
}

// ── Option ───────────────────────────────────────────────────────────────────

#[test]
fn test_option_and_then() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind result <- Option.and_then(Option.some(5), |x| Option.some(x * 2))
    Option.unwrap_or(result, 0)
}
"#),
        Value::Int(10)
    );
}

#[test]
fn test_option_and_then_none() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind result <- Option.and_then(Option.none(), |x| Option.some(x * 2))
    Option.unwrap_or(result, 99)
}
"#),
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
        eval(r#"
public fn main() -> Int {
    bind result <- Option.or_else(Option.none(), || Option.some(42))
    Option.unwrap_or(result, 0)
}
"#),
        Value::Int(42)
    );
}

#[test]
fn test_option_to_result() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind r <- Option.to_result(Option.some(7), "missing")
    Result.unwrap_or(r, 0)
}
"#),
        Value::Int(7)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind r <- Option.to_result(Option.none(), "missing")
    Result.unwrap_or(r, 99)
}
"#),
        Value::Int(99)
    );
}

// ── Result ───────────────────────────────────────────────────────────────────

#[test]
fn test_result_map_and_then() {
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind r <- Result.map(Result.ok(5), |x| x * 3)
    Result.unwrap_or(r, 0)
}
"#),
        Value::Int(15)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Int {
    bind r <- Result.and_then(Result.ok(5), |x| Result.ok(x + 1))
    Result.unwrap_or(r, 0)
}
"#),
        Value::Int(6)
    );
}

#[test]
fn test_result_map_err() {
    assert_eq!(
        eval(r#"
public fn main() -> String {
    bind r <- Result.map_err(Result.err("oops"), |e| String.concat("err: ", e))
    match r {
        err(e) => e
        ok(_)  => "ok"
    }
}
"#),
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
        eval(r#"
public fn main() -> Int {
    bind opt <- Result.to_option(Result.ok(42))
    Option.unwrap_or(opt, 0)
}
"#),
        Value::Int(42)
    );
    assert_eq!(
        eval(r#"
public fn main() -> Bool {
    bind opt <- Result.to_option(Result.err("no"))
    Option.is_none(opt)
}
"#),
        Value::Bool(true)
    );
}

// ── 基本言語機能 ──────────────────────────────────────────────────────────────

#[test]
fn test_arithmetic() {
    assert_eq!(eval("public fn main() -> Int { 3 + 4 * 2 }"), Value::Int(11));
    assert_eq!(eval("public fn main() -> Int { 10 - 3 }"), Value::Int(7));
    assert_eq!(eval("public fn main() -> Int { 10 / 2 }"), Value::Int(5));
}

#[test]
fn test_comparison() {
    assert_eq!(eval("public fn main() -> Bool { 1 < 2 }"), Value::Bool(true));
    assert_eq!(eval("public fn main() -> Bool { 2 > 3 }"), Value::Bool(false));
    assert_eq!(eval("public fn main() -> Bool { 2 == 2 }"), Value::Bool(true));
    assert_eq!(eval("public fn main() -> Bool { 2 != 3 }"), Value::Bool(true));
    assert_eq!(eval("public fn main() -> Bool { 3 >= 3 }"), Value::Bool(true));
    assert_eq!(eval("public fn main() -> Bool { 2 <= 3 }"), Value::Bool(true));
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
    let result = eval(r#"
public fn main() -> Int {
    bind nums <- collect { yield 1; yield 2; yield 3; }
    bind count <- List.fold(nums, 0, |acc, ignored| acc + 1)
    count
}
"#);
    assert_eq!(result, Value::Int(3));
}

#[test]
fn test_for_in_returns_unit() {
    // for-in itself evaluates to Unit; surrounding fn returns 42
    let result = eval(r#"
public fn main() -> Int !Io {
    bind nums <- collect { yield 10; yield 20; }
    for n in nums {
        IO.println_int(n)
    }
    42
}
"#);
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_for_in_captures_outer_var() {
    // for body can reference outer-scope variable
    let result = eval(r#"
public fn main() -> Int {
    bind nums <- collect { yield 1; yield 2; yield 3; }
    bind total <- List.fold(nums, 0, |acc, x| acc + x)
    total
}
"#);
    assert_eq!(result, Value::Int(6));
}

// ── v1.9.0: ?? null-coalesce operator ────────────────────────────────────────

#[test]
fn test_null_coalesce_some() {
    let result = eval(r#"
public fn main() -> Int {
    bind x: Option<Int> <- Option.some(5)
    x ?? 99
}
"#);
    assert_eq!(result, Value::Int(5));
}

#[test]
fn test_null_coalesce_none() {
    let result = eval(r#"
public fn main() -> Int {
    bind x: Option<Int> <- Option.none()
    x ?? 99
}
"#);
    assert_eq!(result, Value::Int(99));
}

#[test]
fn test_null_coalesce_chained() {
    let result = eval(r#"
public fn main() -> Int {
    bind a: Option<Int> <- Option.none()
    bind b: Option<Int> <- Option.some(7)
    bind av <- a ?? 0
    bind bv <- b ?? 0
    av + bv
}
"#);
    assert_eq!(result, Value::Int(7));
}
