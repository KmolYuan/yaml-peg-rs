use crate::{parser::PError, *};

fn show_err<E>(e: PError) -> E {
    panic!("{}", e)
}

#[test]
fn test_json() {
    const DOC: &str = include_str!("json_compatibility.json");
    let mut root = parse(DOC).unwrap_or_else(show_err);
    let node = root.remove(0);
    assert_eq!(
        node,
        node!({
            "a" => "b",
            "c" => node!([123, 321, 1234567]),
            "d" => node!({}),
            "e:f" => "g"
        })
    );
    let n = node.get("a").unwrap();
    assert_eq!(n, &node!("b"));
}

#[test]
fn test_yaml() {
    const DOC: &str = include_str!("complete_doc.yaml");
    let mut root = parse(DOC).unwrap_or_else(show_err);
    let node = root.remove(0);
    assert_eq!(
        node,
        node!({
            "a0 bb" => ".val.",
            "::a1" => node!({
                "b1" => 4.,
                "b2" => node!([
                    "50%",
                    8,
                    16,
                    2e-4,
                    f64::NAN,
                    -f64::INFINITY,
                    "-.infs",
                    "2001-11-23 15:01:42 -5",
                ]),
            }),
            "-a2" => 4.03,
            node!(["q", "r", "s"]) => node!({1 => 2, 3 => 4}),
            "?a3" => node!([
                node!(["d1🀄🃏", "中文"]),
                (),
                (),
                (),
            ]),
            node!({"a4" => ()}) => -30,
            node!({"a4" => ()}) => "b3, b4",
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
    assert_eq!(node.tag(), "tag:test.x.prefix:root");
    let k = node!("a0 bb");
    assert_eq!(node[k].tag(), "tag:test.x.prefix:foo");
    let k = node!("-a2");
    assert_eq!(node[k].tag(), "tag:test.prefix:t1");
    let k = node!("?a3");
    assert_eq!(node[k].tag(), "tag:my.tag.prefix:tt");
}

#[test]
fn test_dump() {
    const DOC: &str = include_str!("dump_result.yaml");
    let nodes = [
        node!({
            "a" => "b",
            "c" => node!([
                node!({"d" => "e"}),
                node!({"f" => node!({"g" => "h"})}),
                node!({"i" => node!(["j"])}),
            ]),
        }),
        node!(["a", "b"]),
    ];
    let doc = dump(&nodes, &[]);
    assert_eq!(doc.replace("\r\n", "\n"), DOC.replace("\r\n", "\n"));
}

#[test]
fn test_indent() {
    const DOC: &str = include_str!("indent.yaml");
    let mut root = parse(DOC).unwrap_or_else(show_err);
    let node1 = root.remove(0);
    let node2 = root.remove(0);
    assert_eq!(
        node1,
        node!({
            "version" => 2,
            "models" => node!([node!({
                "name" => "orders",
                "columns" => node!([node!({
                    "name" => "c_custkey",
                    "tests" => node!(["not_null"]),
                })]),
            })]),
            "map" => node!(["a", "b", "c"]),
        })
    );
    assert_eq!(node2.tag(), "normal-sequence");
    assert_eq!(node2, node!(["a1", "true of", "a2"]));
}

#[test]
fn test_anchor() {
    const DOC: &str = include_str!("anchor.yaml");
    let mut root = parse::<repr::RcRepr>(DOC).unwrap_or_else(show_err);
    let node = root.remove(0);
    assert_eq!(
        node,
        node!([
            node!({"a" => "b"}),
            node!({"a" => "b"}),
            node!({"a" => "b"}),
            node!({"a" => "b"}),
            node!([node!({"a" => "b"}), node!({"a" => "b"})]),
            node!([node!({"a" => "b"}), node!({"a" => "b"})]),
        ])
    );
}
