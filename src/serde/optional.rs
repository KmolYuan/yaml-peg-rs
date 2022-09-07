use serde::{Deserialize, Serialize};

/// A data type that can support listed items,
/// or inline it if there is single item.
///
/// ```
/// use serde::Deserialize;
/// use yaml_peg::{node, serde::Optional};
///
/// #[derive(Deserialize)]
/// struct Content {
///     img: Optional<Img>,
/// }
///
/// #[derive(Deserialize)]
/// struct Img {
///     src: String,
/// }
///
/// impl Default for Img {
///     fn default() -> Self {
///         Self { src: "~/img/.desktop.png".to_string() }
///     }
/// }
///
/// let n_disabled = node!({"img" => false});
/// let n_default = node!({"img" => true});
/// let n_enabled = node!({"img" => node!({"src" => "img/1.png"})});
/// let disabled = Content::deserialize(n_disabled).unwrap();
/// let default = Content::deserialize(n_default).unwrap();
/// let enabled = Content::deserialize(n_enabled).unwrap();
/// let mut doc = String::new();
/// disabled.img.ok(|img| doc += &img.src);
/// doc += ":";
/// default.img.ok(|img| doc += &img.src);
/// doc += ":";
/// enabled.img.ok(|img| doc += &img.src);
/// assert_eq!(":~/img/.desktop.png:img/1.png", doc);
/// ```
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum Optional<T> {
    /// Boolean value, false means disable, true means default.
    Bool(bool),
    /// The custom option.
    Some(T),
}

impl<T> Default for Optional<T> {
    fn default() -> Self {
        Self::Bool(true)
    }
}

impl<T> Optional<T> {
    /// Do things with the provided functions, only if the value is enable.
    ///
    /// If the value is `Bool(true)`, use the default value instead.
    pub fn ok(&self, mut ok: impl FnMut(&T))
    where
        T: Default,
    {
        match self {
            Optional::Bool(false) => {}
            Optional::Bool(true) => ok(&T::default()),
            Optional::Some(t) => ok(t),
        }
    }

    /// Same as [`Self::ok`], but provide default value directly.
    pub fn ok_instead(&self, mut ok: impl FnMut(&T), default: &T) {
        match self {
            Optional::Bool(false) => {}
            Optional::Bool(true) => ok(default),
            Optional::Some(t) => ok(t),
        }
    }

    /// Do things with the provided functions, the functions can return the
    /// value in both cases. (enabled / disabled)
    ///
    /// If the value is `Bool(true)`, use the default value instead.
    pub fn ok_or<Ok, Or, R>(&self, mut ok: Ok, mut or: Or) -> R
    where
        T: Default,
        Ok: FnMut(&T) -> R,
        Or: FnMut() -> R,
    {
        match self {
            Optional::Bool(false) => or(),
            Optional::Bool(true) => ok(&T::default()),
            Optional::Some(t) => ok(t),
        }
    }

    /// Same as [`Self::ok_or`], but provide default value directly.
    pub fn ok_or_instead<Ok, Or, R>(&self, mut ok: Ok, default: &T, mut or: Or) -> R
    where
        Ok: FnMut(&T) -> R,
        Or: FnMut() -> R,
    {
        match self {
            Optional::Bool(false) => or(),
            Optional::Bool(true) => ok(default),
            Optional::Some(t) => ok(t),
        }
    }
}
