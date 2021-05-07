use super::*;

const TEST_JSON: &str = r#"
{
    "a": "b",
    "c": [123, 321, 1234567]
}
"#;
const TEST_YAML_FLOW: &str = r#"
{a: &a !!t b c, def: 123}
"#;

#[test]
fn test_json() {
    let ans = parse_yaml(TEST_JSON).unwrap();
    assert_eq!(
        ans[0],
        node!(yaml_map![
            node!("a") => node!("b"),
            node!("c") => node!(yaml_array![node!(123), node!(321), node!(1234567)]),
        ])
    );
    let n = ans[0].assert_get(&["a"], "").unwrap();
    assert_eq!(n, &node!("b"));
}

#[test]
fn test_yaml_flow() {
    let ans = parse_yaml(TEST_YAML_FLOW).unwrap();
    assert_eq!(
        ans[0],
        node!(yaml_map![node!("a") => node!("b c"), node!("def") => node!(123)])
    );
}
