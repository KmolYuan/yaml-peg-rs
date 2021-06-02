use crate::*;
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    ops::Index,
    str::FromStr,
};

macro_rules! as_method {
    {$(#[$meta:meta])* fn $id:ident = $ty1:ident $(| $ty2:ident)*} => {
        $(#[$meta])*
        pub fn $id<N>(&self) -> Option<N>
        where
            N: FromStr,
        {
            match &self.yaml {
                Yaml::$ty1(n) $(| Yaml::$ty2(n))* => match n.parse() {
                    Ok(v) => Some(v),
                    Err(_) => None,
                },
                _ => None,
            }
        }
    };
}

/// Parser node, includes line number, column number, type assertion and anchor.
///
/// This type will ignore additional members when comparison and hashing.
///
/// ```
/// use std::collections::HashSet;
/// use yaml_peg::Node;
/// let mut s = HashSet::new();
/// s.insert(Node::new("a".into()).pos(0));
/// s.insert(Node::new("a".into()).pos(1));
/// s.insert(Node::new("a".into()).pos(2));
/// assert_eq!(s.len(), 1);
/// ```
///
/// There is a convenient macro [`node!`] to create nodes literally.
///
/// Nodes can be indexing by `usize` or `&str`,
/// but it will always return self if the index is not contained.
///
/// ```
/// use yaml_peg::{Yaml, Node};
/// let node = Node::new(Yaml::Null);
/// assert_eq!(node["a"][0]["bc"], node);
/// ```
///
/// There are `as_*` methods provide `Option` returns,
/// default options can be created by [`Option::unwrap_or`].
///
/// In another hand, using `except_*` methods to convert the YAML types with **error** returns.
/// The `except_*` methods are support to use `null` as empty option (for user inputs).
#[derive(Eq, Clone)]
pub struct Node {
    /// Document position
    pub pos: u64,
    /// Type assertion
    pub ty: String,
    /// Anchor reference
    pub anchor: String,
    /// YAML data
    pub yaml: Yaml,
}

impl Node {
    /// Create node from YAML data.
    pub fn new(yaml: Yaml) -> Self {
        Self {
            pos: 0,
            ty: "".into(),
            anchor: "".into(),
            yaml,
        }
    }

    /// Builder function for position.
    pub fn pos(mut self, pos: u64) -> Self {
        self.pos = pos;
        self
    }

    /// Builder function for type assertion.
    pub fn ty(mut self, ty: String) -> Self {
        self.ty = ty;
        self
    }

    /// Builder function for anchor.
    pub fn anchor(mut self, anchor: String) -> Self {
        self.anchor = anchor;
        self
    }

    /// Check the value is null.
    pub fn is_null(&self) -> bool {
        self.yaml == Yaml::Null
    }

    /// Convert to boolean.
    pub fn as_bool(&self) -> Option<bool> {
        match &self.yaml {
            Yaml::Bool(b) => Some(*b),
            _ => None,
        }
    }

    as_method! {
        /// Convert to integer.
        ///
        /// ```
        /// use yaml_peg::node;
        /// assert_eq!(60, node!(60).as_int().unwrap());
        /// ```
        fn as_int = Int
    }
    as_method! {
        /// Convert to float.
        ///
        /// ```
        /// use yaml_peg::node;
        /// assert_eq!(20.06, node!(20.06).as_float().unwrap());
        /// ```
        fn as_float = Float
    }
    as_method! {
        /// Convert to number.
        ///
        /// ```
        /// use yaml_peg::node;
        /// assert_eq!(60, node!(60).as_number().unwrap());
        /// assert_eq!(20.06, node!(20.06).as_number().unwrap());
        /// ```
        fn as_number = Int | Float
    }

    /// Convert to string pointer.
    ///
    /// This method allows null, it represented as empty string.
    ///
    /// ```
    /// use yaml_peg::node;
    /// assert_eq!(
    ///     "abc",
    ///     node!("abc").as_str().unwrap()
    /// );
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match &self.yaml {
            Yaml::Str(s) => Some(s),
            Yaml::Null => Some(""),
            _ => None,
        }
    }

    /// Convert to array.
    ///
    /// Warn: The object ownership will be took.
    ///
    /// ```
    /// use yaml_peg::node;
    /// assert_eq!(
    ///     &vec![node!(1), node!(2)],
    ///     node!([node!(1), node!(2)]).as_array().unwrap()
    /// );
    /// ```
    pub fn as_array(&self) -> Option<&Array> {
        match &self.yaml {
            Yaml::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Convert to map and try to get the value by keys.
    ///
    /// If get failed, returns [`Option::None`].
    ///
    /// ```
    /// use yaml_peg::node;
    /// assert_eq!(
    ///     node!({node!("a") => node!(30.)}).as_get(&["a"]).unwrap(),
    ///     &node!(30.)
    /// );
    /// ```
    pub fn as_get(&self, keys: &[&str]) -> Option<&Self> {
        if let Yaml::Map(a) = &self.yaml {
            get_from_map(a, keys)
        } else {
            None
        }
    }
}

fn get_from_map<'a>(m: &'a Map, keys: &[&str]) -> Option<&'a Node> {
    if keys.is_empty() {
        panic!("invalid search!");
    }
    let key = node!(keys[0].into());
    if let Some(v) = m.get(&key) {
        match &v.yaml {
            Yaml::Map(m) => {
                if keys[1..].is_empty() {
                    Some(v)
                } else {
                    get_from_map(m, &keys[1..])
                }
            }
            _ => Some(v),
        }
    } else {
        None
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_fmt(format_args!("Node{:?}", &self.yaml))
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.yaml.hash(state)
    }
}

impl PartialEq for Node {
    fn eq(&self, rhs: &Self) -> bool {
        self.yaml == rhs.yaml
    }
}

impl Index<usize> for Node {
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        match &self.yaml {
            Yaml::Array(a) => a.get(index).unwrap_or(self),
            Yaml::Map(m) => m
                .get(&Node::new(Yaml::Int(index.to_string())))
                .unwrap_or(self),
            _ => self,
        }
    }
}

impl Index<&str> for Node {
    type Output = Self;

    fn index(&self, index: &str) -> &Self::Output {
        if let Yaml::Map(m) = &self.yaml {
            m.get(&node!(index.into())).unwrap_or(self)
        } else {
            self
        }
    }
}
