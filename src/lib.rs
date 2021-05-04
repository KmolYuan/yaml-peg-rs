pub use crate::indicator::*;
pub use crate::node::*;
pub use crate::parser::*;

mod indicator;
mod node;
mod parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indicator() {
        let doc = indicated_msg("{\"a\": \n\"b\"}", 6);
        assert_eq!(doc, "(1:0)\n\"b\"}\n^")
    }

    #[test]
    fn test_json() {
        let ans = parse_yaml(r#"{"a": "b", "c": 123}"#).unwrap();
        assert_eq!(
            ans[0],
            Node::new(Yaml::Map(
                vec![
                    (
                        Node::new(Yaml::Str("a".into())).pos(1),
                        Node::new(Yaml::Str("b".into())).pos(6),
                    ),
                    (
                        Node::new(Yaml::Str("c".into())).pos(11),
                        Node::new(Yaml::Str("123".into())).pos(16),
                    )
                ]
                .into_iter()
                .collect()
            ))
        );
    }

    #[test]
    fn test_yaml() {
        let ans = parse_yaml(r#"{a: &a !!t b, cde: 123}"#).unwrap();
        assert_eq!(
            ans[0],
            Node::new(Yaml::Map(
                vec![
                    (
                        Node::new(Yaml::Str("a".into())).pos(1),
                        Node::new(Yaml::Str("b".into()))
                            .pos(11)
                            .anchor(Some("a".into()))
                            .ty(Some("t".into())),
                    ),
                    (
                        Node::new(Yaml::Str("cde".into())).pos(14),
                        Node::new(Yaml::Str("123".into())).pos(19),
                    )
                ]
                .into_iter()
                .collect()
            ))
        );
    }
}
