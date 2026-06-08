use std::{fmt::Debug, path::PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "wavmanip",
    author = "Ali Ghelichkhani <idocs4aliving@gmail.com>",
    version,
    about = "A CLI tool for changing volume and reversing .wav/.wave files.",
    after_help = "\
Examples:
    wavmanip(.exe) -v 2 infile.wav outfile.wav 
    wavmanip(.exe) --volume=1.3 rec23.wave reversed.wav --reverse
    wavmanip(.exe) --reverse inputs/infile.wav
"
)]
pub struct Cli {
    /// Input WAV file
    pub input_file: PathBuf,

    /// Output WAV file
    pub output_file: Option<PathBuf>,

    /// Volume multiplier
    #[arg(short, long = "volume", value_parser = parse_volume)]
    pub volume_multiplier: Option<f32>,

    /// Reverse audio samples
    #[arg(short, long)]
    pub reverse: bool,
}

/// Private function for parsing the volume argument
fn parse_volume(s: &str) -> Result<f32, String> {
    let value: f32 = s.parse().map_err(|_| "must be a number")?;

    if !value.is_finite() {
        return Err("must be a finite number".into());
    }
    if !(0.0..=100.0).contains(&value) {
        return Err("must be between 0.0 and 100.0".into());
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_volume() {
        assert!(parse_volume("1.1").is_ok());
    }

    #[test]
    fn rejects_invalid_volume() {
        assert!(parse_volume("Skibidi Toilet").is_err());
    }

    #[test]
    fn rejects_nan_volume() {
        assert!(parse_volume("nan").is_err());
        assert!(parse_volume("NaN").is_err());
    }

    #[test]
    fn rejects_inf_volume() {
        assert!(parse_volume("inf").is_err());
        assert!(parse_volume("-inf").is_err());
        assert!(parse_volume("+inf").is_err());
    }
}
