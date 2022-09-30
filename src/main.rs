use vihr::TimeBook;

use std::{env, fs};

fn main() {
    let filename = env::var("VIHR_SAVE_FILE");

    if filename.is_err() {
        eprintln!("Environment variable 'VIHR_SAVE_FILE' is not defined.");
        std::process::exit(1);
    }

    let filename = filename.unwrap();

    let json = fs::read_to_string(&filename);

    if let Err(e) = json {
        eprintln!("Could not read save file '{}'.", filename);
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let json = json.unwrap();

    let book = serde_json::from_str(&json);

    if let Err(e) = book {
        eprintln!("Could parse json from file '{}'.", filename);
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let book: TimeBook = book.unwrap();
}

