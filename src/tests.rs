use super::*;

const TEST_JSON: &str = r#"
{
    "a": "b",
    "c": [123, 321, 1234567],
    "d": {}
}
"#;
const TEST_YAML: &str = r#"
---
a0 bb: val
::a1: &x  # Test comment after wrapped! (1)
  b1: 4.
  b2:
    - c1
-a2: !!t1 4.03  # Test comment after normal scalars~
?  # Test comment after wrapped! (2)
  - q
  - r  # Test comment after plain string...
  - s
: {1: 2, 3: 4}  # Test comment after wrapped! (3)
?a3: !t2
  - [d1, 中文]
  - ~
a4: *x
"#;

#[test]
fn test_json() {
    let ans = parse(TEST_JSON).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(
        ans[0],
        node!({
            node!("a") => node!("b"),
            node!("c") => node!([node!(123), node!(321), node!(1234567)]),
            node!("d") => node!({})
        })
    );
    let n = ans[0].assert_get(&["a"], "").unwrap();
    assert_eq!(n, &node!("b"));
}

#[test]
fn test_yaml() {
    let ans = parse(TEST_YAML).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(
        ans[0],
        node!({
            node!("a0 bb") => node!("val"),
            node!("::a1") => node!({
                node!("b1") => node!(4.),
                node!("b2") => node!([node!("c1")]),
            }),
            node!("-a2") => node!(4.03),
            node!([node!("q"), node!("r"), node!("s")]) => node!({
                node!(1) => node!(2),
                node!(3) => node!(4),
            }),
            node!("?a3") => node!([node!([node!("d1"), node!("中文")]), node!(Yaml::Null)]),
            node!("a4") => node!(Yaml::Anchor("x".into())),
        })
    );
}
