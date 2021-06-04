use super::*;

const TEST_JSON: &str = include_str!("test.json");
const TEST_YAML: &str = include_str!("test.yaml");

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
    let n = ans[0].get(&["a"]).unwrap();
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
                node!("b2") => node!([
                    node!("50%"),
                    node!(Yaml::Float("2e-4".to_owned())),
                    node!(Yaml::Float("NaN".to_owned())),
                    node!(Yaml::Float("-inf".to_owned())),
                    node!("-.infs"),
                ]),
            }),
            node!("-a2") => node!(4.03),
            node!([node!("q"), node!("r"), node!("s")]) => node!({
                node!(1) => node!(2),
                node!(3) => node!(4),
            }),
            node!("?a3") => node!([
                node!(*("x")),
                node!([node!("d1🀄🃏"), node!("中文")]),
                node!(null),
                node!(null),
            ]),
            node!({node!("a4") => node!(null)}) => node!(-30),
            node!(*("y")) => node!("b3, b4"),
            node!("test multiline") => node!({
                node!("folded") => node!("aaa bbb ccc\nddd\n"),
                node!("literal") => node!("aaa\nbbb\n  ccc\n\n  ddd\n"),
                node!("plain") => node!("aaa \"bbb\" 'ccc', ddd\\n\neee fff"),
                node!("single quoted") => node!("aaa \"bbb\" 'ccc', ddd\\n\neee fff"),
                node!("double quoted") => node!("aaa \"bbb\" 'ccc', ddd\\n\neee fff"),
            }),
        })
    );
}
