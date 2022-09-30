use vihr::*;

use std::{env, fs};
use std::path::Path;

fn main() {
    let filename = env::var("VIHR_SAVE_FILE");

    if filename.is_err() {
        eprintln!("Environment variable 'VIHR_SAVE_FILE' is not defined.");
        std::process::exit(1);
    }

    let filename = filename.unwrap();
    let book;

    // Serde needs a string reference. It needs to live as long as `book`.
    let json_str;

    if Path::new(&filename).exists() {
        let json = fs::read_to_string(&filename);

        if let Err(e) = json {
            eprintln!("Could not read save file '{}'.", filename);
            eprintln!("{}", e);
            std::process::exit(1);
        }

        json_str = json.unwrap();

        book = serde_json::from_str(&json_str);
    } else {
        println!("Save file doesn't exist. It will be created.");
        book = Ok(TimeBook::default());
    }

    if let Err(e) = book {
        eprintln!("Could not parse json from file '{}'.", filename);
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let mut book: TimeBook = book.unwrap();

    if let Err(e) = handle_commands(&mut book) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let json = serde_json::to_string(&book);

    if let Err(e) = json {
        eprintln!("Could not serialize the TimeBook to json.");
        eprintln!("{}", e);
        std::process::exit(1);
    }

    let json = json.unwrap();
    
    if let Err(e) = fs::write(&filename, json) {
        eprintln!("Could not write save file '{}'.", filename);
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn handle_commands(_book: &mut TimeBook) -> Result<()> {
    Ok(())
}

