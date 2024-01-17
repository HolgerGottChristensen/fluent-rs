use icu::locid::Locale;

static REGION_MATCHING_KEYS: &[&str] = &[
    "az", "bg", "cs", "de", "es", "fi", "fr", "hu", "it", "lt", "lv", "nl", "pl", "ro", "ru",
];

pub trait MockLikelySubtags {
    fn maximize(&mut self) -> bool;
}

impl MockLikelySubtags for Locale {
    fn maximize(&mut self) -> bool {
        let extended = match self.to_string().as_str() {
            "en" => "en-Latn-US",
            "fr" => "fr-Latn-FR",
            "sr" => "sr-Cyrl-SR",
            "sr-RU" => "sr-Latn-SR",
            "az-IR" => "az-Arab-IR",
            "zh-GB" => "zh-Hant-GB",
            "zh-US" => "zh-Hant-US",
            _ => {
                let lang = self.id.language;

                for subtag in REGION_MATCHING_KEYS {
                    if lang.as_str() == *subtag {
                        self.id.region = Some(subtag.parse().unwrap());
                        return true;
                    }
                }
                return false;
            }
        };
        let langid: Locale = extended.parse().expect("Failed to parse langid.");
        self.id.language = langid.id.language;
        self.id.script = langid.id.script;
        self.id.region = langid.id.region;
        true
    }
}