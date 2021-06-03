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
/// use yaml_peg::node;
/// let n = node!(null);
/// assert_eq!(n["a"][0]["bc"], n);
/// ```
///
/// There are `as_*` methods provide `Option` returns,
/// default options can be created by [`Option::unwrap_or`],
/// and the error [`Result`] can be return by [`Option::ok_or`] to indicate the position,
/// which shown as following example:
///
/// ```
/// use yaml_peg::node;
///
/// fn main() -> Result<(), (&'static str, u64)> {
///     let n = node!({
///         node!("title") => node!(12.)
///     });
///     let n = n.as_get(&["title"]).ok_or(("missing \"title\"", n.pos))?;
///     assert_eq!(
///         Err(("title", 0)),
///         n.as_str().ok_or(("title", n.pos))
///     );
///     Ok(())
/// }
/// ```
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

    /// Builder function for anchor annotation.
    pub fn anchor(mut self, anchor: String) -> Self {
        self.anchor = anchor;
        self
    }

    /// Check the value is null.
    pub fn is_null(&self) -> bool {
        self.yaml == Yaml::Null
    }

    /// Convert to boolean.
    ///
    /// ```
    /// use yaml_peg::{node};
    /// assert!(node!(true).as_bool().unwrap());
    /// ```
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
    /// You can check them by [`str::is_empty`].
    ///
    /// ```
    /// use yaml_peg::node;
    /// assert_eq!("abc", node!("abc").as_str().unwrap());
    /// assert!(node!(null).as_str().unwrap().is_empty());
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match &self.yaml {
            Yaml::Str(s) => Some(s),
            Yaml::Null => Some(""),
            _ => None,
        }
    }

    /// Convert to the string pointer of an anchor.
    ///
    /// ```
    /// use yaml_peg::node;
    /// assert_eq!("abc", node!(*("abc")).as_anchor().unwrap());
    /// ```
    pub fn as_anchor(&self) -> Option<&str> {
        match &self.yaml {
            Yaml::Anchor(s) => Some(s),
            _ => None,
        }
    }

    /// Convert to array.
    ///
    /// WARNING: The object ownership will be took.
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

    /// Convert to map and try to get the value by keys recursivly.
    ///
    /// If get failed, returns [`Option::None`].
    ///
    /// ```
    /// use yaml_peg::node;
    /// assert_eq!(
    ///     node!({node!("a") => node!({node!("b") => node!(30.)})}).as_get(&["a", "b"]).unwrap(),
    ///     &node!(30.)
    /// );
    /// ```
    pub fn as_get<Y>(&self, keys: &[Y]) -> Option<&Self>
    where
        Y: Into<Yaml> + Copy,
    {
        if let Yaml::Map(a) = &self.yaml {
            get_from_map(a, keys)
        } else {
            None
        }
    }
}

fn get_from_map<'a, Y>(m: &'a Map, keys: &[Y]) -> Option<&'a Node>
where
    Y: Into<Yaml> + Copy,
{
    if keys.is_empty() {
        panic!("invalid search!");
    }
    if let Some(n) = m.get(&node!(keys[0])) {
        match &n.yaml {
            Yaml::Map(m) => {
                if keys[1..].is_empty() {
                    Some(n)
                } else {
                    get_from_map(m, &keys[1..])
                }
            }
            _ => Some(n),
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
            m.get(&node!(index)).unwrap_or(self)
        } else {
            self
        }
    }
}
