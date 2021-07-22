//! This module contains representation holder, the reference counter type.
//!
//! [`Rc`] is the single thread reference counter,
//! and [`Arc`] is the multiple thread reference counter.
use crate::*;
use std::{
    fmt::{Debug, Formatter, Result as FmtResult},
    hash::{Hash, Hasher},
    rc::Rc,
    sync::Arc,
};

/// The representation holder for [`Rc`].
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct RcRepr(Rc<Inner<Self>>);
/// The representation holder for [`Arc`].
#[derive(Hash, Eq, PartialEq, Clone)]
pub struct ArcRepr(Arc<Inner<Self>>);

#[derive(Eq, Clone)]
struct Inner<R: repr::Repr> {
    pos: u64,
    ty: String,
    anchor: String,
    yaml: YamlBase<R>,
}

impl<R: repr::Repr> Debug for Inner<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_fmt(format_args!("{:?}", &self.yaml))
    }
}

impl<R: repr::Repr> Hash for Inner<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.yaml.hash(state)
    }
}

impl<R: repr::Repr> PartialEq for Inner<R> {
    fn eq(&self, rhs: &Self) -> bool {
        self.yaml.eq(&rhs.yaml)
    }
}

/// The generic representation holder for [`YamlBase`] and [`NodeBase`].
///
/// See the implementor list for the choose.
pub trait Repr: Hash + Eq + PartialEq + Clone + Debug + Sized {
    fn repr(yaml: YamlBase<Self>, pos: u64, ty: String, anchor: String) -> Self;
    fn pos(&self) -> u64;
    fn ty(&self) -> &str;
    fn anchor(&self) -> &str;
    fn yaml(&self) -> &YamlBase<Self>;
    fn into_yaml(self) -> YamlBase<Self>;
}

macro_rules! impl_repr {
    ($ty:ty, $inner:ident) => {
        impl Repr for $ty {
            fn repr(yaml: YamlBase<Self>, pos: u64, ty: String, anchor: String) -> Self {
                Self($inner::new(Inner {
                    pos,
                    ty,
                    anchor,
                    yaml,
                }))
            }

            #[inline(always)]
            fn pos(&self) -> u64 {
                self.0.pos
            }

            #[inline(always)]
            fn ty(&self) -> &str {
                &self.0.ty
            }

            #[inline(always)]
            fn anchor(&self) -> &str {
                &self.0.anchor
            }

            #[inline(always)]
            fn yaml(&self) -> &YamlBase<Self> {
                &self.0.yaml
            }

            #[inline(always)]
            fn into_yaml(self) -> YamlBase<Self> {
                $inner::try_unwrap(self.0).unwrap().yaml
            }
        }

        impl Debug for $ty {
            fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
                f.write_fmt(format_args!("{:?}", &self.0.yaml))
            }
        }
    };
}

impl_repr! {RcRepr, Rc}
impl_repr! {ArcRepr, Arc}
