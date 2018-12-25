use super::Record;

#[test]
fn test_serde() {
    let s = "{\"x\":[{\"y\":\"z\"}]}";
    let r = Record::parse(s);
    assert_eq!(r.deparse(), s);
}

#[test]
fn test_get_path() {
    let r = Record::parse("{\"x\":[{\"y\":\"z\"}]}");
    assert_eq!(r.get_path("x").deparse(), "[{\"y\":\"z\"}]");
    assert_eq!(r.get_path("y").deparse(), "null");
    assert_eq!(r.get_path("y/z").deparse(), "null");
    assert_eq!(r.get_path("x/#0").deparse(), "{\"y\":\"z\"}");
    assert_eq!(r.get_path("x/#1").deparse(), "null");
    assert_eq!(r.get_path("x/#0/y").deparse(), "\"z\"");
}

#[test]
fn test_set_path() {
    let mut r = Record::parse("{\"x\":[{\"y\":\"z\"}]}");
    let r2 = r.clone();
    r.set_path("x/#0/y", Record::from_str("w"));
    assert_eq!(r.deparse(), "{\"x\":[{\"y\":\"w\"}]}");
    assert_eq!(r2.deparse(), "{\"x\":[{\"y\":\"z\"}]}");
    r.set_path("a/#2/b", Record::from_str("c"));
    assert_eq!(r.deparse(), "{\"a\":[null,null,{\"b\":\"c\"}],\"x\":[{\"y\":\"w\"}]}");
}
