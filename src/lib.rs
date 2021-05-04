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
                    ((1, "a".into()).into(), (6, "b".into()).into()),
                    ((11, "c".into()).into(), (16, "123".into()).into()),
                ]
                .into_iter()
                .collect()
            ))
        );
    }

    #[test]
    fn test_yaml() {
        let ans = parse_yaml(r#"{a: &a !!t b c, def: 123}"#).unwrap();
        assert_eq!(
            ans[0],
            Node::new(Yaml::Map(
                vec![
                    ((1, "a".into()).into(), (11, "b c".into(), "a", "t").into()),
                    ((16, "def".into()).into(), (21, "123".into()).into()),
                ]
                .into_iter()
                .collect()
            ))
        );
    }
}
