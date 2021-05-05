use crate::*;
use std::{
    hash::{Hash, Hasher},
    io::Result,
    ops::Index,
    slice::Iter,
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

macro_rules! assert_method {
    {$(#[$meta:meta])* fn $id:ident = $ty1:ident $(| $ty2:ident)*} => {
        $(#[$meta])*
        pub fn $id<E, N>(&self, e: E) -> Result<N>
        where
            E: AsRef<str>,
            N: FromStr,
        {
            match match &self.yaml {
                Yaml::$ty1(n) $(| Yaml::$ty2(n))* => n,
                _ => "",
            }
            .parse()
            {
                Ok(v) => Ok(v),
                Err(_) => Err(err!(e.as_ref())),
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
/// use yaml_pom::Node;
/// let mut s = HashSet::new();
/// s.insert(Node::new("a".into()).pos(0));
/// s.insert(Node::new("a".into()).pos(1));
/// s.insert(Node::new("a".into()).pos(2));
/// assert_eq!(s.len(), 1);
/// ```
///
/// Nodes can be indexing by `usize` or `&str`,
/// but it will always return self if the index is not contained.
///
/// ```
/// use yaml_pom::{Yaml, Node};
/// let node = Node::new(Yaml::Null);
/// assert_eq!(node["a"][0]["bc"], node);
/// ```
///
/// There are `as_*` methods provide `Option` returns,
/// default options can be created by [`Option::unwrap_or`].
///
/// In another hand, using `assert_*` methods to convert the YAML types with **error** returns.
/// The `assert_*` methods are support to use `null` as empty option (for user inputs).
#[derive(Eq, Debug, Clone)]
pub struct Node {
    /// Document position
    pub pos: usize,
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
    pub fn pos(mut self, pos: usize) -> Self {
        self.pos = pos;
        self
    }

    /// Builder function for type assertion.
    pub fn ty(mut self, ty: Option<String>) -> Self {
        self.ty = ty.unwrap_or("".into());
        self
    }

    /// Builder function for anchor.
    pub fn anchor(mut self, anchor: Option<String>) -> Self {
        self.anchor = anchor.unwrap_or("".into());
        self
    }

    /// Check the value is null.
    pub fn is_null(&self) -> bool {
        if let Yaml::Null = self.yaml {
            true
        } else {
            false
        }
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
        fn as_int = Int
    }
    as_method! {
        /// Convert to float.
        fn as_float = Float
    }
    as_method! {
        /// Convert to number.
        fn as_number = Int | Float
    }

    /// Convert to array.
    ///
    /// Warn: The object ownership will be took.
    pub fn as_array(&self) -> Option<&Array> {
        match &self.yaml {
            Yaml::Array(a) => Some(a),
            _ => None,
        }
    }

    /// Convert to map and try to get the value by keys.
    ///
    /// If get failed, returns [`Option::None`].
    pub fn as_get(&self, keys: &[&str]) -> Option<&Self> {
        if let Yaml::Map(a) = &self.yaml {
            get_from_map(a, keys)
        } else {
            None
        }
    }

    /// Assert the data is boolean.
    pub fn assert_bool<E>(&self, e: E) -> Result<bool>
    where
        E: AsRef<str>,
    {
        match &self.yaml {
            Yaml::Bool(b) => Ok(*b),
            _ => Err(err!(e.as_ref())),
        }
    }

    assert_method! {
        /// Assert the data is integer.
        ///
        /// If get failed, returns [`std::io::Error`].
        fn assert_int = Int
    }
    assert_method! {
        /// Assert the data is float.
        ///
        /// If get failed, returns [`std::io::Error`].
        fn assert_float = Float
    }
    assert_method! {
        /// Assert the data is float.
        ///
        /// If get failed, returns [`std::io::Error`].
        fn assert_number = Int | Float
    }

    /// Assert the data is string reference.
    ///
    /// If get failed, returns [`std::io::Error`].
    /// Null value will generate an empty string.
    /// Warn: The object ownership will be took.
    pub fn assert_str<E>(&self, e: E) -> Result<&str>
    where
        E: AsRef<str>,
    {
        match &self.yaml {
            Yaml::Str(s) => Ok(s.as_ref()),
            Yaml::Null => Ok(""),
            _ => Err(err!(e.as_ref())),
        }
    }

    /// Assert the data is string.
    ///
    /// If get failed, returns [`std::io::Error`].
    /// Null value will generate an empty string.
    pub fn assert_string<E>(&self, e: E) -> Result<String>
    where
        E: AsRef<str>,
    {
        match &self.yaml {
            Yaml::Str(s) => Ok(s.clone()),
            Yaml::Null => Ok("".into()),
            _ => Err(err!(e.as_ref())),
        }
    }

    /// Assert the data is array.
    ///
    /// If get failed, returns [`std::io::Error`].
    /// Null value will generate an empty array.
    pub fn assert_array<E>(&self, e: E) -> Result<(usize, Iter<Node>)>
    where
        E: AsRef<str>,
    {
        match &self.yaml {
            Yaml::Array(a) => Ok((a.len(), a.iter())),
            Yaml::Null => Ok((0, [].iter())),
            _ => Err(err!(e.as_ref())),
        }
    }

    /// Assert the data is map and try to get the value by keys.
    ///
    /// If get failed, returns [`std::io::Error`].
    pub fn assert_get<E>(&self, keys: &[&str], e: E) -> Result<&Self>
    where
        E: AsRef<str>,
    {
        if let Yaml::Map(m) = &self.yaml {
            get_from_map(m, keys).ok_or(err!(e.as_ref()))
        } else {
            Err(err!(e.as_ref()))
        }
    }
}

fn get_from_map<'a>(m: &'a Map, keys: &[&str]) -> Option<&'a Node> {
    if keys.is_empty() {
        panic!("invalid search!");
    }
    let key = Node::from(keys[0]);
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
            m.get(&index.into()).unwrap_or(self)
        } else {
            self
        }
    }
}

impl From<&str> for Node {
    fn from(s: &str) -> Self {
        Node::new(s.into())
    }
}

impl From<(usize, Yaml)> for Node {
    fn from((pos, yaml): (usize, Yaml)) -> Self {
        Self::new(yaml).pos(pos)
    }
}

impl From<(usize, Yaml, &str, &str)> for Node {
    fn from((pos, yaml, a, ty): (usize, Yaml, &str, &str)) -> Self {
        Self::new(yaml)
            .pos(pos)
            .anchor(Some(a.into()))
            .ty(Some(ty.into()))
    }
}
