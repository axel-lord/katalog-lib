//! Reflection utils.

use ::core::fmt::Display;

#[doc(inline)]
pub use ::core::str::FromStr;

#[doc(inline)]
pub use ::katalog_lib_traits::*;

#[doc(inline)]
pub use ::katalog_lib_reflect_derive::{AsStr, Cycle, Fields, FromStr, Proxy, Variants};

/// Error returned by [FromStr] implementations
/// when trying to crate an enum from an unknown variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct UnknownVariant;

impl Display for UnknownVariant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("no variant with given name available")
    }
}
impl ::core::error::Error for UnknownVariant {}

#[cfg(test)]
mod tests {
    use super::*;
    use ::pretty_assertions::assert_eq;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Variants, Cycle, AsStr, FromStr)]
    #[reflect(crate_path = crate, as_ref, display, try_from, case_convert)]
    enum VariantsTestEnum {
        First,
        Second,
        #[as_str = "3:rd"]
        Third,
        #[as_str("4")]
        Fourth,
    }

    #[derive(Debug, Proxy, Fields)]
    #[reflect(crate_path = crate, option, getter, debug)]
    struct OptDefaultTestStruct {
        first: Option<String>,
        #[reflect(default = 5)]
        second: Option<i32>,
        #[reflect(proxy(no_option))]
        third: u32,
        #[reflect(some_pattern = Ok)]
        fourth: Result<u8, ()>,
    }

    #[test]
    fn derived_option_default_all_default() {
        let s = OptDefaultTestStruct {
            first: None,
            second: None,
            third: 7,
            fourth: Err(()),
        };

        assert_eq!(s.proxy().first().as_str(), "");
        assert_eq!(*s.proxy().second(), 5);
        assert_eq!(*s.proxy().third(), 7);
        assert_eq!(*s.proxy().fourth(), 0);
    }

    #[test]
    fn derived_option_default_all_set() {
        let s = OptDefaultTestStruct {
            first: Some(String::from("Hello")),
            second: Some(53),
            third: 9,
            fourth: Ok(15),
        };

        assert_eq!(s.proxy().first().as_str(), "Hello");
        assert_eq!(*s.proxy().second(), 53);
        assert_eq!(*s.proxy().third(), 9);
        assert_eq!(*s.proxy().fourth(), 15);
    }

    #[test]
    fn derived_variants() {
        use VariantsTestEnum::*;

        assert_eq!(First.index_of(), 0);
        assert_eq!(Fourth.index_of(), 3);

        assert_eq!(VariantsTestEnum::VARIANTS, &[First, Second, Third, Fourth]);
    }

    #[test]
    fn derived_cycle() {
        use VariantsTestEnum::*;

        assert_eq!(First.cycle_next(), Second);
        assert_eq!(First.cycle_prev(), Fourth);

        assert_eq!(Fourth.cycle_next(), First);
        assert_eq!(Fourth.cycle_prev(), Third);
    }

    #[test]
    fn derived_as_str() {
        use VariantsTestEnum::*;

        assert_eq!(First.as_str(), "First");
        assert_eq!(Second.as_str(), "Second");
    }

    #[test]
    fn derived_as_str_variants() {
        for (variant, str_rep) in VariantsTestEnum::VARIANTS
            .iter()
            .zip(["First", "Second", "3:rd", "4"])
        {
            assert_eq!(variant.as_str(), str_rep);
        }
    }

    #[test]
    fn derived_from_str() {
        use VariantsTestEnum::*;

        assert_eq!(Ok(First), "First".parse());
        assert_eq!(Ok(Second), "Second".parse());
        assert_eq!(Ok(Third), "3:rd".parse());
        assert_eq!(Ok(Fourth), "4".parse());
        assert_eq!(Err(UnknownVariant), "abc".parse::<VariantsTestEnum>());
    }
}
