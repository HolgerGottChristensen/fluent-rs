use intl_memoizer_for_carbide::{concurrent::IntlLangMemoizer, Memoizable};
use rustc_hash::FxHashMap;
use icu::locid::Locale;
use crate::FluentValue;

use crate::memoizer::MemoizerKind;
use crate::types::FluentType;

/// Specialized [`FluentBundle`](crate::bundle::FluentBundle) over
/// concurrent [`IntlLangMemoizer`](intl_memoizer::concurrent::IntlLangMemoizer).
///
/// A concurrent `FluentBundle` can be constructed with the
/// [`FluentBundle::new_concurrent`] method.
///
/// See [`FluentBundle`](crate::FluentBundle) for the non-concurrent specialization.
pub type FluentBundle<R> = crate::bundle::FluentBundle<R, IntlLangMemoizer>;

impl<R> FluentBundle<R> {
    /// A constructor analogous to [`FluentBundle::new`] but operating
    /// on a concurrent version of [`IntlLangMemoizer`] over [`Mutex`](std::sync::Mutex).
    ///
    /// # Example
    ///
    /// ```
    /// use fluent_bundle::concurrent::FluentBundle;
    /// use fluent_bundle::FluentResource;
    /// use icu::locid::locale;
    ///
    /// let locale_en = locale!("en-US");
    /// let mut bundle: FluentBundle<FluentResource> =
    ///     FluentBundle::new_concurrent(vec![locale_en]);
    /// ```
    pub fn new_concurrent(locales: Vec<Locale>) -> Self {
        let first_locale = locales.get(0).cloned().unwrap_or_default();
        let mut res = Self {
            locales,
            resources: vec![],
            entries: FxHashMap::default(),
            intls: IntlLangMemoizer::new(first_locale),
            use_isolating: true,
            transform: None,
            formatter: None,
        };

        res.add_function("NUMBER", |args, named_args| {
            if args.len() != 1 {
                return FluentValue::Error
            }

            let arg = args[0].clone();

            //println!("NUMBER CALLED: {:#?}", arg);
            //println!("NUMBER NEW: {:#?}", named_args);

            let res = match arg {
                FluentValue::Number(mut num) => {
                    num.options.merge(named_args);
                    FluentValue::Number(num)
                }
                _ => FluentValue::Error
            };

            //println!("NUMBER RES: {:#?}", res);

            res
        }).unwrap();

        res.add_function("DATETIME", |args, named_args| {
            if args.len() != 1 {
                return FluentValue::Error
            }

            let arg = args[0].clone();

            let res = match arg {
                FluentValue::DateTime(mut dt) => {
                    dt.options.merge(named_args);
                    FluentValue::DateTime(dt)
                }
                _ => FluentValue::Error
            };

            res
        }).unwrap();

        res
    }
}

impl MemoizerKind for IntlLangMemoizer {
    fn new(lang: Locale) -> Self
        where
            Self: Sized,
    {
        Self::new(lang)
    }

    fn with_try_get_threadsafe<I, R, U>(&self, args: I::Args, cb: U) -> Result<R, I::Error>
        where
            Self: Sized,
            I: Memoizable + Send + Sync + 'static,
            I::Args: Send + Sync + 'static,
            U: FnOnce(&I) -> R,
    {
        self.with_try_get(args, cb)
    }

    fn stringify_value(&self, value: &dyn FluentType) -> std::borrow::Cow<'static, str> {
        value.as_string_threadsafe(self)
    }

    fn language(&self) -> &Locale {
        &self.lang()
    }
}