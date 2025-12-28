//! `ThemeValueEnum` impl.
use ::core::{fmt::Display, mem::discriminant, str::FromStr};
use ::std::sync::OnceLock;

use ::clap::{ValueEnum, builder::PossibleValue};
use ::derive_more::{From, Into};
use ::iced_core::{Theme, theme::Base};
use ::katalog_lib_traits::PartialVariants;
use ::serde::{Deserialize, Serialize};

/// Theme wrapper implementing [ValueEnum].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ThemeValueEnum(&'static Theme);

impl PartialEq for ThemeValueEnum {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.name() == other.0.name()
    }
}

impl PartialVariants for ThemeValueEnum {
    fn partial_variants<'a>() -> impl IntoIterator<Item = &'a Self>
    where
        Self: 'a,
    {
        <Self as ValueEnum>::value_variants()
    }
}

impl ThemeValueEnum {
    /// Convert into an instance of inner value.
    #[inline]
    pub fn into_inner(self) -> Theme {
        self.0.clone()
    }
}

impl Display for ThemeValueEnum {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        Display::fmt(self.0.name(), f)
    }
}

/// Error returned when converting to a [ThemeValueEnum] from a
/// [String] fails.
#[derive(Debug, ::thiserror::Error, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, From, Into)]
#[error("{0} is not the name of a theme")]
#[repr(transparent)]
pub struct ThemeValueEnumFromStringError(pub String);

impl Default for ThemeValueEnum {
    fn default() -> Self {
        Self(&Theme::Dark)
    }
}

impl TryFrom<String> for ThemeValueEnum {
    type Error = ThemeValueEnumFromStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        for variant in Theme::ALL {
            if variant.name().eq_ignore_ascii_case(&value) {
                return Ok(Self(variant));
            }
        }
        Err(value.into())
    }
}

impl FromStr for ThemeValueEnum {
    type Err = ThemeValueEnumFromStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for variant in Theme::ALL {
            if variant.name().eq_ignore_ascii_case(s) {
                return Ok(Self(variant));
            }
        }
        Err(s.to_owned().into())
    }
}

impl From<ThemeValueEnum> for String {
    #[inline]
    fn from(value: ThemeValueEnum) -> Self {
        value.0.name().to_owned()
    }
}

impl TryFrom<Theme> for ThemeValueEnum {
    type Error = Theme;

    fn try_from(value: Theme) -> Result<Self, Self::Error> {
        for theme in Theme::ALL {
            if discriminant(theme) == discriminant(&value) {
                return Ok(Self(theme));
            }
        }
        Err(value)
    }
}

impl From<ThemeValueEnum> for Theme {
    fn from(value: ThemeValueEnum) -> Self {
        value.into_inner()
    }
}

impl ValueEnum for ThemeValueEnum {
    fn value_variants<'a>() -> &'a [Self] {
        static VARIANTS: OnceLock<Vec<ThemeValueEnum>> = OnceLock::new();
        VARIANTS.get_or_init(|| Theme::ALL.iter().map(ThemeValueEnum).collect())
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(PossibleValue::new(self.0.name()))
    }

    fn from_str(input: &str, ignore_case: bool) -> Result<Self, String> {
        for theme in Theme::ALL {
            if ignore_case {
                if theme.name().eq_ignore_ascii_case(input) {
                    return Ok(Self(theme));
                }
            } else if theme.name() == input {
                return Ok(Self(theme));
            }
        }
        Err(format!("could not get iced theme {input}"))
    }
}
