use std::str::FromStr;
use super::Record;

#[test]
fn test1() {
    assert!(Record::from_str("x").is_err());
}

#[test]
fn test2() {
    assert!(!Record::from_str("{}").is_err());
}

#[test]
fn test_serde() {
    let s = "{\"x\":[{\"y\":\"z\"}]}";
    let r = Record::from_str(s).unwrap();
    assert_eq!(r.to_string(), s);
}

#[test]
fn test_get_path() {
    let r = Record::from_str("{\"x\":[{\"y\":\"z\"}]}").unwrap();
    assert_eq!(r.get_path("x").to_string(), "[{\"y\":\"z\"}]");
    assert_eq!(r.get_path("y").to_string(), "null");
    assert_eq!(r.get_path("y/z").to_string(), "null");
    assert_eq!(r.get_path("x/#0").to_string(), "{\"y\":\"z\"}");
    assert_eq!(r.get_path("x/#1").to_string(), "null");
    assert_eq!(r.get_path("x/#0/y").to_string(), "\"z\"");
}

#[test]
fn test_set_path() {
    let mut r = Record::from_str("{\"x\":[{\"y\":\"z\"}]}").unwrap();
    let r2 = r.clone();
    r.set_path("x/#0/y", Record::from_primitive_string("w"));
    assert_eq!(r.to_string(), "{\"x\":[{\"y\":\"w\"}]}");
    assert_eq!(r2.to_string(), "{\"x\":[{\"y\":\"z\"}]}");
    r.set_path("a/#2/b", Record::from_primitive_string("c"));
    assert_eq!(r.to_string(), "{\"a\":[null,null,{\"b\":\"c\"}],\"x\":[{\"y\":\"w\"}]}");
}
