mod empty_tests {
    use super::*;

    #[test]
    fn a_dictation_without_words_is_not_kept() {
        assert!(!worth_keeping(&Entry::now("   ", "", "pt", "pt", "", "")));
        assert!(worth_keeping(&Entry::now("Olá.", "", "pt", "pt", "", "")));
    }
}

use super::*;

fn scratch(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();
    let folder = std::env::temp_dir().join(format!("parlo-history-{name}-{nanos}"));
    fs::create_dir_all(&folder).unwrap();
    folder.join("history.jsonl")
}

fn tidy(path: &Path) {
    if let Some(parent) = path.parent() {
        let _ = fs::remove_dir_all(parent);
    }
}

fn kept(path: &Path, text: &str) {
    append_to(
        path,
        &Entry::now(text, text, "pt", "en", "Notepad", "dictate"),
    )
    .unwrap();
}

#[test]
fn keeps_the_newest_first() {
    let path = scratch("order");
    kept(&path, "bom dia");
    kept(&path, "até logo");
    let recent = recent_at(&path, 10);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].translation, "até logo");
    assert_eq!(recent[1].text, "bom dia");
    tidy(&path);
}

#[test]
fn a_limit_returns_only_what_was_asked() {
    let path = scratch("limit");
    for index in 0..5 {
        kept(&path, &format!("frase {index}"));
    }
    let recent = recent_at(&path, 2);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].text, "frase 4");
    tidy(&path);
}

#[test]
fn the_store_keeps_a_bound() {
    let path = scratch("bound");
    for index in 0..(LIMIT + 5) {
        kept(&path, &format!("linha {index}"));
    }
    let entries = entries_at(&path);
    assert_eq!(entries.len(), LIMIT);
    assert_eq!(entries[LIMIT - 1].text, format!("linha {}", LIMIT + 4));
    tidy(&path);
}

#[test]
fn a_damaged_line_never_hides_the_rest() {
    let path = scratch("damaged");
    kept(&path, "primeira");
    let mut file = fs::read_to_string(&path).unwrap();
    file.push_str("isto não é json\n");
    fs::write(&path, file).unwrap();
    kept(&path, "segunda");
    let recent = recent_at(&path, 10);
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].text, "segunda");
    tidy(&path);
}

#[test]
fn clearing_removes_the_file_and_tolerates_a_missing_one() {
    let path = scratch("clear");
    kept(&path, "apagar");
    assert!(path.exists());
    clear_at(&path).unwrap();
    assert!(!path.exists());
    assert!(clear_at(&path).is_ok());
    tidy(&path);
}

#[test]
fn every_entry_carries_its_own_id() {
    let path = scratch("ids");
    kept(&path, "primeira");
    kept(&path, "segunda");
    let entries = entries_at(&path);
    assert_ne!(entries[0].id, entries[1].id);
    assert!(!entries[0].favourite);
    tidy(&path);
}

#[test]
fn a_favourite_survives_a_reload() {
    let path = scratch("favourite");
    kept(&path, "guardar");
    let id = entries_at(&path)[0].id.clone();
    with_entry(&path, &id, |entry| entry.favourite = true).unwrap();
    assert!(entries_at(&path)[0].favourite);
    tidy(&path);
}

#[test]
fn a_correction_replaces_the_heard_sentence() {
    let path = scratch("edit");
    kept(&path, "errado");
    let id = entries_at(&path)[0].id.clone();
    with_entry(&path, &id, |entry| entry.text = "certo".to_string()).unwrap();
    assert_eq!(entries_at(&path)[0].text, "certo");
    tidy(&path);
}

#[test]
fn an_empty_correction_is_refused() {
    assert!(edit_with_suggestions("qualquer", "   ").is_err());
}

#[test]
fn removing_one_keeps_the_rest() {
    let path = scratch("remove");
    kept(&path, "primeira");
    kept(&path, "segunda");
    let entries = entries_at(&path);
    let target = entries[1].id.clone();
    let mut remaining = entries_at(&path);
    remaining.retain(|entry| entry.id != target);
    write_entries(&path, &remaining).unwrap();
    let left = entries_at(&path);
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].text, "primeira");
    tidy(&path);
}

#[test]
fn a_line_written_before_ids_existed_still_answers() {
    let path = scratch("legacy");
    fs::write(
        &path,
        "{\"at\":1,\"text\":\"antiga\",\"translation\":\"old\",\"from\":\"pt\",\"to\":\"en\",\"app\":\"\",\"mode\":\"\"}\n",
    )
    .unwrap();
    assert_eq!(entries_at(&path)[0].id, "e0");
    let entries = entries_at(&path);
    let id = entries[0].id.clone();
    with_entry(&path, &id, |entry| entry.favourite = true).unwrap();
    assert!(entries_at(&path)[0].favourite);
    tidy(&path);
}

#[test]
fn an_unknown_id_is_refused() {
    let path = scratch("unknown");
    assert!(with_entry(&path, "nada", |_| {}).is_err());
    tidy(&path);
}

