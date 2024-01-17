use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use chrono::{Datelike, DateTime, FixedOffset, Timelike};
use icu::calendar::{AnyCalendar, Date};
use icu::datetime::{DateFormatter, DateTimeFormatter, DateTimeFormatterOptions, TimeFormatter, ZonedDateTimeFormatter};
use icu::datetime::input::{DateInput, IsoTimeInput};
use icu::datetime::options::length;
use icu::datetime::options::length::Time;
use icu::datetime::time_zone::{FallbackFormat, TimeZoneFormatter, TimeZoneFormatterOptions};
use icu::locid::Locale;
use icu::timezone::CustomTimeZone;
use crate::{FluentArgs, FluentValue};
use crate::types::IsoFormat::{Basic, Extended, UtcBasic, UtcExtended};
use crate::types::IsoMinutes::Required;
use crate::types::IsoSeconds::Optional;

#[derive(Debug, PartialEq, Clone)]
pub struct FluentDateTime {
    pub value: DateTime<FixedOffset>,
    pub options: FluentDateTimeOptions,
}

enum Formatter {
    Date(DateFormatter),
    DateTime(DateTimeFormatter),
    Time(TimeFormatter),
    ZonedDateTime(ZonedDateTimeFormatter),
    TimeZone(TimeZoneFormatter),
}

impl Formatter {
    fn format_string<T: DateInput<Calendar = AnyCalendar> + IsoTimeInput>(&self, date: &T, zone: &CustomTimeZone) -> String {
        match self {
            Formatter::Date(d) => d.format_to_string(date).unwrap(),
            Formatter::DateTime(d) => d.format_to_string(date).unwrap(),
            Formatter::Time(d) => d.format_to_string(date),
            Formatter::ZonedDateTime(d) => d.format_to_string(date, zone).unwrap(),
            Formatter::TimeZone(d) => d.format_to_string(zone),
        }
    }

    fn new(locale: &Locale, date: Option<length::Date>, time: Option<length::Time>, zone: Option<FallbackFormat>) -> Option<Formatter> {
        match (date, time, zone) {
            (None, None, None) => None,
            (Some(date_style), None, None) => Some(Formatter::Date(DateFormatter::try_new_with_length(&locale.into(), date_style).expect("Failed to create DateFormatter instance."))),
            (Some(date_style), Some(time_style), None) => {
                let time_style = match time_style {
                    Time::Full |
                    Time::Long |
                    Time::Medium => Time::Medium,
                    Time::Short => Time::Short,
                    _ => unimplemented!()
                };

                let options =
                    DateTimeFormatterOptions::Length(length::Bag::from_date_time_style(
                        date_style,
                        time_style,
                    ));

                let dtf = DateTimeFormatter::try_new(&locale.into(), options.clone())
                    .expect("Failed to create DateTimeFormatter instance.");

                Some(Formatter::DateTime(dtf))
            }
            (Some(date_style), Some(time_style), Some(timezone_style)) => {
                let timezone_options =
                    TimeZoneFormatterOptions::from(timezone_style);

                let options =
                    DateTimeFormatterOptions::Length(length::Bag::from_date_time_style(
                        date_style,
                        time_style,
                    ));

                let dtf = ZonedDateTimeFormatter::try_new(&locale.into(), options, timezone_options)
                    .expect("Failed to create ZonedDateTimeFormatter instance.");

                Some(Formatter::ZonedDateTime(dtf))
            }
            (None, Some(time_format), None) => {
                let time_format = match time_format {
                    Time::Full |
                    Time::Long |
                    Time::Medium => Time::Medium,
                    Time::Short => Time::Short,
                    _ => unimplemented!()
                };

                let dtf = TimeFormatter::try_new_with_length(&locale.into(), time_format)
                    .expect("Failed to create TimeFormatter instance.");

                Some(Formatter::Time(dtf))
            }
            (None, None, Some(timezone_style)) => {
                let dtf = TimeZoneFormatter::try_new(&locale.into(), TimeZoneFormatterOptions::from(timezone_style))
                    .expect("Failed to create TimeFormatter instance.");

                Some(Formatter::TimeZone(dtf))
            }
            _ => None,
        }
    }
}

thread_local! {
    static FORMATTERS: RefCell<HashMap<Locale, HashMap<(FluentDateStyle, FluentTimeStyle, FluentTimezoneStyle), Formatter>>> = RefCell::new(HashMap::new());
}

impl FluentDateTime {
    pub fn as_string(&self, locale: &Locale) -> Cow<'static, str> {
        let typed_date = icu::calendar::DateTime::try_new_gregorian_datetime(
            self.value.year(),
            self.value.month() as u8,
            self.value.day() as u8,
            self.value.hour() as u8,
            self.value.minute() as u8,
            self.value.second() as u8
        ).unwrap();

        let date = typed_date.to_iso().to_any();
        let time_zone = CustomTimeZone::from_str(&self.value.timezone().to_string()).unwrap();

        let date_style = match self.options.date_style {
            FluentDateStyle::Full => Some(length::Date::Full),
            FluentDateStyle::Long => Some(length::Date::Long),
            FluentDateStyle::Medium => Some(length::Date::Medium),
            FluentDateStyle::Short => Some(length::Date::Short),
            FluentDateStyle::Hidden => None,
        };

        let time_style = match self.options.time_style {
            FluentTimeStyle::Full => Some(Time::Full),
            FluentTimeStyle::Long => Some(Time::Long),
            FluentTimeStyle::Medium => Some(Time::Medium),
            FluentTimeStyle::Short => Some(Time::Short),
            FluentTimeStyle::Hidden => None,
        };

