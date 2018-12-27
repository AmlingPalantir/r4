use record::Record;

fn test_xform(i: &str, c: &str, o: &str) {
    let r = Record::parse(i);
    let r = super::load(c)(r);
    assert_eq!(r.deparse(), o);
}

#[test]
fn test_simple() {
    test_xform("{}", r#"{{x}} = "y""#, r#"{"x":"y"}"#);
}

#[test]
fn test_reassign() {
    test_xform(r#"{"a":[1,2]}"#, r#"{{b}} = {{a}}"#, r#"{"a":[1,2],"b":[1,2]}"#);
}

#[test]
fn test_assign_hash() {
    test_xform("{}", r#"{{x}} = {a: "b"}"#, r#"{"x":{"a":"b"}}"#);
}

#[test]
fn test_array_literal() {
    test_xform("{}", r#"{{x}} = [1, "b"]"#, r#"{"x":[1,"b"]}"#);
}

#[test]
fn test_deep_index() {
    test_xform(r#"{"a":{"b":[{"c":"x"}]}}"#, r#"{{a}} = {{a/b/#0/c}}"#, r#"{"a":"x"}"#);
}
