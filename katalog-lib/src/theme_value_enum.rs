//! `ThemeValueEnum` impl.
use ::core::{
    fmt::Display,
    mem::{Discriminant, discriminant},
    str::FromStr,
};
use ::std::{
    collections::HashMap,
    sync::{LazyLock, OnceLock},
};

use ::clap::{ValueEnum, builder::PossibleValue};
use ::derive_more::{From, Into};
use ::iced_core::{Theme, theme::Base};
use ::katalog_lib_traits::PartialVariants;
use ::serde::{Deserialize, Serialize};

/// Uppercase equality.
fn uc_eq(a: &str, b: &str) -> bool {
    let a = a.chars().flat_map(|chr| chr.to_uppercase());
    let b = b.chars().flat_map(|chr| chr.to_uppercase());
    a.eq(b)
}

/// Theme details.
#[derive(Debug, Clone)]
struct Thm {
    /// Theme itself.
    theme: &'static Theme,
    /// Simplified name of theme.
    simple: String,
}

/// Mapping from theme names to theme.
static THEMES: LazyLock<HashMap<Discriminant<Theme>, Thm>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(Theme::ALL.len());

    for theme in Theme::ALL {
        let name = theme.name();
        let simple = name
            .chars()
            .map(|chr| if chr.is_whitespace() { '-' } else { chr })
            .flat_map(|chr| chr.to_lowercase())
            .filter(|chr| chr.is_ascii())
            .collect();

        map.insert(discriminant(theme), Thm { theme, simple });
    }

    map
});

impl Thm {
    /// Get theme name.
    fn name(&self) -> &'static str {
        self.theme.name()
    }
    /// Get theme from key.
    fn get(key: &str) -> Option<&'static Thm> {
        THEMES
            .values()
            .find(|thm| uc_eq(key, &thm.simple) || uc_eq(key, thm.name()))
    }
}

/// Theme wrapper implementing [ValueEnum].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, From, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ThemeValueEnum(&'static Theme);

impl PartialEq for ThemeValueEnum {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        discriminant(self.0) == discriminant(other.0)
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
        Thm::get(&value).map_or_else(
            || Err(ThemeValueEnumFromStringError::from(value)),
            |thm| Ok(Self(thm.theme)),
        )
    }
}

impl FromStr for ThemeValueEnum {
    type Err = ThemeValueEnumFromStringError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Thm::get(s).map_or_else(
            || Err(ThemeValueEnumFromStringError::from(s.to_owned())),
            |thm| Ok(Self(thm.theme)),
        )
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
        THEMES
            .get(&discriminant(self.0))
            .map(|Thm { theme, simple }| {
                PossibleValue::new(simple.as_str())
                    .alias(theme.name())
                    .help(theme.name())
            })
    }

    fn from_str(input: &str, _ignore_case: bool) -> Result<Self, String> {
        Thm::get(input).map_or_else(
            || Err(format!("could not get iced theme {input}")),
            |thm| Ok(Self(thm.theme)),
        )
    }
}
