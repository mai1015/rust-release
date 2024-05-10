use std::{env, fs};
use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[cfg(debug_assertions)]
use log::LevelFilter::{Debug};
#[cfg(not(debug_assertions))]
use log::LevelFilter::{Info, Warn};
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
    /// Scans a path and generates a file data
    Scan {
        /// Optional path to operate on
        path: PathBuf,

        /// Sets a custom config file
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        #[arg(short, long, value_name = "LIST", value_delimiter = ',')]
        ignore: Option<Vec<String>>,
    },
    /// Compares a path with a source file
    Diff {
        /// path to operate on
        path: PathBuf,

        /// source file
        source: PathBuf,

        #[arg(short, long, value_name = "LIST", value_delimiter = ',')]
        ignore: Option<Vec<String>>,
    },
    /// Generates a patch from a path and a source file
    Patch {
        /// path to operate on
        path: PathBuf,

        /// source file
        source: PathBuf,

        /// Sets a custom config file
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,

        #[arg(short, long, value_name = "LIST", value_delimiter = ',')]
        ignore: Option<Vec<String>>,
    },
    /// Generates a release from a path and a source file
    Release {
        /// path to operate on
        path: PathBuf,

        /// source file
        source: PathBuf,

        /// Sets a custom config file
        #[arg(short='o', long, value_name = "PATH")]
        output_path: Option<PathBuf>,

        /// Sets a custom config file
        #[arg(short='f', long, value_name = "PATH")]
        output_file: Option<PathBuf>,

        #[arg(short, long, value_name = "LIST", value_delimiter = ',')]
        ignore: Option<Vec<String>>,
    },
}


#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    #[cfg(debug_assertions)]
    simple_logger::SimpleLogger::new().with_level(Debug).init().unwrap();

    #[cfg(not(debug_assertions))]
    if cli.debug.unwrap_or(false) {
        simple_logger::SimpleLogger::new().with_level(Info).init().unwrap();
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
        Some(Commands::Scan { path , output, ignore}) => {
            let path = path.to_owned();
            if !path.exists() || path.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }
            let output = output.to_owned().unwrap_or(PathBuf::from("out.bin.gz"));
            log::info!("Scanning {}", path.display());
            let ignores = ignore.to_owned().unwrap_or_default();
            let file_data = generate_file_data_from_path(&path, &ignores).await.unwrap();
            log::info!("Saving output to {}", output.display());
            file_data.save(output).unwrap();
        },
        Some(Commands::Diff { path, source, ignore }) => {
            log::info!("Compare {} with {} at {}", path.display(), source.display(), env::current_dir().unwrap().display());
            if !path.exists() || path.is_file() || !source.exists() || !source.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }

            let source_filedata = FileData::load(source);
            let ignores = ignore.to_owned().unwrap_or_default();
            let target_filedata = generate_file_data_from_path(&path, &ignores).await.unwrap();
            let diffs = source_filedata.diff(&target_filedata);
            for diff in diffs {
                log::info!("{}", diff);
            }
        },
        Some(Commands::Patch { path, source, output, ignore }) => {
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
            let ignores = ignore.to_owned().unwrap_or_default();
            let target_filedata = generate_file_data_from_path(&path, &ignores).await.unwrap();
            let diffs = source_filedata.diff(&target_filedata);

            generate_patch(path, &output, diffs);
        },
        Some(Commands::Release { path, source, output_path, output_file, ignore }) => {
            if !path.exists() || path.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }
            let output_path = output_path.to_owned().unwrap_or(PathBuf::from(".").join("release"));
            let output_file = output_file.to_owned().unwrap_or(PathBuf::from("out.bin.gz"));

            if output_path.exists() {
                log::warn!("Release folder already exist: {}", output_path.display());
            } else {
                fs::create_dir_all(&output_path).unwrap();
            }

            log::info!("Generate release {} with {} to {}", path.display(), source.display(), output_path.display());
            let source_filedata = FileData::load(source);
            let ignores = ignore.to_owned().unwrap_or_default();
            let target_filedata = generate_file_data_from_path(&path, &ignores).await.unwrap();
            let diffs = source_filedata.diff(&target_filedata);

            log::info!("Copying files...");
            generate_patch(path, &output_path, diffs);

            log::info!("Saving output to {}", output_file.display());
            target_filedata.save(output_file).unwrap();
        },
        _ => {},
    }

}

#[cfg(not(feature = "async"))]
fn main() {
    let cli = Cli::parse();

    #[cfg(debug_assertions)]
    simple_logger::SimpleLogger::new().with_level(Debug).init().unwrap();

    #[cfg(not(debug_assertions))]
    if cli.debug.unwrap_or(false) {
        simple_logger::SimpleLogger::new().with_level(Info).init().unwrap();
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
        Some(Commands::Scan { path , output, ignore}) => {
            let path = path.to_owned();
            if !path.exists() || path.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }
            let output = output.to_owned().unwrap_or(PathBuf::from("out.bin.gz"));
            log::info!("Scanning {}", path.display());
            let ignores = ignore.to_owned().unwrap_or_default();
            let file_data = generate_file_data_from_path(&path, &ignores).unwrap();
            log::info!("Saving output to {}", output.display());
            file_data.save(output).unwrap();
        },
        Some(Commands::Diff { path, source, ignore }) => {
            log::info!("Compare {} with {} at {}", path.display(), source.display(), env::current_dir().unwrap().display());
            if !path.exists() || path.is_file() || !source.exists() || source.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }
            log::info!("Compare {} with {}", path.display(), source.display());

            let source_filedata = FileData::load(source);
            let ignores = ignore.to_owned().unwrap_or_default();
            let target_filedata = generate_file_data_from_path(&path, &ignores).unwrap();
            let diffs = source_filedata.diff(&target_filedata);
            for diff in diffs {
                log::info!("{}", diff);
            }
        },
        Some(Commands::Patch { path, source, output, ignore }) => {
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
            let ignores = ignore.to_owned().unwrap_or_default();
            let target_filedata = generate_file_data_from_path(&path, &ignores).unwrap();
            let diffs = source_filedata.diff(&target_filedata);

            generate_patch(path, &output, diffs);
        },
        Some(Commands::Release { path, source, output_path, output_file, ignore }) => {
            if !path.exists() || path.is_file() {
                log::error!("Path does not exist or it is a file");
                return;
            }
            let output_path = output_path.to_owned().unwrap_or(PathBuf::from(".").join("release"));
            let output_file = output_file.to_owned().unwrap_or(PathBuf::from("out.bin.gz"));

            if output_path.exists() {
                log::warn!("Release folder already exist: {}", output_path.display());
            } else {
                fs::create_dir_all(&output_path).unwrap();
            }

            log::info!("Generate release {} with {} to {}", path.display(), source.display(), output_path.display());
            let source_filedata = FileData::load(source);
            let ignores = ignore.to_owned().unwrap_or_default();
            let target_filedata = generate_file_data_from_path(&path, &ignores).unwrap();
            let diffs = source_filedata.diff(&target_filedata);

            log::info!("Copying files...");
            generate_patch(path, &output_path, diffs);

            log::info!("Saving output to {}", output_file.display());
            target_filedata.save(output_file).unwrap();
        },
        _ => {},
    }
}

fn generate_patch(path: &PathBuf, output: &PathBuf, diffs: Vec<FileDiff>) {
    for diff in diffs {
        match diff {
            FileDiff::Change(detail) | FileDiff::Add(detail) => {
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
}