#[test]
fn a_search_finds_a_word_and_ignores_accents() {
    let path = scratch("search");
    kept(&path, "o relatório ficou pronto");
    kept(&path, "a reunião começa às dez");
    assert_eq!(search_at(&path, "relatorio", 10).len(), 1);
    assert_eq!(search_at(&path, "reuniao", 10).len(), 1);
    assert_eq!(search_at(&path, "nada", 10).len(), 0);
    tidy(&path);
}

#[test]
fn a_search_reads_the_program_and_the_translation() {
    let path = scratch("search-app");
    append_to(
        &path,
        &Entry::now("bom dia", "good morning", "pt", "en", "Notepad", "dictate"),
    )
    .unwrap();
    assert_eq!(search_at(&path, "notepad", 10).len(), 1);
    assert_eq!(search_at(&path, "good morning", 10).len(), 1);
    tidy(&path);
}

#[test]
fn a_search_keeps_the_newest_first_and_respects_the_limit() {
    let path = scratch("search-order");
    kept(&path, "alpha one");
    kept(&path, "alpha two");
    let hits = search_at(&path, "alpha", 1);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].text, "alpha two");
    tidy(&path);
}

#[test]
fn a_restyle_rewrites_the_sentence_in_place() {
    let path = scratch("restyle");
    kept(&path, "bom dia como estas");
    let id = entries_at(&path)[0].id.clone();
    let changed = restyle_at(&path, &id, "formal").unwrap();
    assert_eq!(changed.text, "Bom dia como estas.");
    assert_eq!(entries_at(&path)[0].text, "Bom dia como estas.");
    tidy(&path);
}

#[test]
fn a_correction_of_one_word_proposes_that_word() {
    let proposed = suggestions("meet me on teusday", "meet me on Tuesday");
    assert_eq!(
        proposed,
        vec![("teusday".to_string(), "Tuesday".to_string())]
    );
}

#[test]
fn an_unchanged_word_proposes_nothing() {
    assert!(suggestions("bom dia", "bom dia").is_empty());
    assert!(suggestions("bom dia", "Bom dia.").is_empty());
}

#[test]
fn a_whole_rewrite_proposes_nothing() {
    assert!(suggestions("one two three four five", "six seven eight nine ten").is_empty());
}

#[test]
fn short_words_are_never_proposed() {
    assert!(suggestions("at it", "to it").is_empty());
}

#[test]
fn an_edit_returns_the_suggestions_it_found() {
    let path = scratch("learn");
    kept(&path, "meet me on teusday");
    let id = entries_at(&path)[0].id.clone();
    let heard = entries_at(&path)[0].text.clone();
    let proposed = suggestions(&heard, "meet me on Tuesday");
    assert_eq!(proposed.len(), 1);
    with_entry(&path, &id, |entry| {
        entry.text = "meet me on Tuesday".to_string()
    })
    .unwrap();
    assert_eq!(entries_at(&path)[0].text, "meet me on Tuesday");
    tidy(&path);
}

#[test]
fn an_export_names_the_day_the_program_and_the_sentence() {
    let path = scratch("export");
    let mut entry = Entry::now("olá", "hello", "pt", "en", "Slack", "dictate");
    entry.at = 1_700_000_000;
    append_to(&path, &entry).unwrap();
    let text = markdown(&entries_at(&path));
    assert!(text.starts_with("# Parlo dictations\n\n- **2023-11-14 22:13** · Slack"));
    assert!(text.contains("pt → en"));
    assert!(text.contains("\n  olá\n"));
    tidy(&path);
}

#[test]
fn an_empty_export_says_so() {
    assert_eq!(markdown(&[]), "No dictations yet.\n");
}

#[test]
fn a_line_older_than_the_window_is_removed() {
    let path = scratch("prune");
    let mut old = Entry::now("velha", "old", "pt", "en", "", "dictate");
    old.at = 10 * SECONDS_PER_DAY;
    append_to(&path, &old).unwrap();
    let mut fresh = Entry::now("nova", "new", "pt", "en", "", "dictate");
    fresh.at = 99 * SECONDS_PER_DAY;
    append_to(&path, &fresh).unwrap();
    assert_eq!(prune_at(&path, 30, 100 * SECONDS_PER_DAY), Ok(1));
    let left = entries_at(&path);
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].text, "nova");
    tidy(&path);
}

#[test]
fn a_window_of_no_days_keeps_every_line() {
    let path = scratch("prune-off");
    let mut old = Entry::now("velha", "old", "pt", "en", "", "dictate");
    old.at = 1;
    append_to(&path, &old).unwrap();
    assert_eq!(prune_at(&path, 0, 100 * SECONDS_PER_DAY), Ok(0));
    assert_eq!(entries_at(&path).len(), 1);
    tidy(&path);
}

#[test]
fn an_empty_search_returns_the_recent_lines() {
    let path = scratch("search-empty");
    kept(&path, "uma");
    assert_eq!(search_at(&path, "", 10).len(), 1);
    tidy(&path);
}
