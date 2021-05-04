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
