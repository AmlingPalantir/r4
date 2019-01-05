use record::Record;

fn test_one(i: &str, c: &str, o: &str) {
    let r = Record::parse(i);
    let mut f = super::stream(c, false);
    let r = f(r);
    assert_eq!(r.deparse(), o);
}

#[test]
fn test_simple() {
    test_one(r#"{}"#, r#"r["x"] = "y""#, r#"{"x":"y"}"#);
}

#[test]
fn test_reassign_ud() {
    test_one(r#"{"a":[1,2]}"#, r#"r["b"] = r["a"]"#, r#"{"a":[1,2],"b":[1,2]}"#);
}

#[test]
fn test_assign_hash() {
    test_one(r#"{}"#, r#"r["x"] = {a="b"}"#, r#"{"x":{"a":"b"}}"#);
}

#[test]
fn test_tables_suck() {
    test_one(r#"{}"#, r#"r["x"] = {1, "b"}"#, r#"{"x":{"1":1,"2":"b"}}"#);
}

#[test]
fn test_deep_index() {
    test_one(r#"{"a":{"b":[{"c":"x"}]}}"#, r#"r["a"] = r["a"]["b"][1]["c"]"#, r#"{"a":"x"}"#);
}
