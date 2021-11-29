//! This module contains representation holder, the reference counter type.
//!
//! [`Rc`] is the single thread reference counter,
//! and [`Arc`] is the multiple thread reference counter.
use crate::*;
use alloc::{rc::Rc, string::String, sync::Arc};
use core::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    ops::Deref,
};

/// The representation holder for [`Rc`].
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RcRepr(Rc<Inner<Self>>);
/// The representation holder for [`Arc`].
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct ArcRepr(Arc<Inner<Self>>);

/// Inner data of the nodes.
///
/// Please access the fields by [`NodeBase`].
#[derive(Eq, Clone)]
pub struct Inner<R: Repr> {
    pub(crate) pos: u64,
    pub(crate) tag: String,
    pub(crate) anchor: String,
    pub(crate) yaml: YamlBase<R>,
}

impl<R: Repr> Debug for Inner<R> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_fmt(format_args!("{:?}", &self.yaml))
    }
}

impl<R: Repr> Hash for Inner<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.yaml.hash(state);
    }
}

impl<R: Repr> PartialEq for Inner<R> {
    fn eq(&self, rhs: &Self) -> bool {
        self.yaml.eq(&rhs.yaml)
    }
}

/// The generic representation holder for [`YamlBase`] and [`NodeBase`].
///
/// See the implementor list for the choose.
pub trait Repr: AsRef<Inner<Self>> + Deref + Hash + Eq + Clone + Debug {
    /// The creation function of this type.
    fn repr(yaml: YamlBase<Self>, pos: u64, tag: String, anchor: String) -> Self;
}

macro_rules! impl_repr {
    ($ty:ty, $inner:ident) => {
        impl Repr for $ty {
            fn repr(yaml: YamlBase<Self>, pos: u64, tag: String, anchor: String) -> Self {
                Self($inner::new(Inner {
                    pos,
                    tag,
                    anchor,
                    yaml,
                }))
            }
        }

        impl Debug for $ty {
            fn fmt(&self, f: &mut Formatter) -> FmtResult {
                f.write_fmt(format_args!("{:?}", &self.0.yaml))
            }
        }

        impl AsRef<Inner<Self>> for $ty {
            #[inline(always)]
            fn as_ref(&self) -> &Inner<Self> {
                &self.0
            }
        }

        impl Deref for $ty {
            type Target = $inner<Inner<Self>>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

impl_repr! {RcRepr, Rc}
impl_repr! {ArcRepr, Arc}
