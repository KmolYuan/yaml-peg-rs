use super::*;

const TEST_JSON: &str = r#"
---
{
    "a": "b",
    "c": [123, 321, 1234567]
}
"#;
const TEST_YAML_CONST: &str = "&a !!float -12.3";
const TEST_YAML_FLOW: &str = r#"
---
a0 bb: val
a1: &x
  b1: 4.
  b2: d
a2: !!t1 4.03
a3: {1: 2, 3: 4}
a4: !t2
  - [a1, 中文]
  - 2
a5: *x
"#;

#[test]
fn test_json() {
    let ans = match parse(TEST_JSON) {
        Ok(n) => n,
        Err(e) => {
            println!("{}", e);
            panic!()
        }
    };
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
fn test_yaml_const() {
    let ans = match parse(TEST_YAML_CONST) {
        Ok(n) => n,
        Err(e) => {
            println!("{}", e);
            panic!()
        }
    };
    assert_eq!(ans[0], node!(-12.3));
}

#[test]
fn test_yaml_flow() {
    let ans = match parse(TEST_YAML_FLOW) {
        Ok(n) => n,
        Err(e) => {
            println!("{}", e);
            panic!()
        }
    };
    assert_eq!(
        ans[0],
        node!(yaml_map![node!("a") => node!("b c"), node!("def") => node!(123)])
    );
}
