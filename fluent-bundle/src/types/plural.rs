use std::cell::RefCell;
use std::collections::HashMap;
use icu::locid::Locale;
use icu::plurals::{PluralCategory, PluralOperands, PluralRules, PluralRuleType};

thread_local! {
    // Ordinal, Cardinal
    static PLURALS: RefCell<HashMap<Locale, (PluralRules, PluralRules)>> = RefCell::new(HashMap::new());
}

pub fn plural_category<I: Into<PluralOperands>>(locale: &Locale, plural_rule_type: PluralRuleType, input: I) -> PluralCategory {
    PLURALS.with(|cell| {
        if let Some((ordinal, cardinal)) = cell.borrow().get(locale) {
            return match plural_rule_type {
                PluralRuleType::Cardinal => cardinal.category_for(input),
                PluralRuleType::Ordinal => ordinal.category_for(input),
                _ => panic!("New plural rule type that should be implemented")
            };
        }

        let ordinal = PluralRules::try_new(&locale.into(), PluralRuleType::Ordinal).unwrap();
        let cardinal = PluralRules::try_new(&locale.into(), PluralRuleType::Cardinal).unwrap();

        let res = match plural_rule_type {
            PluralRuleType::Cardinal => cardinal.category_for(input),
            PluralRuleType::Ordinal => ordinal.category_for(input),
            _ => panic!("New plural rule type that should be implemented")
        };

        cell.borrow_mut().insert(locale.clone(), (ordinal, cardinal));

        res
    })
}
