use chrono::NaiveDateTime;
use vihr::*;

use clap::{Parser, Subcommand};

use std::path::Path;
use std::{env, fs, io, io::Write};

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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts recording time for a category.
    Start {
        /// The category to record time to.
        #[clap(value_parser)]
        category: String,
        /// The starting point of the recording.
        /// If not specified the current moment will be used.
        #[clap(value_parser, long, short)]
        start_time: Option<NaiveDateTime>,
    },
    /// Stops recording time.
    Stop {
        /// An optional description of the spent time.
        #[clap(value_parser)]
        desc: Option<String>,
        /// The ending point of the recording.
        /// If not specified the current moment will be used.
        #[clap(value_parser, long, short)]
        stop_time: Option<NaiveDateTime>,
    },
    /// Shows if time is currently being recorded.
    Status,
    /// Cancels current time recording.
    Cancel,
    /// Adds spent time to a category.
    Add {
        /// The category to add the spent time to.
        #[clap(value_parser)]
        category: String,
        /// The starting point of the recording.
        #[clap(value_parser)]
        start_time: NaiveDateTime,
        /// The ending point of the recording.
        #[clap(value_parser)]
        stop_time: NaiveDateTime,
        /// An optional description of the spent time.
        #[clap(value_parser)]
        desc: Option<String>,
    },
    /// Removes spent time from a category.
    Remove {
        /// The category from which to remove the spent time.
        #[clap(value_parser)]
        category: String,
        /// The id of the spent time to remove.
        #[clap(value_parser)]
        id: usize,
    },
    /// Prints a summary of time spent.
    Summary {
        /// The time span from which to print the summary.
        #[clap(value_parser)]
        shown_span: Option<ShownTimeSpan>,
        /// The category to print.
        #[clap(value_parser, long, short)]
        category: Option<String>,
    },
    /// Prints a log of spent times.
    Log {
        /// The time span from which to print the log.
        #[clap(value_parser)]
        shown_span: Option<ShownTimeSpan>,
        /// The category to print.
        #[clap(value_parser, long, short)]
        category: Option<String>,
    },
    /// Adds a new category.
    AddCategory {
        /// The category to add.
        #[clap(value_parser)]
        category: String,
    },
    /// Removes a category.
    RemoveCategory {
        /// The category to remove.
        #[clap(value_parser)]
        category: String,
    },
    /// Prints all categories.
    ListCategories,
}

fn handle_commands(book: &mut TimeBook) -> Result<()> {
    let cli = CliArgs::parse();

    match cli.command {
        Commands::Start {
            category,
            start_time,
        } => {
            book.start(category, start_time)?;
        }
        Commands::Stop { desc, stop_time } => {
            book.stop(stop_time, desc)?;
        }
        Commands::Status => {
            let (s, d) = book.status()?;
            println!("Since {}: {}", d, s);
        }
        Commands::Cancel => {
            book.cancel()?;
        }
        Commands::Add {
            category,
            start_time,
            stop_time,
            desc,
        } => {
            book.add_time_usage(&category, start_time, stop_time, desc)?;
        }
        Commands::Remove { category, id } => {
            book.remove_time_usage(&category, id)?;
        }
        Commands::Summary {
            shown_span,
            category,
        } => {
            let shown_span = shown_span.unwrap_or(ShownTimeSpan::All);

            if let Some(c) = category {
                let spent = book.time_spent(&c, shown_span)?;
                println!(
                    "{}: {} h {} min(s)",
                    c,
                    spent.num_hours(),
                    spent.num_minutes() - spent.num_hours() * 60
                );
            } else {
                for cat in book.categories() {
                    let spent = book.time_spent(cat, shown_span)?;
                    println!(
                        "{}: {} h {} min(s)",
                        cat,
                        spent.num_hours(),
                        spent.num_minutes() - spent.num_hours() * 60
                    );
                }
            }
        }
        Commands::Log {
            shown_span,
            category,
        } => {
            println!(
                "{}",
                book.time_usage_log(shown_span.unwrap_or(ShownTimeSpan::All), category)?
            );
        }
        Commands::AddCategory { category } => {
            book.add_category(category)?;
        }
        Commands::RemoveCategory { category } => {
            let stdin = io::stdin();
            let mut stdout = io::stdout();
            let mut confirmation_buff = String::new();

            loop {
                print!("Remove category {} (y/n)? ", category);
                stdout.flush().expect("Failed to flush stdout");
                confirmation_buff.clear();
                stdin
                    .read_line(&mut confirmation_buff)
                    .expect("Failed to read line");
                confirmation_buff = confirmation_buff.to_lowercase().trim().to_string();

                if &confirmation_buff != "y" && &confirmation_buff != "n" {
                    eprintln!("Invalid option.");
                    continue;
                }
                break;
            }

            let confirmed = &confirmation_buff == "y";

            if confirmed {
                book.remove_category(&category)?;
            } else {
                println!("Abort!");
            }
        }
        Commands::ListCategories => {
            for cat in book.categories() {
                println!("{}", cat);
            }
        }
    }

    Ok(())
}
