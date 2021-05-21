use super::*;

const TEST_JSON: &str = r#"
---
{
    "a": "b",
    "c": [123, 321, 1234567],
    "d": {}
}
"#;
const TEST_YAML_CONST: &str = "&a !!float -12.3";
const TEST_YAML: &str = r#"
---
a0 bb: val
a1: &x
  b1: 4.
  b2:
    - c1
a2: !!t1 4.03
?
  - q
  - r
  - s
: {1: 2, 3: 4}
a3: !t2
  - [d1, 中文]
  - ~
a4: *x
"#;

#[test]
fn test_json() {
    let ans = parse(TEST_JSON).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(
        ans[0],
        node!(yaml_map![
            node!("a") => node!("b"),
            node!("c") => node!(yaml_array![node!(123), node!(321), node!(1234567)]),
            node!("d") => node!(yaml_map![])
        ])
    );
    let n = ans[0].assert_get(&["a"], "").unwrap();
    assert_eq!(n, &node!("b"));
}

#[test]
fn test_yaml_const() {
    let ans = parse(TEST_YAML_CONST).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(ans[0], node!(-12.3));
}

#[test]
fn test_yaml() {
    let ans = parse(TEST_YAML).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(
        ans[0],
        node!(yaml_map![
            node!("a0 bb") => node!("val"),
            node!("a1") => node!(yaml_map![
                node!("b1") => node!(4.),
                node!("b2") => node!(yaml_array![node!("c1")]),
            ]),
            node!("a2") => node!(4.03),
            node!(yaml_array![node!("q"), node!("r"), node!("s")]) => node!(yaml_map![
                node!(1) => node!(2),
                node!(3) => node!(4),
            ]),
            node!("a3") => node!(yaml_array![node!(yaml_array![node!("d1"), node!("中文")]), node!(Yaml::Null)]),
            node!("a4") => node!(Yaml::Anchor("x".into())),
        ])
    );
}
