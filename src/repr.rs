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
    type Rc: Deref<Target = Yaml<Self>> + Hash + Eq + Clone + Debug;

    /// The creation function of this type.
    fn new_rc(yaml: Yaml<Self>) -> Self::Rc;
}

impl Repr for RcRepr {
    type Rc = Rc<Yaml<Self>>;

    fn new_rc(yaml: Yaml<Self>) -> Self::Rc {
        Rc::new(yaml)
    }
}

impl Repr for ArcRepr {
    type Rc = Arc<Yaml<Self>>;

    fn new_rc(yaml: Yaml<Self>) -> Self::Rc {
        Arc::new(yaml)
    }
}
