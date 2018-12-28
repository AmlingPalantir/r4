use record::Record;

fn test_one(input: &str, c: &str, eret: &str, er: &str) {
    let r = Record::parse(input);
    let (oret, r) = super::load(c)(r);
    assert_eq!(r.deparse(), er);
    assert_eq!(oret.deparse(), eret);
}

#[test]
fn test_simple() {
    test_one("{}", r#"{{x}} = "y""#, r#""y""#, r#"{"x":"y"}"#);
}

#[test]
fn test_reassign() {
    test_one(r#"{"a":[1,2]}"#, r#"{{b}} = {{a}}"#, "[1,2]", r#"{"a":[1,2],"b":[1,2]}"#);
}

#[test]
fn test_assign_hash() {
    test_one("{}", r#"{{x}} = {a: "b"}"#, r#"{"a":"b"}"#, r#"{"x":{"a":"b"}}"#);
}

#[test]
fn test_array_literal() {
    test_one("{}", r#"{{x}} = [1, "b"]"#, r#"[1,"b"]"#, r#"{"x":[1,"b"]}"#);
}

#[test]
fn test_deep_index() {
    test_one(r#"{"a":{"b":[{"c":"x"}]}}"#, r#"{{a}} = {{a/b/#0/c}}"#, r#""x""#, r#"{"a":"x"}"#);
}

#[test]
fn test_del() {
    test_one(r#"{"a":[{"b":"c"}]}"#, r#"{{x}} = d{{a/#0/b}}"#, r#""c""#, r#"{"a":[{}],"x":"c"}"#);
}

#[test]
fn test_diamond() {
    test_one("{}", r#"{{a}} = {{b}} = {}; {{a/c}} = "d""#, r#""d""#, r#"{"a":{"c":"d"},"b":{"c":"d"}}"#);
}

#[test]
fn test_get_fill() {
    test_one("{}", r#"{{x}} = {{a/b}}; {{x/y}} = "z""#, r#""z""#, r#"{"x":{"y":"z"}}"#);
    test_one("{}", r#"{{x}} = f{{a/b}}; {{x/y}} = "z""#, r#""z""#, r#"{"a":{"b":{"y":"z"}},"x":{"y":"z"}}"#);
}

#[test]
fn test_vars() {
    test_one("{}", r#" $a = {}; {{a:b}} = "c"; {{x}} = $a; {{r:y/z}} = $a; {{a:d/e}} = "f"; "#, r#""f""#, r#"{"x":{"b":"c","d":{"e":"f"}},"y":{"z":{"b":"c","d":{"e":"f"}}}}"#);
}
