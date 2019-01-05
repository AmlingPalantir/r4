use record::Record;

fn test_one(i: &str, c: &str, o: &str) {
    let r = Record::parse(i);
    let r = super::eval(c, r);
    assert_eq!(r.deparse(), o);
}

#[test]
fn test_simple() {
    test_one("{}", "r[\"x\"] = \"y\"", "{\"x\":\"y\"}");
}

#[test]
fn test_reassign_ud() {
    test_one("{\"a\":[1,2]}", "r[\"b\"] = r[\"a\"]", "{\"a\":[1,2],\"b\":[1,2]}");
}

#[test]
fn test_assign_hash() {
    test_one("{}", "r[\"x\"] = {a=\"b\"}", "{\"x\":{\"a\":\"b\"}}");
}

#[test]
fn test_tables_suck() {
    test_one("{}", "r[\"x\"] = {1, \"b\"}", "{\"x\":{\"1\":1,\"2\":\"b\"}}");
}

#[test]
fn test_deep_index() {
    test_one("{\"a\":{\"b\":[{\"c\":\"x\"}]}}", "r[\"a\"] = r[\"a\"][\"b\"][1][\"c\"]", "{\"a\":\"x\"}");
}
