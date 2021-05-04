pub use crate::indicator::*;
pub use crate::node::*;
pub use crate::parser::*;

macro_rules! err {
    ($e:expr) => {{
        use std::io::{Error, ErrorKind};
        Error::new(ErrorKind::InvalidData, $e)
    }};
}

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
    fn test_hash() {
        use std::collections::HashSet;
        let mut s = HashSet::new();
        s.insert(Node::new("a".into()).pos(0));
        s.insert(Node::new("a".into()).pos(1));
        s.insert(Node::new("a".into()).pos(2));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_json() {
        let ans = parse_yaml(r#"{"a": "b", "c": 123}"#).unwrap();
        assert_eq!(
            ans[0],
            Node::new(Yaml::Map(
                vec![
                    ((1, "a".into()).into(), (6, "b".into()).into()),
                    ((11, "c".into()).into(), (16, Yaml::int("123")).into()),
                ]
                .into_iter()
                .collect()
            ))
        );
        let n = ans[0].assert_get(&["a"], "").unwrap();
        assert_eq!(n, &"b".into());
    }

    #[test]
    fn test_yaml() {
        let ans = parse_yaml(r#"{a: &a !!t b c, def: 123}"#).unwrap();
        assert_eq!(
            ans[0],
            Node::new(Yaml::Map(
                vec![
                    ((1, "a".into()).into(), (11, "b c".into(), "a", "t").into()),
                    ((16, "def".into()).into(), (21, Yaml::int("123")).into()),
                ]
                .into_iter()
                .collect()
            ))
        );
    }
}
