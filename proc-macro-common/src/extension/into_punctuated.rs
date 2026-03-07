//! [IntoPunctuated] impls.

use ::syn::punctuated::Punctuated;

/// Trait for items convertible into punctuated lists.
pub trait IntoPunctuated
where
    Self: IntoIterator + Sized,
{
    /// Convert into a punctuated list without trailing punctuation.
    fn into_punctuated<P>(self, mut punct: impl FnMut() -> P) -> Punctuated<Self::Item, P> {
        let mut p = Punctuated::new();
        let mut items = self.into_iter();
        if let Some(item) = items.next() {
            p.push_value(item);
        }
        for item in items {
            p.push_punct(punct());
            p.push_value(item);
        }
        p
    }

    /// Convert into a punctuated list with trailing punctuation.
    fn into_punctuated_with_trailing<P>(
        self,
        mut punct: impl FnMut() -> P,
    ) -> Punctuated<Self::Item, P> {
        let mut p = Punctuated::new();
        for item in self {
            p.push_value(item);
            p.push_punct(punct());
        }
        p
    }
}

impl<T: IntoIterator> IntoPunctuated for T {}
