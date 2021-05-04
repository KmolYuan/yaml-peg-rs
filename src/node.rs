use linked_hash_map::LinkedHashMap;

pub type Array = Vec<Node>;
pub type Map = LinkedHashMap<Node, Node>;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum Yaml {
    Null,
    Bool(bool),
    Str(String),
    Array(Array),
    Map(Map),
    Anchor(String),
}

impl From<&str> for Yaml {
    fn from(s: &str) -> Self {
        Yaml::Str(s.into())
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct Node {
    pub pos: usize,
    pub ty: String,
    pub anchor: String,
    pub yaml: Yaml,
}

impl Node {
    pub fn new(yaml: Yaml) -> Self {
        Self {
            pos: 0,
            ty: "".into(),
            anchor: "".into(),
            yaml,
        }
    }

    pub fn pos(mut self, pos: usize) -> Self {
        self.pos = pos;
        self
    }

    pub fn ty(mut self, ty: Option<String>) -> Self {
        self.ty = ty.unwrap_or("".into());
        self
    }

    pub fn anchor(mut self, anchor: Option<String>) -> Self {
        self.anchor = anchor.unwrap_or("".into());
        self
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
