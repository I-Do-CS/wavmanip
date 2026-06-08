pub mod cli;
pub mod config;

use anyhow::{Ok, Result};

use crate::{cli::Cli, config::Config};

pub fn run(cli: Cli) -> Result<()> {
    // Parse CLI
    let config = Config::try_from(cli)?;
    println!("{:#?}", config);

    // Validate input path

    // Check file size

    // Open input file

    // Read and validate WAV header

    // Load WAV into WaveFile struct

    // Apply transformations

    // Resolve output path

    // Create parent directories

    // Write temp file

    // Rename temp file to final output

    // Success

    Ok(())
}
