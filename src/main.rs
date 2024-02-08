use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[cfg(debug_assertions)]
use log::LevelFilter::{Debug};
#[cfg(not(debug_assertions))]
use log::LevelFilter::{Debug, Warn};
use api_release::data::FileData;

use api_release::fs::generate_file_data_from_path;
use api_release::node::diff::FileDiff;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long)]
    debug: Option<bool>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    // /// does testing things
    // Test {
    //     /// lists test values
    //     #[arg(short, long)]
    //     list: bool,
    // },
    Scan {
        /// Optional path to operate on
        path: PathBuf,

        /// Sets a custom config file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },
    Diff {
        /// path to operate on
        path: PathBuf,

        /// source file
        source: PathBuf,
    },
    Patch {
        /// path to operate on
        path: PathBuf,

        /// source file
        source: PathBuf,

        /// Sets a custom config file
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
    }
}

fn main() {
    let cli = Cli::parse();

    #[cfg(debug_assertions)]
    simple_logger::SimpleLogger::new().with_level(Debug).init().unwrap();

    #[cfg(not(debug_assertions))]
    if cli.debug.unwrap_or(false) {
        simple_logger::SimpleLogger::new().with_level(Debug).init().unwrap();
    } else {
        simple_logger::SimpleLogger::new().with_level(Warn).init().unwrap();
    }

    log::info!("Starting up v{}", env!("CARGO_PKG_VERSION"));

    match &cli.command {
        // Some(Commands::Test { list }) => {
        //     if *list {
        //         log::info!("Listing test values");
        //     } else {
        //         log::info!("Running test");
        //     }
        // },
        Some(Commands::Scan { path , output}) => {
            let path = path.to_owned();
            if !path.exists() || path.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }
            let output = output.to_owned().unwrap_or(PathBuf::from("out.bin.gz"));
            log::info!("Scanning {}", path.display());
            let file_data = generate_file_data_from_path(&path).unwrap();
            log::info!("Saving output to {}", output.display());
            file_data.save(output).unwrap();
        },
        Some(Commands::Diff { path, source }) => {
            if !path.exists() || path.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }
            log::info!("Compare {} with {}", path.display(), source.display());

            let source_filedata = FileData::load(source);
            let target_filedata = generate_file_data_from_path(&path).unwrap();
            let diffs = source_filedata.diff(&target_filedata);
            for diff in diffs {
                log::info!("{}", diff);
            }
        },
        Some(Commands::Patch { path, source, output }) => {
            if !path.exists() || path.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }
            let output = output.to_owned().unwrap_or(PathBuf::from(".").join("patch"));

            if output.exists() {
                log::warn!("Patch folder already exist: {}", output.display());
            } else {
                fs::create_dir_all(&output).unwrap();
            }

            log::info!("Generate patch {} with {} to {}", path.display(), source.display(), output.display());

            let source_filedata = FileData::load(source);
            let target_filedata = generate_file_data_from_path(&path).unwrap();
            let diffs = source_filedata.diff(&target_filedata);

            for diff in diffs {
                match diff {
                    FileDiff::Change(detail) | FileDiff::Add(detail)  => {
                        log::info!("Adding file: {}", &detail);
                        log::debug!("Copying from {} to {}", detail.get_path(&path).display(), detail.get_path(&output).display());
                        if detail.is_file {
                            let target_path = detail.get_path(&output);
                            if let Some(parent) = target_path.parent() {
                                fs::create_dir_all(parent).unwrap();
                            }
                            fs::copy(detail.get_path(&path), target_path).unwrap();
                        } else {
                            fs::create_dir_all(detail.get_path(&output)).unwrap();
                        }
                    },
                    _ => {},
                }
            }
        },
        None => {},
    }
}
