const APP: &str = include_str!("../web/app.js");
const CAPTIONS: &str = include_str!("../web/captions.html");

fn function_body<'a>(source: &'a str, name: &str) -> &'a str {
    let start = source
        .find(&format!("function {name}("))
        .unwrap_or_else(|| panic!("{name} is missing"));
    let end = source[start..]
        .find("\n}\n")
        .map(|offset| start + offset)
        .unwrap_or(source.len());
    &source[start..end]
}

mod tests {
    use super::*;

    #[test]
    fn every_control_the_app_binds_is_on_the_page() {
        let page = include_str!("../web/index.html");
        for id in [
            "settings-compute",
            "compute-note",
            "voice-import",
            "voice-import-file",
            "voice-import-note",
            "compare-run",
            "compare-text",
            "compare-first",
            "compare-second",
            "compare-note",
            "compare-readings",
            "pack-name",
            "pack-save",
            "pack-import",
            "pack-import-file",
            "pack-note",
            "pack-list",
        ] {
            assert!(
                page.contains(&format!("id=\"{id}\"")),
                "{id} is missing from the page"
            );
            assert!(
                APP.contains(&format!("getElementById(\"{id}\")")),
                "{id} is not read by the script"
            );
        }
    }

    #[test]
    fn the_meeting_screen_redraws_the_lines_while_it_records() {
        let body = function_body(APP, "meetingStep");
        assert!(body.contains("await loadMeetings()"), "{body}");
        assert!(function_body(APP, "stopMeeting").contains("loadMeetings()"));
    }

    #[test]
    fn the_write_languages_follow_the_packs_both_places_offer() {
        let table: serde_json::Value =
            serde_json::from_str(include_str!("../web/languages.json")).expect("a language table");
        let names = table["names"].as_object().expect("named languages");
        let pairs = table["pairs"].as_array().expect("language pairs");
        let registry = include_str!("registry.rs");
        for pair in pairs {
            let from = pair[0].as_str().expect("a source language");
            let to = pair[1].as_str().expect("a target language");
            assert!(
                names.contains_key(from) && names.contains_key(to),
                "{from}->{to} has no name"
            );
            assert!(
                crate::config::check_language(from) && crate::config::check_language(to),
                "{from}->{to} is not a pair of two letter codes"
            );
            assert!(
                registry.contains(&format!("id: \"mt-{from}-{to}\"")),
                "{from}->{to} has no translation pack"
            );
        }
        for line in registry
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("id: \"mt-"))
        {
            let id = line.split('"').nth(1).expect("a pack id");
            assert!(
                pairs.iter().any(|pair| pair[0].as_str() == Some(&id[3..5])
                    && pair[1].as_str() == Some(&id[6..8])),
                "{id} is missing from the language table"
            );
        }
        let island = include_str!("../web/island.html");
        assert!(
            island.contains("languages.json"),
            "the overlay carries its own list"
        );
        assert!(
            !island.contains("<option value=\"es\">"),
            "the overlay went back to a hardcoded list"
        );
        assert!(
            APP.contains("languages.json"),
            "the app carries its own list"
        );
    }

    #[test]
    fn the_captions_page_offers_the_computer_s_sound() {
        assert!(CAPTIONS.contains("getDisplayMedia"));
        assert!(CAPTIONS.contains("id=\"source\""));
    }

    #[test]
    fn every_settings_tab_shows_panes_that_exist() {
        let page = include_str!("../web/index.html");
        let start = APP
            .find("const SETTINGS_TABS = {")
            .expect("the settings tabs are declared");
        let block = &APP[start
            ..start
                + APP[start..]
                    .find(
                        "};
",
                    )
                    .unwrap()];
        let mut tabs = 0;
        for line in block.lines().filter(|line| line.contains('[')) {
            let name = line.split(':').next().unwrap().trim();
            tabs += 1;
            assert!(
                page.contains(&format!("data-tab=\"{name}\"")),
                "the {name} tab has no button"
            );
            for id in line.split('"').skip(1).step_by(2) {
                assert!(
                    page.contains(&format!("id=\"{id}\"")),
                    "the {name} tab shows {id}, which is not on the page"
                );
            }
        }
        assert_eq!(tabs, 5, "the app keeps five settings tabs");
    }
}
