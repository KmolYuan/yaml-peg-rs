//! This module contains the representation holders, the underlayer type of [`Yaml`] type.
//!
//! [`Rc`] is the single thread reference counter,
//! and [`Arc`] is the multiple thread reference counter.
use crate::Yaml;
use alloc::{rc::Rc, sync::Arc};
use core::{fmt::Debug, hash::Hash, ops::Deref};

/// The representation symbol for [`Rc`].
pub struct RcRepr;
/// The representation symbol for [`Arc`].
pub struct ArcRepr;

/// The generic representation holder for [`Yaml`].
///
/// See the implementor list for the choose.
pub trait Repr: Sized {
    /// Type of the representation, e.g., the reference counter type.
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
