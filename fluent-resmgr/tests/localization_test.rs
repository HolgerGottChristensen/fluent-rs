use fluent_fallback::Localization;
use fluent_resmgr::resource_manager::ResourceManager;
use std::borrow::Cow;
use icu::locid::locale;

#[test]
fn localization_format_value() {
    let res_mgr = ResourceManager::new("./tests/resources/{locale}/{res_id}".into());

    let loc = Localization::with_env(
        vec!["test.ftl".into()],
        true,
        vec!["en-US".parse().unwrap(), "pl".parse().unwrap()],
        res_mgr,
    );
    let bundles = loc.bundles();
    let mut errors = vec![];

    let value = bundles
        .format_value_sync("hello-world", None, &mut errors)
        .unwrap();
    assert_eq!(value, Some(Cow::Borrowed("Hello World")));

    let value2 = bundles
        .format_value_sync("new-message", None, &mut errors)
        .unwrap();
    assert_eq!(value2, Some(Cow::Borrowed("Nowa Wiadomość")));

    let value3 = bundles
        .format_value_sync("missing-message", None, &mut errors)
        .unwrap();
    assert_eq!(value3, None);
}

#[test]
fn resmgr_get_bundle() {
    let res_mgr = ResourceManager::new("./tests/resources/{locale}/{res_id}".into());

    let bundle = res_mgr
        .get_bundle(vec![locale!("en-US")], vec!["test.ftl".into()])
        .expect("Could not get bundle");

    let mut errors = vec![];
    let msg = bundle.get_message("hello-world").expect("Message exists");
    let pattern = msg.value().expect("Message has a value");
    let value = bundle.format_pattern(pattern, None, &mut errors);
    assert_eq!(value, "Hello World");
}

#[test]
fn resmgr_get_bundles() {
    let res_mgr = ResourceManager::new("./tests/resources/{locale}/{res_id}".into());

    let locales = vec![locale!("en-US"), locale!("pl")];
    let mut bundles_iter = res_mgr.get_bundles(locales, vec!["test.ftl".into()]);

    {
        let bundle = bundles_iter
            .next()
            .unwrap()
            .expect("Failed to get en-US bundle.");

        let mut errors = vec![];
        let msg = bundle.get_message("hello-world").expect("Message exists");
        let pattern = msg.value().expect("Message has a value");
        let value = bundle.format_pattern(pattern, None, &mut errors);
        assert_eq!(value, "Hello World");
    }

    {
        let bundle = bundles_iter
            .next()
            .unwrap()
            .expect("Failed to get pl bundle.");

        let mut errors = vec![];
        let msg = bundle.get_message("hello-world").expect("Witaj Świecie");
        let pattern = msg.value().expect("Message has a value");
        let value = bundle.format_pattern(pattern, None, &mut errors);
        assert_eq!(value, "Witaj Świecie");
    }

    assert!(bundles_iter.next().is_none(), "The iterator is consumed.");
}
