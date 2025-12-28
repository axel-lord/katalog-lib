//! Common Library used by spel-katalog and arkiv-katalog.

pub mod reflect {
    //! Reflection module.

    #[doc(inline)]
    pub use ::katalog_lib_reflect::*;
}

#[doc(inline)]
pub use ::katalog_lib_traits::*;

#[doc(inline)]
pub use ::katalog_lib_widget::*;

#[doc(inline)]
pub use theme_value_enum::{ThemeValueEnum, ThemeValueEnumFromStringError};

mod theme_value_enum;
