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
    fn the_overlay_writes_in_every_language_the_models_carry() {
        let page = include_str!("../web/island.html");
        assert!(page.contains("<option value=\"\">Spoken language</option>"));
        for code in ["en", "pt", "es", "fr", "de", "it", "nl", "ru"] {
            assert!(
                page.contains(&format!("<option value=\"{code}\">")),
                "the overlay cannot write in {code}"
            );
        }
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
