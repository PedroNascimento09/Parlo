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
    fn the_meeting_screen_redraws_the_lines_while_it_records() {
        let body = function_body(APP, "meetingStep");
        assert!(body.contains("await loadMeetings()"), "{body}");
        assert!(function_body(APP, "stopMeeting").contains("loadMeetings()"));
    }

    #[test]
    fn the_captions_page_offers_the_computer_s_sound() {
        assert!(CAPTIONS.contains("getDisplayMedia"));
        assert!(CAPTIONS.contains("id=\"source\""));
    }
}
