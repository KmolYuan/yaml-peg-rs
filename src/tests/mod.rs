use super::*;

const TEST_JSON: &str = include_str!("test.json");
const TEST_YAML: &str = include_str!("test.yaml");

#[test]
fn test_json() {
    let (ans, anchors) = parse(TEST_JSON).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(anchors.len(), 0);
    assert_eq!(anchor_visit(&ans[0]), anchors);
    assert_eq!(
        ans[0],
        node!({
            "a" => "b",
            "c" => node!([123, 321, 1234567]),
            "d" => node!({})
        })
    );
    let n = ans[0].get("a").unwrap();
    assert_eq!(n, &node!("b"));
}

#[test]
fn test_yaml() {
    let (ans, anchors) = parse(TEST_YAML).unwrap_or_else(|e| panic!("{}", e));
    assert_eq!(anchors.len(), 2);
    assert!(anchors.contains_key("x"));
    assert!(anchors.contains_key("y"));
    assert_eq!(anchor_visit(&ans[0]), anchors);
    assert_eq!(
        ans[0],
        node!({
            "a0 bb" => ".val.",
            "::a1" => node!({
                "b1" => 4.,
                "b2" => node!([
                    "50%",
                    node!(YamlBase::Float("2e-4".to_string())),
                    node!(YamlBase::Float("NaN".to_string())),
                    node!(YamlBase::Float("-inf".to_string())),
                    "-.infs",
                    "2001-11-23 15:01:42 -5",
                ]),
            }),
            "-a2" => 4.03,
            node!(["q", "r", "s"]) => node!({1 => 2, 3 => 4}),
            "?a3" => node!([
                node!(*"x"),
                node!(["d1🀄🃏", "中文"]),
                (),
                (),
            ]),
            node!({"a4" => ()}) => -30,
            node!(*"y") => "b3, b4",
            "test multiline" => node!([
                node!({
                    "folded" => "aaa{}[] bbb ccc\nddd\n# eee\n",
                    "literal" => "aaa{}[]\nbbb\n  ccc\n\n  ddd\n\n# eee\n",
                }),
                node!({
                    "plain" => "aaa{}[] \"bbb\" 'ccc', ddd\\n\neee fff",
                    "single quoted" => "aaa{}[] \"bbb\" 'ccc', ddd\\n\neee fff\n# ggg",
                    "double quoted" => "aaa{}[] \"bbb\" 'ccc', ddd\\n\neee fff\n# ggg",
                }),
                "literal\n\n",
                "literal",
            ]),
        })
    );
    let k = node!("a0 bb");
    assert_eq!(ans[0][k].tag(), "tag:test.x.prefix:foo");
    let k = node!("-a2");
    assert_eq!(ans[0][k].tag(), "tag:test.prefix:t1");
    let k = node!("?a3");
    assert_eq!(ans[0][k].tag(), "tag:my.tag.prefix:tt");
}

#[test]
fn test_dump() {
    use crate::dumper::NL;
    let doc = dump(&[
        node!({
            "a" => "b",
            "c" => node!([
                node!({"d" => "e"}),
                node!({"f" => node!({"g" => "h"})}),
                node!({"i" => node!(["j"])}),
            ]),
        }),
        node!(["a", "b"]),
    ]);
    let ans = "\
a: b
c:
  - d: e
  - f:
      g: h
  - i:
    - j
---
- a
- b
";
    assert_eq!(doc, ans.replace('\n', NL));
}
