use crate::{repr::*, *};
use alloc::string::ToString;
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::Hash,
    iter::FromIterator,
    ops::{Deref, Index},
    str::FromStr,
};

macro_rules! as_method {
    {$(#[$meta:meta])* fn $id:ident = $ty:ident$(($op:ident))?
        $(| ($default:expr)?)?
        $(| $ty2:ident)* -> $r:ty} => {
        $(#[$meta])*
        pub fn $id(&self) -> Result<$r, u64> {
            match self.yaml() {
                YamlBase::$ty(v) $(| YamlBase::$ty2(v))* => Ok(v$(.$op())?),
                $(YamlBase::Null => Ok($default),)?
                _ => Err(self.pos()),
            }
        }
    };
}

macro_rules! as_num_method {
    {$(#[$meta:meta])* fn $id:ident = $ty1:ident $(| $ty2:ident)*} => {
        $(#[$meta])*
        pub fn $id<N: FromStr>(&self) -> Result<N, u64> {
            match self.yaml() {
                YamlBase::$ty1(n) $(| YamlBase::$ty2(n))* => match n.parse() {
                    Ok(v) => Ok(v),
                    Err(_) => Err(self.pos()),
                },
                _ => Err(self.pos()),
            }
        }
    };
}

macro_rules! impl_iter {
    ($(impl $item:ty)+) => {
        $(impl<R: Repr> FromIterator<$item> for NodeBase<R> {
            fn from_iter<T: IntoIterator<Item = $item>>(iter: T) -> Self {
                Self::from(iter.into_iter().collect::<YamlBase<R>>())
            }
        })+
    };
}

/// A node with [`alloc::rc::Rc`] holder.
pub type Node = NodeBase<RcRepr>;
/// A node with [`alloc::sync::Arc`] holder.
pub type ArcNode = NodeBase<ArcRepr>;

/// Readonly node, including line number, column number, type assertion and anchor.
/// You can access [`YamlBase`] type through [`NodeBase::yaml`] method.
///
/// This type will ignore additional information when comparison and hashing.
///
/// ```
/// use std::collections::HashSet;
/// use yaml_peg::{Node, Yaml};
///
/// let mut s = HashSet::new();
/// s.insert(Node::new(Yaml::from("a"), 0, "", ""));
/// s.insert(Node::new(Yaml::from("a"), 1, "my-type", ""));
/// s.insert(Node::new(Yaml::from("a"), 2, "", "my-anchor"));
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
/// Same as containers, to prevent panic, the [`NodeBase::get`] method is the best choice.
/// The [`NodeBase::get_default`] can provide missing key value when indexing.
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
/// # Anchor
///
/// The anchors can be infer from [`AnchorBase`], and attach with [`NodeBase::as_anchor`] method.
///
/// # Clone
///
/// Since the YAML data is wrapped by reference counter [`alloc::rc::Rc`] and [`alloc::sync::Arc`],
/// cloning node just increase the reference counter,
/// the entire data structure are still shared together.
///
/// ```
/// use std::rc::Rc;
/// use yaml_peg::node;
///
/// let a = node!("a");
/// {
///     let b = a.clone();
///     assert_eq!(2, Rc::strong_count(&b));
/// }
/// assert_eq!(1, Rc::strong_count(&a));
/// ```
///
/// If you want to copy data, please get the data first.
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct NodeBase<R: Repr>(pub(crate) R);

impl<R: Repr> NodeBase<R> {
    /// Create node from YAML data.
    pub fn new(yaml: YamlBase<R>, pos: u64, tag: &str, anchor: &str) -> Self {
        Self(R::repr(yaml, pos, tag.to_string(), anchor.to_string()))
    }

    /// Document position.
    #[inline(always)]
    pub fn pos(&self) -> u64 {
        self.0.as_ref().pos
    }

    /// Tag.
    /// If the tag is not specified, returns a default tag from core schema.
    ///
    /// Anchor has no tag.
    #[inline(always)]
    pub fn tag(&self) -> &str {
        match self.0.as_ref().tag.as_str() {
            "" => match self.yaml() {
                YamlBase::Null => concat!(parser::tag_prefix!(), "null"),
                YamlBase::Bool(_) => concat!(parser::tag_prefix!(), "bool"),
                YamlBase::Int(_) => concat!(parser::tag_prefix!(), "int"),
                YamlBase::Float(_) => concat!(parser::tag_prefix!(), "float"),
                YamlBase::Str(_) => concat!(parser::tag_prefix!(), "str"),
                YamlBase::Seq(_) => concat!(parser::tag_prefix!(), "seq"),
                YamlBase::Map(_) => concat!(parser::tag_prefix!(), "map"),
                YamlBase::Anchor(_) => "",
            },
            s => s,
        }
    }

    /// Anchor reference.
    #[inline(always)]
    pub fn anchor(&self) -> &str {
        &self.0.as_ref().anchor
    }

    /// YAML data.
    #[inline(always)]
    pub fn yaml(&self) -> &YamlBase<R> {
        &self.0.as_ref().yaml
    }

    /// Check the value is null.
    pub fn is_null(&self) -> bool {
        *self.yaml() == YamlBase::Null
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
    }

    as_num_method! {
        /// Convert to integer.
        ///
        /// ```
        /// use yaml_peg::node;
        ///
        /// assert_eq!(60, node!(60).as_int().unwrap());
        /// ```
        fn as_int = Int
    }

    as_num_method! {
        /// Convert to float.
        ///
        /// ```
        /// use yaml_peg::node;
        ///
        /// assert_eq!(20.06, node!(20.06).as_float().unwrap());
        /// ```
        fn as_float = Float
    }

    as_num_method! {
        /// Convert to number.
        ///
        /// ```
        /// use yaml_peg::node;
        ///
        /// assert_eq!(60, node!(60).as_number().unwrap());
        /// assert_eq!(20.06, node!(20.06).as_number().unwrap());
        /// ```
        fn as_number = Int | Float
    }

    as_method! {
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
            YamlBase::Str(s) | YamlBase::Int(s) | YamlBase::Float(s) => Ok(s),
            YamlBase::Bool(true) => Ok("true"),
            YamlBase::Bool(false) => Ok("false"),
            YamlBase::Null => Ok(""),
            _ => Err(self.pos()),
        }
    }

    /// Infer an anchor with anchor visitor.
    ///
    /// If the node is not a anchor, or the anchor does not exist, the original node is returned.
    /// Since the anchor type is invalid except for this method, missing anchor will still return an error.
    ///
    /// ```
    /// use yaml_peg::{node, anchors};
    ///
    /// let node_a = node!(*"a");
    /// let v = anchors!["a" => 20.];
    /// assert_eq!(20., node_a.as_anchor(&v).as_float().unwrap());
    /// ```
    pub fn as_anchor<'a, 'b: 'a>(&'a self, v: &'b AnchorBase<R>) -> &'a Self {
        match self.yaml() {
            YamlBase::Anchor(s) if v.contains_key(s) => v.get(s).unwrap(),
            _ => self,
        }
    }

    as_method! {
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
    }

    as_method! {
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
        if let YamlBase::Map(m) = self.yaml() {
            if let Some(n) = m.get(&key.into()) {
                Ok(n)
            } else {
                Err(self.pos())
            }
        } else {
            Err(self.pos())
        }
    }

    /// Same as [`NodeBase::get`] but provide default value if the key is missing.
    /// For this method, a transform method `as_*` is required.
    ///
    /// + If the value exist, return the value.
    /// + If value is a wrong type, return `Err` with node position.
    /// + If the value is not exist, return the default value.
    ///
    /// ```
    /// # fn main() -> Result<(), u64> {
    /// use yaml_peg::{node, NodeBase};
    ///
    /// let a = node!({node!("a") => node!({node!("b") => node!("c")})});
    /// assert_eq!(
    ///     "c",
    ///     a.get("a")?.get_default("b", "d", NodeBase::as_str)?
    /// );
    /// let b = node!({node!("a") => node!({})});
    /// assert_eq!(
    ///     "d",
    ///     b.get("a")?.get_default("b", "d", NodeBase::as_str)?
    /// );
    /// let c = node!({node!("a") => node!({node!("b") => node!(20.)})});
    /// assert_eq!(
    ///     Err(0),
    ///     c.get("a")?.get_default("b", "d", NodeBase::as_str)
    /// );
    /// # Ok::<(), u64>(()) }
    /// ```
    ///
    /// ```
    /// # fn main() -> Result<(), u64> {
    /// use yaml_peg::{node, NodeBase};
    ///
    /// let n = node!({node!("a") => node!([node!(1), node!(2), node!(3)])});
    /// let a = n.get_default("c", vec![], NodeBase::as_seq)?;
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
        F: Fn(&'a Self) -> Result<Ret, u64>,
    {
        if let YamlBase::Map(m) = self.yaml() {
            if let Some(n) = m.get(&key.into()) {
                factory(n)
            } else {
                Ok(default)
            }
        } else {
            Err(self.pos())
        }
    }

    /// Trait as map then get a key-value with anchor visitor,
    /// returns default value if key is not found.
    ///
    /// If the value type is incorrect, returns [`Err`].
    /// Same as [`NodeBase::get_default`] but support anchor visitor.
    ///
    /// ```
    /// # fn main() -> Result<(), u64> {
    /// use yaml_peg::{anchors, node, Node};
    ///
    /// let v = anchors!["c" => 10];
    /// let a = node!({node!("a") => node!({node!("b") => node!(*"c")})});
    /// let c = a.get("a")?.with(&v, "b", 0, Node::as_int)?;
    /// assert_eq!(10, c);
    /// let c = a.get("a")?.with(&v, "b", "default", Node::as_str);
    /// assert_eq!(Err(0), c);
    /// # Ok::<(), u64>(()) }
    /// ```
    pub fn with<'a, 'b: 'a, Y, Ret, F>(
        &'a self,
        v: &'b AnchorBase<R>,
        key: Y,
        default: Ret,
        factory: F,
    ) -> Result<Ret, u64>
    where
        Y: Into<Self>,
        F: Fn(&'a Self) -> Result<Ret, u64>,
    {
        if let YamlBase::Map(m) = self.yaml() {
            if let Some(n) = m.get(&key.into()) {
                factory(n.as_anchor(v))
            } else {
                Ok(default)
            }
        } else {
            Err(self.pos())
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
        if let YamlBase::Seq(a) = self.yaml() {
            if let Some(n) = a.get(ind.0) {
                Ok(n)
            } else {
                Err(self.pos())
            }
        } else {
            Err(self.pos())
        }
    }
}

impl<R: Repr> Debug for NodeBase<R> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("Node{:?}", &self.0))
    }
}

impl<R: Repr> Deref for NodeBase<R> {
    type Target = R::Target;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// Node index indicator use to indicate sequence position.
pub struct Ind(pub usize);

impl<R: Repr> Index<Ind> for NodeBase<R> {
    type Output = Self;

    fn index(&self, index: Ind) -> &Self::Output {
        if let YamlBase::Seq(a) = self.yaml() {
            a.index(index.0)
        } else {
            panic!("out of bound!")
        }
    }
}

impl<R, I> Index<I> for NodeBase<R>
where
    R: Repr,
    I: Into<Self>,
{
    type Output = Self;

    fn index(&self, index: I) -> &Self::Output {
        if let YamlBase::Map(m) = self.yaml() {
            m.get(&index.into())
                .unwrap_or_else(|| panic!("out of bound!"))
        } else {
            panic!("out of bound!")
        }
    }
}

impl<R, Y> From<Y> for NodeBase<R>
where
    R: Repr,
    Y: Into<YamlBase<R>>,
{
    fn from(yaml: Y) -> Self {
        Self::new(yaml.into(), 0, "", "")
    }
}

impl_iter! {
    impl Self
    impl (Self, Self)
}
