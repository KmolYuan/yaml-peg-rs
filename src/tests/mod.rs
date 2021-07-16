use super::*;

const TEST_JSON: &str = include_str!("test.json");
const TEST_YAML: &str = include_str!("test.yaml");

#[test]
fn test_json() {
    let (ans, anchors) = parse(TEST_JSON).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(anchors.len(), 0);
    assert_eq!(
        ans[0],
        node!({
            node!("a") => node!("b"),
            node!("c") => node!([node!(123), node!(321), node!(1234567)]),
            node!("d") => node!({})
        })
    );
    let n = ans[0].get(&["a"]).unwrap();
    assert_eq!(n, &node!("b"));
}

#[test]
fn test_yaml() {
    let (ans, anchors) = parse(TEST_YAML).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(anchors.len(), 2);
    assert!(anchors.contains_key("x"));
    assert!(anchors.contains_key("y"));
    assert_eq!(
        ans[0],
        node!({
            node!("a0 bb") => node!(".val."),
            node!("::a1") => node!({
                node!("b1") => node!(4.),
                node!("b2") => node!([
                    node!("50%"),
                    node!(Yaml::Float("2e-4".to_owned())),
                    node!(Yaml::Float("NaN".to_owned())),
                    node!(Yaml::Float("-inf".to_owned())),
                    node!("-.infs"),
                    node!("2001-11-23 15:01:42 -5"),
                ]),
            }),
            node!("-a2") => node!(4.03),
            node!([node!("q"), node!("r"), node!("s")]) => node!({
                node!(1) => node!(2),
                node!(3) => node!(4),
            }),
            node!("?a3") => node!([
                node!(*"x"),
                node!([node!("d1🀄🃏"), node!("中文")]),
                node!(null),
                node!(null),
            ]),
            node!({node!("a4") => node!(null)}) => node!(-30),
            node!(*"y") => node!("b3, b4"),
            node!("test multiline") => node!([
                node!({
                    node!("folded") => node!("aaa{}[] bbb ccc\nddd\n# eee\n"),
                    node!("literal") => node!("aaa{}[]\nbbb\n  ccc\n\n  ddd\n\n# eee\n"),
                }),
                node!({
                    node!("plain") => node!("aaa{}[] \"bbb\" 'ccc', ddd\\n\neee fff"),
                    node!("single quoted") => node!("aaa{}[] \"bbb\" 'ccc', ddd\\n\neee fff\n# ggg"),
                    node!("double quoted") => node!("aaa{}[] \"bbb\" 'ccc', ddd\\n\neee fff\n# ggg"),
                }),
                node!("literal\n\n"),
                node!("literal"),
            ]),
        })
    );
}

#[test]
fn test_dump() {
    use crate::dumper::NL;
    let doc = dump(vec![
        node!({
            node!("a") => node!("b"),
            node!("c") => node!([
                node!({node!("d") => node!("e")}),
                node!({node!("f") => node!({
                    node!("g") => node!("h")
                })}),
                node!({node!("i") => node!([
                    node!("j")
                ])}),
            ]),
        }),
        node!([node!("a"), node!("b")]),
    ]);
    assert_eq!(
        doc,
        format!(
            "a: b{0}c:{0}  - d: e{0}  - f:{0}      g: h{0}  - i:{0}    - j{0}---{0}- a{0}- b{0}",
            NL
        )
    );
}
