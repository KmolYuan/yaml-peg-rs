use linked_hash_map::LinkedHashMap;
use std::{
    hash::{Hash, Hasher},
    io::Result,
    slice::Iter,
    str::FromStr,
};

pub type Array = Vec<Node>;
pub type Map = LinkedHashMap<Node, Node>;

/// Yaml data types.
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Yaml {
    /// Null
    Null,
    /// Boolean
    Bool(bool),
    /// Integer
    Int(String),
    /// Float
    Float(String),
    /// String
    Str(String),
    /// Array
    Array(Array),
    /// Map
    Map(Map),
    /// Anchor insertion
    Anchor(String),
}

impl Yaml {
    pub fn int<T>(s: T) -> Self
    where
        T: Into<String>,
    {
        Self::Int(s.into())
    }

    pub fn float<T>(s: T) -> Self
    where
        T: Into<String>,
    {
        Self::Float(s.into())
    }

    pub fn string<T>(s: T) -> Self
    where
        T: Into<String>,
    {
        Yaml::Str(s.into())
    }
}

impl From<&str> for Yaml {
    fn from(s: &str) -> Self {
        Yaml::Str(s.into())
    }
}

/// Parser node, includes some information.
///
/// This type will ignore additional members when comparison and hashing.
#[derive(Eq, Debug, Clone)]
pub struct Node {
    /// Document position
    pub pos: usize,
    /// Type assertion
    pub ty: String,
    /// Anchor reference
    pub anchor: String,
    /// Yaml data
    pub yaml: Yaml,
}

impl Node {
    /// Create node from yaml data.
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

    /// Assert the data is boolean.
    pub fn assert_bool<E: Into<String>>(&self, e: E) -> Result<bool> {
        match &self.yaml {
            Yaml::Bool(b) => Ok(*b),
            _ => Err(err!(e.into())),
        }
    }

    /// Assert the data is integer.
    pub fn assert_int<E: Into<String>, N: FromStr>(&self, e: E) -> Result<N> {
        match match &self.yaml {
            Yaml::Int(n) => n,
            _ => "",
        }
        .parse()
        {
            Ok(v) => Ok(v),
            Err(_) => Err(err!(e.into())),
        }
    }

    /// Assert the data is float.
    pub fn assert_float<E: Into<String>, N: FromStr>(&self, e: E) -> Result<N> {
        match match &self.yaml {
            Yaml::Float(n) => n,
            _ => "",
        }
        .parse()
        {
            Ok(v) => Ok(v),
            Err(_) => Err(err!(e.into())),
        }
    }

    /// Assert the data is string reference.
    ///
    /// Null value will generate an empty string.
    /// Warn: The object ownership will be took.
    pub fn assert_str<E>(&self, e: E) -> Result<&str>
    where
        E: Into<String>,
    {
        match &self.yaml {
            Yaml::Str(s) => Ok(s.as_ref()),
            Yaml::Null => Ok(""),
            _ => Err(err!(e.into())),
        }
    }

    /// Assert the data is string.
    ///
    /// Null value will generate an empty string.
    pub fn assert_string<E>(&self, e: E) -> Result<String>
    where
        E: Into<String>,
    {
        match &self.yaml {
            Yaml::Str(s) => Ok(s.clone()),
            Yaml::Null => Ok("".into()),
            _ => Err(err!(e.into())),
        }
    }

    /// Assert the data is array.
    ///
    /// Null value will generate an empty array.
    pub fn assert_array<E>(&self, e: E) -> Result<(usize, Iter<Node>)>
    where
        E: Into<String>,
    {
        match &self.yaml {
            Yaml::Array(a) => Ok((a.len(), a.iter())),
            Yaml::Null => Ok((0, [].iter())),
            _ => Err(err!(e.into())),
        }
    }

    /// Assert the data is map and try to get the value by keys.
    pub fn assert_get<E>(&self, keys: &[&str], e: E) -> Result<&Node>
    where
        E: Into<String>,
    {
        if let Yaml::Map(m) = &self.yaml {
            get_from_map(m, keys, e)
        } else {
            Err(err!(e.into()))
        }
    }
}

fn get_from_map<'a, E>(m: &'a Map, keys: &[&str], e: E) -> Result<&'a Node>
where
    E: Into<String>,
{
    if keys.is_empty() {
        panic!("invalid search!");
    }
    let key = Node::from(keys[0]);
    match m.get(&key) {
        Some(v) => match &v.yaml {
            Yaml::Map(m) => {
                if keys[1..].is_empty() {
                    Ok(v)
                } else {
                    get_from_map(m, &keys[1..], e)
                }
            }
            _ => Ok(v),
        },
        None => Err(err!(e.into())),
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
