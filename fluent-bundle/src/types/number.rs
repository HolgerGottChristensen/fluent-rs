use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::TryInto;
use std::default::{Default};
use std::str::FromStr;
use fixed_decimal::FixedDecimal;
use icu::decimal::FixedDecimalFormatter;
use icu::decimal::options::{FixedDecimalFormatterOptions, GroupingStrategy};
use icu::locid::Locale;

use crate::args::FluentArgs;
use crate::types::FluentValue;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FluentNumberStyle {
    Decimal,
    Currency,
    Percent,
}

impl std::default::Default for FluentNumberStyle {
    fn default() -> Self {
        Self::Decimal
    }
}

impl From<&str> for FluentNumberStyle {
    fn from(input: &str) -> Self {
        match input {
            "decimal" => Self::Decimal,
            "currency" => Self::Currency,
            "percent" => Self::Percent,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FluentNumberNotation {
    Standard,
    Scientific,
    Engineering,
    // Compact
}

impl std::default::Default for FluentNumberNotation {
    fn default() -> Self {
        Self::Standard
    }
}

impl From<&str> for FluentNumberNotation {
    fn from(input: &str) -> Self {
        match input {
            "standard" => Self::Standard,
            "scientific" => Self::Scientific,
            "engineering" => Self::Engineering,
            _ => Self::default(),
        }
    }
}

/// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat#usegrouping
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FluentNumberGrouping {
    Always,
    Auto,
    Min2,
    Never,
}

impl std::default::Default for FluentNumberGrouping {
    fn default() -> Self {
        Self::Auto
    }
}

impl From<&str> for FluentNumberGrouping {
    fn from(input: &str) -> Self {
        match input {
            "always" => Self::Always,
            "auto" => Self::Auto,
            "min2" => Self::Min2,
            "never" => Self::Never,
            _ => Self::default(),
        }
    }
}

/// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat#usegrouping
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FluentNumberRoundingMode {
    Ceil,
    Floor,
    Expand,
    Trunc,
    HalfCeil,
    HalfFloor,
    HalfExpand,
    HalfTrunc,
    HalfEven,
}

impl std::default::Default for FluentNumberRoundingMode {
    fn default() -> Self {
        Self::HalfExpand
    }
}

impl From<&str> for FluentNumberRoundingMode {
    fn from(input: &str) -> Self {
        match input {
            "ceil" => Self::Ceil,
            "floor" => Self::Floor,
            "expand" => Self::Expand,
            "trunc" => Self::Trunc,
            "halfCeil" => Self::HalfCeil,
            "halfFloor" => Self::HalfFloor,
            "halfExpand" => Self::HalfExpand,
            "halfTrunc" => Self::HalfTrunc,
            "halfEven" => Self::HalfEven,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FluentNumberCurrencyDisplayStyle {
    Symbol,
    Code,
    Name,
}

impl std::default::Default for FluentNumberCurrencyDisplayStyle {
    fn default() -> Self {
        Self::Symbol
    }
}

impl From<&str> for FluentNumberCurrencyDisplayStyle {
    fn from(input: &str) -> Self {
        match input {
            "symbol" => Self::Symbol,
            "code" => Self::Code,
            "name" => Self::Name,
            _ => Self::default(),
        }
    }
}

/// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat#locale_options
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct FluentNumberOptions {
    pub style: FluentNumberStyle,
    pub notation: FluentNumberNotation,
    pub currency: Option<String>,
    pub currency_display: FluentNumberCurrencyDisplayStyle,
    pub use_grouping: FluentNumberGrouping,

    /// The minimum number of integer digits to use.
    /// A value with a smaller number of integer digits
    /// than this number will be left-padded with zeros
    /// (to the specified length) when formatted.
    pub minimum_integer_digits: Option<usize>,

    /// The minimum number of fraction digits to use.
    pub minimum_fraction_digits: Option<usize>,

    /// The maximum number of fraction digits to use.
    pub maximum_fraction_digits: Option<usize>,

    /// The minimum number of significant digits to use.
    pub minimum_significant_digits: Option<usize>,

    /// The maximum number of significant digits to use.
    pub maximum_significant_digits: Option<usize>,

    pub rounding_mode: FluentNumberRoundingMode,
}

impl FluentNumberOptions {
    pub fn merge(&mut self, opts: &FluentArgs) {
        for (key, value) in opts.iter() {
            match (key, value) {
                ("style", FluentValue::String(n)) => {
                    self.style = n.as_ref().into();
                }
                ("notation", FluentValue::String(n)) => {
                    self.notation = n.as_ref().into();
                }
                ("currency", FluentValue::String(n)) => {
                    self.currency = Some(n.to_string());
                }
                ("currencyDisplay", FluentValue::String(n)) => {
                    self.currency_display = n.as_ref().into();
                }
                ("useGrouping", FluentValue::String(n)) => {
                    self.use_grouping = n.as_ref().into();
                }
                ("roundingMode", FluentValue::String(n)) => {
                    self.rounding_mode = n.as_ref().into();
                }
                ("minimumIntegerDigits", FluentValue::Number(n)) => {
                    self.minimum_integer_digits = Some(n.into());
                }
                ("minimumFractionDigits", FluentValue::Number(n)) => {
                    self.minimum_fraction_digits = Some(n.into());
                }
                ("maximumFractionDigits", FluentValue::Number(n)) => {
                    self.maximum_fraction_digits = Some(n.into());
                }
                ("minimumSignificantDigits", FluentValue::Number(n)) => {
                    self.minimum_significant_digits = Some(n.into());
                }
                ("maximumSignificantDigits", FluentValue::Number(n)) => {
                    self.maximum_significant_digits = Some(n.into());
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FluentNumber {
    pub value: f64,
    pub options: FluentNumberOptions,
}

thread_local! {
    static FORMATTERS: RefCell<HashMap<Locale, HashMap<FluentNumberGrouping, FixedDecimalFormatter>>> = RefCell::new(HashMap::new());
}

impl FluentNumber {
    pub const fn new(value: f64, options: FluentNumberOptions) -> Self {
        Self { value, options }
    }

    pub fn as_string(&self, locale: &Locale) -> Cow<'static, str> {
        match self.options.notation {
            FluentNumberNotation::Standard => self.as_string_standard(locale),
            FluentNumberNotation::Scientific => self.as_string_scientific(locale, 1),
            FluentNumberNotation::Engineering => self.as_string_scientific(locale, 3),
        }
    }

    fn with_formatter<R, F: Fn(&FixedDecimalFormatter)->R>(&self, locale: &Locale, f: F)->R {
        let grouping = match self.options.use_grouping {
            FluentNumberGrouping::Always => GroupingStrategy::Always,
            FluentNumberGrouping::Auto => GroupingStrategy::Auto,
            FluentNumberGrouping::Min2 => GroupingStrategy::Min2,
            FluentNumberGrouping::Never => GroupingStrategy::Never,
        };

        FORMATTERS.with(|cell| {
            if let Some(groupings_map) = cell.borrow_mut().get_mut(locale) {
                if let Some(formatter) = groupings_map.get(&self.options.use_grouping) {
                    return f(formatter);
                }

                let new_formatter = FixedDecimalFormatter::try_new(
                    &locale.into(),
                    FixedDecimalFormatterOptions::from(grouping),
                )
                    .expect("locale should be present");

                let res = f(&new_formatter);

                groupings_map.insert(self.options.use_grouping, new_formatter);

                return res;
            }

            let mut groupings_map = HashMap::new();

            let new_formatter = FixedDecimalFormatter::try_new(
                &locale.into(),
                FixedDecimalFormatterOptions::from(grouping),
            )
                .expect("locale should be present");

            let res = f(&new_formatter);

            groupings_map.insert(self.options.use_grouping, new_formatter);

            cell.borrow_mut().insert(locale.clone(), groupings_map);

            res
        })
    }

    fn as_string_standard(&self, locale: &Locale) -> Cow<'static, str> {
        self.with_formatter(locale, |formatter| {
            formatter.format(&self.as_decimal()).to_string()
        }).into()
    }

    fn as_string_scientific(&self, locale: &Locale, multiple_of: i16) -> Cow<'static, str> {
        let mut decimal = FixedDecimal::from_str(&self.value.to_string())
            .expect("That the f64 value when formatted as a string is convertable to a fixed decimal");

        let magnitude = decimal.nonzero_magnitude_start() / multiple_of * multiple_of;


        decimal.multiply_pow10(-magnitude);
        decimal.trim_start();
        decimal.trim_end();

        let minimum_integer_digits = self.options.minimum_integer_digits.unwrap_or(2);
        let minimum_fraction_digits = self.options.minimum_fraction_digits.unwrap_or(3);
        let maximum_fraction_digits = self.options.maximum_fraction_digits.unwrap_or(minimum_fraction_digits.max(3)) as i16;

        match self.options.rounding_mode {
            FluentNumberRoundingMode::Ceil => decimal.ceil(-maximum_fraction_digits),
            FluentNumberRoundingMode::Floor => decimal.floor(-maximum_fraction_digits),
            FluentNumberRoundingMode::Expand => decimal.expand(-maximum_fraction_digits),
            FluentNumberRoundingMode::Trunc => decimal.trunc(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfCeil => decimal.half_ceil(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfFloor => decimal.half_floor(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfExpand => decimal.half_expand(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfTrunc => decimal.half_trunc(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfEven => decimal.half_even(-maximum_fraction_digits),
        };

        decimal.trim_end();
        decimal.pad_end(-(minimum_fraction_digits as i16));

        let mut magnitude_decimal = FixedDecimal::from(magnitude.abs());
        magnitude_decimal.pad_start(minimum_integer_digits as i16);

        self.with_formatter(locale, |formatter| {
            let mut string = formatter.format(&decimal).to_string();
            string.push_str("E");
            if magnitude.is_negative() {
                string.push_str("-"); // TODO: Should be accessing formatter data but I see no method for it.
            } else {
                string.push_str("+"); // TODO: Should be accessing formatter data but I see no method for it.
            }
            string.push_str(&formatter.format(&magnitude_decimal).to_string());

            string
        }).into()
    }

    fn as_decimal(&self) -> FixedDecimal {
        let minimum_integer_digits = self.options.minimum_integer_digits.unwrap_or(1);
        let minimum_fraction_digits = self.options.minimum_fraction_digits.unwrap_or(0);
        let maximum_fraction_digits = self.options.maximum_fraction_digits.unwrap_or(minimum_fraction_digits.max(3)) as i16;

        let f3 = FixedDecimal::from_str(&self.value.to_string())
            .expect("That the f64 value when formatted as a string is convertable to a fixed decimal")
            .padded_start(minimum_integer_digits as i16);

        let f4 = match self.options.rounding_mode {
            FluentNumberRoundingMode::Ceil => f3.ceiled(-maximum_fraction_digits),
            FluentNumberRoundingMode::Floor => f3.floored(-maximum_fraction_digits),
            FluentNumberRoundingMode::Expand => f3.expanded(-maximum_fraction_digits),
            FluentNumberRoundingMode::Trunc => f3.trunced(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfCeil => f3.half_ceiled(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfFloor => f3.half_floored(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfExpand => f3.half_expanded(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfTrunc => f3.half_trunced(-maximum_fraction_digits),
            FluentNumberRoundingMode::HalfEven => f3.half_evened(-maximum_fraction_digits),
        };

        f4.trimmed_end().padded_end(-(minimum_fraction_digits as i16))
    }
}

impl FromStr for FluentNumber {
    type Err = std::num::ParseFloatError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        f64::from_str(input).map(|n| {
            let mfd = input.find('.').map(|pos| input.len() - pos - 1);
            let opts = FluentNumberOptions {
                minimum_fraction_digits: mfd,
                ..Default::default()
            };
            Self::new(n, opts)
        })
    }
}

impl<'l> From<FluentNumber> for FluentValue<'l> {
    fn from(input: FluentNumber) -> Self {
        FluentValue::Number(input)
    }
}

macro_rules! from_num {
    ($num:ty) => {
        impl From<$num> for FluentNumber {
            fn from(n: $num) -> Self {
                Self {
                    value: n as f64,
                    options: FluentNumberOptions::default(),
                }
            }
        }
        impl From<&$num> for FluentNumber {
            fn from(n: &$num) -> Self {
                Self {
                    value: *n as f64,
                    options: FluentNumberOptions::default(),
                }
            }
        }
        impl From<FluentNumber> for $num {
            fn from(input: FluentNumber) -> Self {
                input.value as $num
            }
        }
        impl From<&FluentNumber> for $num {
            fn from(input: &FluentNumber) -> Self {
                input.value as $num
            }
        }
        impl From<$num> for FluentValue<'_> {
            fn from(n: $num) -> Self {
                FluentValue::Number(n.into())
            }
        }
        impl From<&$num> for FluentValue<'_> {
            fn from(n: &$num) -> Self {
                FluentValue::Number(n.into())
            }
        }
    };
    ($($num:ty)+) => {
        $(from_num!($num);)+
    };
}

impl From<&FluentNumber> for icu::plurals::PluralOperands {
    fn from(input: &FluentNumber) -> Self {
        icu::plurals::PluralOperands::from(&input.as_decimal()) // TODO this does not allow to handle trailing zeros
    }
}

from_num!(i8 i16 i32 i64 i128 isize);
from_num!(u8 u16 u32 u64 u128 usize);
from_num!(f32 f64);

#[cfg(test)]
mod tests {
    use crate::types::FluentValue;

    #[test]
    fn value_from_copy_ref() {
        let x = 1i16;
        let y = &x;
        let z: FluentValue = y.into();
        assert_eq!(z, FluentValue::try_number("1"));
    }
}
