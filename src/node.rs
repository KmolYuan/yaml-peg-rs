use crate::{parser::Anchors, repr::*, *};
use alloc::string::{String, ToString};
use core::{
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    iter::FromIterator,
    marker::PhantomData,
    ops::Index,
};

macro_rules! as_method {
    {$($(#[$meta:meta])* fn $id:ident = $ty:ident$(($op:ident))?
        $(| ($default:expr)?)?
        $(| $ty2:ident)* -> $r:ty)+} => {$(
        $(#[$meta])*
        pub fn $id(&self) -> Result<$r, u64> {
            match self.yaml() {
                Yaml::$ty(v) $(| Yaml::$ty2(v))* => Ok(v$(.$op())?),
                $(Yaml::Null => Ok($default),)?
                _ => Err(self.pos),
            }
        }
    )+};
}

macro_rules! impl_iter {
    ($(impl $item:ty)+) => {
        $(impl<R: Repr> FromIterator<$item> for Node<R> {
            fn from_iter<T: IntoIterator<Item = $item>>(iter: T) -> Self {
                Self::from(iter.into_iter().collect::<Yaml<R>>())
            }
        })+
    };
}

/// A node with [`alloc::rc::Rc`] holder.
pub type NodeRc = Node<RcRepr>;
/// A node with [`alloc::sync::Arc`] holder.
pub type NodeArc = Node<ArcRepr>;

/// Readonly node, including line number, column number, type assertion and anchor.
/// You can access [`Yaml`] type through [`Node::yaml`] method.
///
/// This type will ignore additional information when comparison and hashing.
///
/// ```
/// use std::collections::HashSet;
/// use yaml_peg::{NodeRc, Yaml};
///
/// let mut s = HashSet::new();
/// s.insert(NodeRc::new(Yaml::from("a"), 0, ""));
/// s.insert(NodeRc::new("a", 1, "my-tag"));
/// s.insert(NodeRc::new("a", 2, ""));
/// assert_eq!(s.len(), 1);
/// ```
///
/// There is also a convenient macro [`node!`] to create nodes literally.
/// Please see the macro description for more information.
///
/// Nodes can be indexing by convertable values, or sequence indicator [`Ind`],
/// but it will be panic if the index is not contained.
///
/// ```
/// use yaml_peg::{node, Ind};
///
/// let n = node!(["a", "b", "c"]);
/// assert_eq!(node!("b"), n[Ind(1)]);
/// ```
///
/// ```should_panic
/// use yaml_peg::{node, Ind};
///
/// let n = node!(());
/// let n = &n["a"][Ind(0)]["b"];
/// ```
///
/// Same as containers, to prevent panic, the [`Node::get`] method is the best choice.
/// The [`Node::get_default`] can provide missing key value when indexing.
///
/// There are `as_*` methods provide `Result<T, u64>` returns with node position,
/// default options can be created by [`Result::unwrap_or`],
/// additional error message can be attach by [`Result::map_err`],
/// and the optional [`Option`] can be return by [`Result::ok`],
/// which shown as following example:
///
/// ```
/// use yaml_peg::node;
///
/// fn main() -> Result<(), (&'static str, u64)> {
///     let n = node!({
///         "title" => 12.
///     });
///     let n = n.get("title").map_err(|p| ("missing \"title\"", p))?;
///     assert_eq!(
///         Err(("title", 0)),
///         n.as_str().map_err(|p| ("title", p))
///     );
///     assert_eq!(
///         Option::<&str>::None,
///         n.as_str().ok()
///     );
///     Ok(())
/// }
/// ```
///
/// # Clone
///
/// Since the YAML data is wrapped by reference counter [`alloc::rc::Rc`]/[`alloc::sync::Arc`],
/// cloned nodes are still shared together, just like Python or JavaScript.
/// Please see the [`Node::rc_ref`]/[`Node::yaml`]/[`Node::clone_yaml`] methods for details.
///
/// If you still want to copy data, use [`From`]/[`Into`] or serialize them to your custom type.
pub struct Node<R: Repr> {
    pos: u64,
    tag: String,
    yaml: R::Rc,
    _marker: PhantomData<R>,
}

impl<R: Repr> Node<R> {
    /// Create node from YAML data.
    pub fn new(yaml: impl Into<Yaml<R>>, pos: u64, tag: impl ToString) -> Self {
        Self::new_repr(R::new_rc(yaml.into()), pos, tag)
    }

    /// Create from a representation.
    pub fn new_repr(yaml: R::Rc, pos: u64, tag: impl ToString) -> Self {
        Self {
            yaml,
            pos,
            tag: tag.to_string(),
            _marker: PhantomData,
        }
    }

    /// Set from existing YAML data.
    pub fn set_yaml(&mut self, yaml: impl Into<Yaml<R>>) {
        self.set_repr(R::new_rc(yaml.into()));
    }

    /// Set from existing YAML representation.
    pub fn set_repr(&mut self, yaml: R::Rc) {
        self.yaml = yaml;
    }

    /// Document position.
    pub fn pos(&self) -> u64 {
        self.pos
    }

    /// Tag. If the tag is not specified, returns a default tag from core schema.
    ///
    /// Anchor has no tag.
    pub fn tag(&self) -> &str {
        match self.tag.as_str() {
            "" => match self.yaml() {
                Yaml::Null => concat!(parser::tag_prefix!(), "null"),
                Yaml::Bool(_) => concat!(parser::tag_prefix!(), "bool"),
                Yaml::Int(_) => concat!(parser::tag_prefix!(), "int"),
                Yaml::Float(_) => concat!(parser::tag_prefix!(), "float"),
                Yaml::Str(_) => concat!(parser::tag_prefix!(), "str"),
                Yaml::Seq(_) => concat!(parser::tag_prefix!(), "seq"),
                Yaml::Map(_) => concat!(parser::tag_prefix!(), "map"),
                Yaml::Alias(_) => "",
            },
            s => s,
        }
    }

    /// YAML data.
    pub fn yaml(&self) -> &Yaml<R> {
        &self.yaml
    }

    /// Clone YAML repr.
    pub fn clone_yaml(&self) -> R::Rc {
        self.yaml.clone()
    }

    /// As reference for the underlying reference counter.
    ///
    /// ```
    /// use std::{rc::Rc, sync::Arc};
    /// use yaml_peg::node;
    ///
    /// let a_rc = node!("a");
    /// let a_arc = node!(arc "a");
    /// {
    ///     let b_rc = a_rc.clone();
    ///     let b_arc = a_arc.clone();
    ///     assert_eq!(2, Rc::strong_count(b_rc.rc_ref()));
    ///     assert_eq!(2, Arc::strong_count(b_arc.rc_ref()));
    /// }
    /// assert_eq!(1, Rc::strong_count(a_rc.rc_ref()));
    /// assert_eq!(1, Arc::strong_count(a_arc.rc_ref()));
    /// ```
    pub fn rc_ref(&self) -> &R::Rc {
        &self.yaml
    }

    /// Check the value is null.
    pub fn is_null(&self) -> bool {
        *self.yaml() == Yaml::Null
    }

    /// Convert to integer.
    ///
    /// ```
    /// use yaml_peg::node;
    ///
    /// assert_eq!(60, node!(60).as_int().unwrap());
    /// ```
    pub fn as_int(&self) -> Result<i64, u64> {
        match self.yaml() {
            Yaml::Int(s) => to_i64(s).map_err(|_| self.pos),
            _ => Err(self.pos),
        }
    }

    /// Convert to float.
    ///
    /// ```
    /// use yaml_peg::node;
    ///
    /// assert_eq!(20.06, node!(20.06).as_float().unwrap());
    /// ```
    pub fn as_float(&self) -> Result<f64, u64> {
        match self.yaml() {
            Yaml::Float(s) => to_f64(s).map_err(|_| self.pos),
            _ => Err(self.pos),
        }
    }

    /// Convert to float for any number.
    ///
    /// ```
    /// use yaml_peg::node;
    ///
    /// assert_eq!(60., node!(60).as_number().unwrap());
    /// assert_eq!(20.06, node!(20.06).as_number().unwrap());
    /// ```
    pub fn as_number(&self) -> Result<f64, u64> {
        match self.yaml() {
            Yaml::Int(s) => to_i64(s).map(|n| n as f64).map_err(|_| self.pos),
            Yaml::Float(s) => to_f64(s).map_err(|_| self.pos),
            _ => Err(self.pos),
        }
    }

    as_method! {
        /// Convert to boolean.
        ///
        /// ```
        /// use yaml_peg::node;
        ///
        /// assert!(node!(true).as_bool().unwrap());
        /// ```
        fn as_bool = Bool(clone) -> bool

        /// Convert to string pointer.
        ///
        /// This method allows null, it represented as empty string.
        /// You can check them by [`str::is_empty`].
        ///
        /// ```
        /// use yaml_peg::node;
        ///
        /// assert_eq!("abc", node!("abc").as_str().unwrap());
        /// assert!(node!(()).as_str().unwrap().is_empty());
        /// ```
        fn as_str = Str | ("")? -> &str

        /// Convert to sequence.
        ///
        /// ```
        /// use yaml_peg::node;
        ///
        /// let n = node!(["55"]);
        /// assert_eq!(node!("55"), n.as_seq().unwrap()[0]);
        /// for n in n.as_seq().unwrap() {
        ///     assert_eq!(node!("55"), n);
        /// }
        /// ```
        fn as_seq = Seq(clone) -> Seq<R>

        /// Convert to map.
        ///
        /// ```
        /// use yaml_peg::node;
        ///
        /// let n = node!({1 => 2});
        /// assert_eq!(node!(2), n.as_map().unwrap()[&node!(1)]);
        /// for (k, v) in n.as_map().unwrap() {
        ///     assert_eq!(node!(1), k);
        ///     assert_eq!(node!(2), v);
        /// }
        /// ```
        fn as_map = Map(clone) -> Map<R>
    }

    /// Convert to string pointer for string, null, bool, int, and float type.
    ///
    /// This method is useful when the option mixed with digit values.
    ///
    /// ```
    /// use yaml_peg::node;
    ///
    /// assert_eq!("abc", node!("abc").as_value().unwrap());
    /// assert_eq!("123", node!(123).as_value().unwrap());
    /// assert_eq!("12.04", node!(12.04).as_value().unwrap());
    /// assert_eq!("true", node!(true).as_value().unwrap());
    /// assert_eq!("false", node!(false).as_value().unwrap());
    /// assert!(node!(()).as_value().unwrap().is_empty());
    /// ```
    pub fn as_value(&self) -> Result<&str, u64> {
        match self.yaml() {
            Yaml::Str(s) | Yaml::Int(s) | Yaml::Float(s) => Ok(s),
            Yaml::Bool(true) => Ok("true"),
            Yaml::Bool(false) => Ok("false"),
            Yaml::Null => Ok(""),
            _ => Err(self.pos),
        }
    }

    /// Return the reference from `anchors` or self.
    ///
    /// ```
    /// use yaml_peg::{node, parser::Anchors};
    ///
    /// let mut anchors = Anchors::new();
    /// anchors.insert("a".to_string(), node!(20));
    /// assert_eq!(
    ///     20,
    ///     node!(*"a").as_anchor(&anchors).unwrap().as_int().unwrap()
    /// );
    /// ```
    pub fn as_anchor<'a>(&'a self, anchors: &'a Anchors<R>) -> Result<&'a Self, u64> {
        if let Yaml::Alias(a) = self.yaml() {
            anchors.get(a).ok_or(self.pos)
        } else {
            Ok(self)
        }
    }

    /// Convert to map and try to get the value by key.
    ///
    /// If any key is missing, return `Err` with node position.
    ///
    /// ```
    /// # fn main() -> Result<(), u64> {
    /// use yaml_peg::node;
    ///
    /// let n = node!({node!("a") => node!({node!("b") => node!(30.)})});
    /// assert_eq!(&node!(30.), n.get("a")?.get("b")?);
    /// # Ok::<(), u64>(()) }
    /// ```
    pub fn get<Y: Into<Self>>(&self, key: Y) -> Result<&Self, u64> {
        if let Yaml::Map(m) = self.yaml() {
            m.get(&key.into()).ok_or(self.pos)
        } else {
            Err(self.pos)
        }
    }

    /// Same as [`Node::get`] but provide default value if the key is missing.
    /// For this method, a transform method `as_*` is required.
    ///
    /// + If the value exist, return the value.
    /// + If value is a wrong type, return `Err` with node position.
    /// + If the value is not exist, return the default value.
    ///
    /// ```
    /// # fn main() -> Result<(), u64> {
    /// use yaml_peg::{node, Node};
    ///
    /// let a = node!({node!("a") => node!({node!("b") => node!("c")})});
    /// assert_eq!(
    ///     "c",
    ///     a.get("a")?.get_default("b", "d", Node::as_str)?
    /// );
    /// let b = node!({node!("a") => node!({})});
    /// assert_eq!(
    ///     "d",
    ///     b.get("a")?.get_default("b", "d", Node::as_str)?
    /// );
    /// let c = node!({node!("a") => node!({node!("b") => node!(20.)})});
    /// assert_eq!(
    ///     Err(0),
    ///     c.get("a")?.get_default("b", "d", Node::as_str)
    /// );
    /// # Ok::<(), u64>(()) }
    /// ```
    ///
    /// ```
    /// # fn main() -> Result<(), u64> {
    /// use yaml_peg::{node, Node};
    ///
    /// let n = node!({node!("a") => node!([node!(1), node!(2), node!(3)])});
    /// let a = n.get_default("c", vec![], Node::as_seq)?;
    /// assert_eq!(a, vec![]);
    /// # Ok::<(), u64>(()) }
    /// ```
    pub fn get_default<'a, Y, Ret, F>(
        &'a self,
        key: Y,
        default: Ret,
        factory: F,
    ) -> Result<Ret, u64>
    where
        Y: Into<Self>,
        F: FnOnce(&'a Self) -> Result<Ret, u64>,
    {
        if let Yaml::Map(m) = self.yaml() {
            if let Some(n) = m.get(&key.into()) {
                factory(n)
            } else {
                Ok(default)
            }
        } else {
            Err(self.pos)
        }
    }

    /// Get node through index indicator. Only suitable for sequence.
    ///
    /// ```
    /// # fn main() -> Result<(), u64> {
    /// use yaml_peg::{node, Ind};
    ///
    /// let n = node!([node!("a"), node!("b"), node!("c")]);
    /// assert_eq!(&node!("b"), n.get_ind(Ind(1))?);
    /// # Ok::<(), u64>(()) }
    /// ```
    pub fn get_ind(&self, ind: Ind) -> Result<&Self, u64> {
        if let Yaml::Seq(v) = self.yaml() {
            v.get(ind.0).ok_or(self.pos)
        } else {
            Err(self.pos)
        }
    }
}

impl<R: Repr> Debug for Node<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Node{:?}", &self.yaml)
    }
}

