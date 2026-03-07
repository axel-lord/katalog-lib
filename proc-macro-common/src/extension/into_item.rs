//! [IntoItem] impls.

use ::syn::Item;

/// Trait for items convertible into items.
pub trait IntoItem: Into<Item> {
    /// Convert self into an item.
    fn into_item(self) -> Item {
        <Self as Into<Item>>::into(self)
    }
}

impl<T: Into<Item>> IntoItem for T {}
