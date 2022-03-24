//! This module contains representation holder, the reference counter type.
//!
//! [`Rc`] is the single thread reference counter,
//! and [`Arc`] is the multiple thread reference counter.
use crate::*;
use alloc::{rc::Rc, sync::Arc};
use core::{fmt::Debug, hash::Hash, ops::Deref};

/// The representation holder for [`Rc`].
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct RcRepr;
/// The representation holder for [`Arc`].
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ArcRepr;

/// The generic representation holder for [`Yaml`] and [`Node`].
///
/// See the implementor list for the choose.
pub trait Repr: Hash + Eq + Clone + Debug {
    /// Type of the representation.
    type Ty: Deref<Target = Yaml<Self>> + Hash + Eq + Clone + Debug;
    /// The creation function of this type.
    fn repr(yaml: Yaml<Self>) -> Self::Ty;
}

impl Repr for RcRepr {
    type Ty = Rc<Yaml<Self>>;

    fn repr(yaml: Yaml<Self>) -> Self::Ty {
        Rc::new(yaml)
    }
}

impl Repr for ArcRepr {
    type Ty = Arc<Yaml<Self>>;

    fn repr(yaml: Yaml<Self>) -> Self::Ty {
        Arc::new(yaml)
    }
}
