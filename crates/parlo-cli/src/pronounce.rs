use parlo_core::pronounce::{entries, lookup, respell};

pub fn command(args: &[String]) -> Result<(), String> {
    match args {
        [] => {
            for entry in entries() {
                print(entry);
            }
            Ok(())
        }
        [flag, text] if flag == "--text" => {
            println!("{}", respell(text));
            Ok(())
        }
        [word] => match lookup(word) {
            Some(entry) => {
                print(entry);
                Ok(())
            }
            None => Err(format!("{word} is not in the dictionary")),
        },
        _ => Err("usage: parlo pronounce [word] [--text TEXT]".to_string()),
    }
}

fn print(entry: &parlo_core::pronounce::Entry) {
    println!(
        "{}  {}  {}  {}",
        entry.word, entry.respell, entry.ipa, entry.cmu
    );
}