impl<R: Repr> Clone for Node<R> {
    fn clone(&self) -> Self {
        Self {
            tag: self.tag.clone(),
            yaml: self.clone_yaml(),
            ..*self
        }
    }
}

impl<R: Repr> Hash for Node<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.yaml.hash(state)
    }
}

impl<R: Repr> PartialEq for Node<R> {
    fn eq(&self, rhs: &Self) -> bool {
        self.yaml.eq(&rhs.yaml)
    }
}

impl<R: Repr> Eq for Node<R> {}

/// Indicator of the node use to index the sequence position.
pub struct Ind(pub usize);

impl<R: Repr> Index<Ind> for Node<R> {
    type Output = Self;

    fn index(&self, index: Ind) -> &Self::Output {
        if let Yaml::Seq(v) = self.yaml() {
            v.index(index.0)
        } else {
            panic!("out of bound!")
        }
    }
}

impl<R, I> Index<I> for Node<R>
where
    R: Repr,
    I: Into<Self>,
{
    type Output = Self;

    fn index(&self, index: I) -> &Self::Output {
        if let Yaml::Map(m) = self.yaml() {
            m.get(&index.into())
                .unwrap_or_else(|| panic!("out of bound!"))
        } else {
            panic!("out of bound!")
        }
    }
}

impl<R, Y> From<Y> for Node<R>
where
    R: Repr,
    Y: Into<Yaml<R>>,
{
    fn from(yaml: Y) -> Self {
        Self::new(yaml, 0, "")
    }
}

impl_iter! {
    impl Self
    impl (Self, Self)
}
