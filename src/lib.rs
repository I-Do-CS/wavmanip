pub mod cli;
pub mod config;
pub mod wav;

use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

use anyhow::{Context, Ok, Result, ensure};

use crate::{cli::Cli, config::Config};

const MAX_FILE_SIZE_MB: u64 = 500;
const MAX_FILE_SIZE_BYTES: u64 = MAX_FILE_SIZE_MB * 1_000_000;

pub fn run(cli: Cli) -> Result<()> {
    // Parse CLI
    let config = Config::try_from(cli)?;

    // Validate input path
    ensure!(
        config.input_file.exists(),
        "the specified input file was not found {}",
        config.input_file.display()
    );

    // Validate regular file
    ensure!(
        config.input_file.is_file(),
        "{} is not a regular file",
        config.input_file.display()
    );

    // Check file size
    let size = fs::metadata(&config.input_file)?.len();

    // Blank file skip
    ensure!(size > 0, "the specified input file is blank");

    // Too big
    ensure!(
        size <= MAX_FILE_SIZE_BYTES,
        "the specified input file's size ({}MB) must not exceed {MAX_FILE_SIZE_MB}MB",
        size / 1_000_000
    );

    // Check file extension
    let is_wav = is_file_extension("wav", &config.input_file)
        || is_file_extension("wave", &config.input_file);
    ensure!(is_wav, "the specified input file is not a WAV file");

    // Check the file header and make sure it's a valid wav
    let mut file = File::open(&config.input_file)
        .with_context(|| format!("failed to open {}", config.input_file.display()))?;

    let wavfile = wav::WavFile::from_reader(&mut file);
    println!("{:?}", wavfile);
    // Apply transformations

    // Resolve output path

    // Create parent directories

    // Write temp file

    // Rename temp file to final output

    // Success

    Ok(())
}

/// Case-insensitively checks wether a &Path ends with the specified `extension`
fn is_file_extension(extension: &str, path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}
