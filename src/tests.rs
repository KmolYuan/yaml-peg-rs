use super::*;

#[test]
fn test_json() {
    let ans = parse_yaml(r#"{"a": "b", "c": 123}"#).unwrap();
    assert_eq!(
        ans[0],
        Node::new(map![
            node!("a", 1) => node!("b", 6),
            node!("c", 11) => node!(Yaml::int(123), 16),
        ])
    );
    let n = ans[0].assert_get(&["a"], "").unwrap();
    assert_eq!(n, &node!("b"));
}

#[test]
fn test_yaml() {
    let ans = parse_yaml(r#"{a: &a !!t b c, def: 123}"#).unwrap();
    assert_eq!(
        ans[0],
        Node::new(map![
            node!("a", 1) => node!("b c", 11, "a", "t"),
            node!("def", 16) => node!(Yaml::int(123), 21),
        ])
    );
}
