// use intl_memoizer::{IntlMemoizer, Memoizable};
// use intl_pluralrules::{PluralCategory, PluralRuleType, PluralRules as IntlPluralRules};
// use icu::locid::Locale;
//
// struct PluralRules(pub IntlPluralRules);
//
// impl PluralRules {
//     pub fn new(lang: Locale, pr_type: PluralRuleType) -> Result<Self, &'static str> {
//         Ok(Self(IntlPluralRules::create(lang, pr_type)?))
//     }
// }
//
// impl Memoizable for PluralRules {
//     type Args = (PluralRuleType,);
//     type Error = &'static str;
//     fn construct(lang: Locale, args: Self::Args) -> Result<Self, Self::Error> {
//         Self::new(lang, args.0)
//     }
// }
//
// fn main() {
//     let mut memoizer = IntlMemoizer::default();
//
//     let lang: Locale = "en".parse().unwrap();
//     let lang_memoizer = memoizer.get_for_lang(lang);
//     let result = lang_memoizer
//         .with_try_get::<PluralRules, _, _>((PluralRuleType::CARDINAL,), |pr| pr.0.select(5))
//         .unwrap();
//
//     assert_eq!(result, Ok(PluralCategory::OTHER));
// }

fn main() {

}