        let timezone_style = match self.options.timezone_style {
            FluentTimezoneStyle::Hidden => None,
            FluentTimezoneStyle::LocalizedGmt => Some(FallbackFormat::LocalizedGmt),
            FluentTimezoneStyle::Iso8601(a, b, c) => {
                let a = match a {
                    IsoFormat::Basic => icu::datetime::time_zone::IsoFormat::Basic,
                    IsoFormat::Extended => icu::datetime::time_zone::IsoFormat::Extended,
                    IsoFormat::UtcBasic => icu::datetime::time_zone::IsoFormat::UtcBasic,
                    IsoFormat::UtcExtended => icu::datetime::time_zone::IsoFormat::UtcExtended,
                };

                let b = match b {
                    IsoMinutes::Required => icu::datetime::time_zone::IsoMinutes::Required,
                    IsoMinutes::Optional => icu::datetime::time_zone::IsoMinutes::Optional,
                };

                let c = match c {
                    IsoSeconds::Optional => icu::datetime::time_zone::IsoSeconds::Optional,
                    IsoSeconds::Never => icu::datetime::time_zone::IsoSeconds::Never,
                };

                Some(FallbackFormat::Iso8601(a, b, c))
            }
        };

        FORMATTERS.with(|cell| {
            if let Some(formatter_map) = cell.borrow_mut().get_mut(locale) {
                if let Some(formatter) = formatter_map.get_mut(&(self.options.date_style, self.options.time_style, self.options.timezone_style)) {
                    return formatter.format_string(&date, &time_zone).into();
                }

                let new_formatter = Formatter::new(locale, date_style, time_style, timezone_style).unwrap();

                let res = new_formatter.format_string(&date, &time_zone).into();

                formatter_map.insert((self.options.date_style, self.options.time_style, self.options.timezone_style), new_formatter);

                return res;
            }

            let mut map = HashMap::new();

            let new_formatter = Formatter::new(locale, date_style, time_style, timezone_style).unwrap();

            let res = new_formatter.format_string(&date, &time_zone).into();

            map.insert((self.options.date_style, self.options.time_style, self.options.timezone_style), new_formatter);

            cell.borrow_mut().insert(locale.clone(), map);

            res
        })
    }
}

impl<'l> From<FluentDateTime> for FluentValue<'l> {
    fn from(input: FluentDateTime) -> Self {
        FluentValue::DateTime(input)
    }
}

impl From<DateTime<FixedOffset>> for FluentDateTime {
    fn from(value: DateTime<FixedOffset>) -> Self {
        FluentDateTime {
            value,
            options: Default::default(),
        }
    }
}


/// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/NumberFormat#locale_options
#[derive(Debug, Clone, Hash, PartialEq, Eq, Default)]
pub struct FluentDateTimeOptions {
    pub date_style: FluentDateStyle,
    pub time_style: FluentTimeStyle,
    pub timezone_style: FluentTimezoneStyle,
}

impl FluentDateTimeOptions {
    pub fn merge(&mut self, opts: &FluentArgs) {
        for (key, value) in opts.iter() {
            match (key, value) {
                ("dateStyle", FluentValue::String(n)) => {
                    self.date_style = n.as_ref().into();
                }
                ("timeStyle", FluentValue::String(n)) => {
                    self.time_style = n.as_ref().into();
                }
                ("timezoneStyle", FluentValue::String(n)) => {
                    self.timezone_style = n.as_ref().into();
                }
                _ => {}
            }
        }
    }
}

// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/DateTimeFormat#datestyle
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FluentDateStyle {
    Full,
    Long,
    Medium,
    Short,

    /// Hides the date
    Hidden,
}

impl std::default::Default for FluentDateStyle {
    fn default() -> Self {
        Self::Medium
    }
}

impl From<&str> for FluentDateStyle {
    fn from(input: &str) -> Self {
        match input {
            "full" => Self::Full,
            "long" => Self::Long,
            "medium" => Self::Medium,
            "short" => Self::Short,
            "hidden" => Self::Hidden,
            _ => Self::default(),
        }
    }
}


// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/DateTimeFormat#timestyle
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FluentTimeStyle {
    Full,
    Long,
    Medium,
    Short,

    /// Hides the time
    Hidden,
}

impl std::default::Default for FluentTimeStyle {
    fn default() -> Self {
        Self::Medium
    }
}

impl From<&str> for FluentTimeStyle {
    fn from(input: &str) -> Self {
        match input {
            "full" => Self::Full,
            "long" => Self::Long,
            "medium" => Self::Medium,
            "short" => Self::Short,
            "hidden" => Self::Hidden,
            _ => Self::default(),
        }
    }
}


// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/DateTimeFormat/DateTimeFormat#timestyle
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FluentTimezoneStyle {
    LocalizedGmt,
    Iso8601(IsoFormat, IsoMinutes, IsoSeconds),
    Hidden,
}

impl Default for FluentTimezoneStyle {
    fn default() -> Self {
        Self::Hidden
    }
}

impl From<&str> for FluentTimezoneStyle {
    fn from(input: &str) -> Self {
        match input {
            "gmt" => Self::LocalizedGmt,
            "basic" => Self::Iso8601(Basic, Required, Optional),
            "extended" => Self::Iso8601(Extended, Required, Optional),
            "utcBasic" => Self::Iso8601(UtcBasic, Required, Optional),
            "utcExtended" => Self::Iso8601(UtcExtended, Required, Optional),
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum IsoFormat {
    Basic,
    Extended,
    UtcBasic,
    UtcExtended,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum IsoMinutes {
    Required,
    Optional,
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum IsoSeconds {
    Optional,
    Never,
}