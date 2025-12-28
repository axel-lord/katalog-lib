//! Trait definitions.

use ::core::{hash::Hash, mem::discriminant, str::FromStr};

/// Trait for simple enums to provide all values.
///
/// # Safety
/// The `VARIANTS` associated constant must contain all variants.
/// `index_of` must return the correct index.
pub unsafe trait Variants
where
    Self: 'static + Sized,
{
    /// All values for enum.
    const VARIANTS: &[Self];

    /// Get index of variant in variant array.
    fn index_of(&self) -> usize;

    /// Use [Variants::VARIANTS] to get a reference with any lifetime with
    /// a value matching another value.
    fn variant_lifetime_cast<'a>(value: &Self) -> &'a Self {
        let disc = discriminant(value);
        let mut variants = Self::VARIANTS;
        while let [head, remainder @ ..] = variants {
            if discriminant(head) == disc {
                return head;
            }
            variants = remainder;
        }

        // SAFETY: Implementation of Variants and an existing value guarantees
        // at leas once head should have a discriminant matching the value.
        unsafe { ::core::hint::unreachable_unchecked() }
    }
}

/// Trait to provide a set of variants.
pub trait PartialVariants {
    /// Provide an iterator of variants.
    fn partial_variants<'a>() -> impl IntoIterator<Item = &'a Self>
    where
        Self: 'a;

    /// Get the next variant in partial variants.
    ///
    ///
    /// If self is not part of partial_variants the first item
    /// is returned.
    ///
    /// # Panics
    /// If `partial_variants` returns no values.
    fn partial_cycle_next<'a>(&self) -> &'a Self
    where
        Self: 'a + PartialEq,
    {
        let mut iter = Self::partial_variants().into_iter().peekable();
        let Some(first) = iter.peek() else {
            panic!("partial_variants returned an empty iterator")
        };
        let first = *first;

        while let (Some(value), Some(next)) = (iter.next(), iter.peek()) {
            if value == self {
                return next;
            }
        }

        first
    }
    /// Get the prev variant in partial variants.
    ///
    /// If self is not part of partial_variants the last item
    /// is returned.
    ///
    /// # Panics
    /// If `partial_variants` returns no values.
    fn partial_cycle_prev<'a>(&self) -> &'a Self
    where
        Self: 'a + PartialEq,
    {
        let mut iter = Self::partial_variants().into_iter().peekable();
        let Some(first) = iter.peek() else {
            panic!("partial_variants returned an empty iterator")
        };
        let mut last = *first;

        while let (Some(value), Some(next)) = (iter.next(), iter.peek()) {
            last = *next;
            if *next == self {
                return value;
            }
        }

        last
    }
}

impl<T> PartialVariants for T
where
    T: Variants + Cycle,
{
    #[inline]
    fn partial_variants<'a>() -> impl IntoIterator<Item = &'a Self>
    where
        Self: 'a,
    {
        T::VARIANTS
    }

    #[inline]
    fn partial_cycle_next<'a>(&self) -> &'a Self
    where
        Self: 'a + PartialEq,
    {
        T::variant_lifetime_cast(&Cycle::cycle_next(self))
    }

    #[inline]
    fn partial_cycle_prev<'a>(&self) -> &'a Self
    where
        Self: 'a + PartialEq,
    {
        T::variant_lifetime_cast(&Cycle::cycle_prev(self))
    }
}

/// Trait for simple enums to cycle the value.
///
/// # Safety
/// `cycle_next` must return the cyclic next variant in `VARIANTS`.
/// `cycle_prev` must return the cyclic previous variant in `VARIANTS`.
pub unsafe trait Cycle {
    /// Get the next variant. For the last variant will return the first variant.
    fn cycle_next(&self) -> Self;

    /// Get the next variant. For the last variant will return the first variant.
    fn cycle_prev(&self) -> Self;
}

/// Trait for getting the name of an enum variant.
///
/// By default round-trips with derived [FromStr] for simple enums.
pub trait AsStr {
    /// Get the name of the current variant.
    fn as_str<'a>(&self) -> &'a str;
}

impl<T: AsStr> AsStr for &T {
    #[inline]
    fn as_str<'a>(&self) -> &'a str {
        T::as_str(self)
    }
}

/// Provide a proxy struct with custom non-trait methods.
pub trait Proxy {
    /// Proxy type.
    type Proxy: AsRef<Self>;

    /// Return proxy object.
    fn proxy(&self) -> &Self::Proxy;
}

/// Convert a struct to it's fields.
pub trait IntoFields {
    /// Representation of any non-skipped field.
    type Field;

    /// Collection of all non-skipped fields of self.
    type IntoFields: IntoIterator<Item = Self::Field> + AsRef<[Self::Field]>;

    /// Convert self into a collection of fields.
    fn into_fields(self) -> Self::IntoFields;
}

/// Apply a field as a delta updating the current value.
pub trait FieldDelta {
    /// Field delta which may be applied.
    type FieldDelta;

    /// Apply a single field as a change to self.
    fn delta(&mut self, delta: Self::FieldDelta);
}

/// Trait for structs providing an indexing enum to index fields.
pub trait FieldsIdx {
    /// Type to index fields with.
    type FieldIdx;

    /// Type for field reference.
    type FieldRef<'this>
    where
        Self: 'this;

    /// Get a field by index.
    fn get(&self, idx: Self::FieldIdx) -> Self::FieldRef<'_>;
}

/// Trait for structs providing an indexing enum to index fields mutably.
pub trait FieldsIdxMut
where
    Self: FieldsIdx,
{
    /// Type for field reference.
    type FieldMut<'this>
    where
        Self: 'this;

    /// Get a mut field by index.
    fn get_mut(&mut self, idx: Self::FieldIdx) -> Self::FieldMut<'_>;
}

/// Collection trait for all struct field access traits.
///
/// Should be derived.
pub trait Fields
where
    for<'f> Self: 'f
        + IntoFields<Field = <Self as Fields>::Field>
        + FieldsIdx<
            FieldIdx = <Self as Fields>::FieldIdx,
            FieldRef<'f> = <Self as Fields>::FieldRef<'f>,
        >
        + FieldsIdxMut<FieldMut<'f> = <Self as Fields>::FieldMut<'f>>
        + FieldDelta<FieldDelta = <Self as Fields>::Field>,
{
    /// An enum which may be any field of struct.
    type Field: AsRef<<Self as Fields>::FieldIdx>;

    /// Type used when indexing fields.
    type FieldIdx: AsRef<<Self as Fields>::FieldIdx>
        + AsStr
        + FromStr
        + Variants
        + Cycle
        + Copy
        + Eq
        + Ord
        + Hash;

    /// A reference to a field.
    type FieldRef<'f>: AsRef<<Self as Fields>::FieldIdx>;

    /// A mutable reference to a field.
    type FieldMut<'f>: AsRef<<Self as Fields>::FieldIdx>;

    /// Container of references to fields.
    type FieldsRef<'f>: IntoIterator<Item = <Self as Fields>::FieldRef<'f>>
        + AsRef<[<Self as FieldsIdx>::FieldRef<'f>]>
    where
        Self: 'f;

    /// Container of mutable references to fields.
    type FieldsMut<'f>: IntoIterator<Item = <Self as Fields>::FieldMut<'f>>
        + AsRef<[<Self as FieldsIdxMut>::FieldMut<'f>]>
    where
        Self: 'f;

    /// Get references to fields.
    fn fields(&self) -> <Self as Fields>::FieldsRef<'_>;

    /// Get mutable references to fields.
    fn fields_mut(&mut self) -> <Self as Fields>::FieldsMut<'_>;
}
