use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::cli::Cli;

#[derive(Debug)]
pub struct Config {
    pub input_file: PathBuf,
    pub output_file: PathBuf,
    pub volume_multiplier: f32,
    pub reverse: bool,
}
impl Config {
    fn derive_output_path(input: &Path, reverse: bool) -> PathBuf {
        let mut name = input
            .file_stem()
            .map(|s| s.to_os_string())
            .unwrap_or_else(|| "output".into());

        if reverse {
            name.push("_reversed.wav");
        } else {
            name.push("_modified.wav");
        }

        input.with_file_name(name)
    }
}
impl TryFrom<Cli> for Config {
    type Error = anyhow::Error;

    fn try_from(value: Cli) -> Result<Self> {
        let reverse = value.reverse;
        let volume_multiplier = value.volume_multiplier.unwrap_or(1.0);
        let input_file = value.input_file;
        let output_file = value
            .output_file
            .unwrap_or(Self::derive_output_path(&input_file, reverse));

        Ok(Self {
            input_file,
            output_file,
            volume_multiplier,
            reverse,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Cli;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn derives_modified_output_path_in_same_directory() {
        let input = PathBuf::from("music/song.wav");

        let output = Config::derive_output_path(&input, false);

        assert_eq!(output, PathBuf::from("music/song_modified.wav"));
    }

    #[test]
    fn derives_reversed_output_path_in_same_directory() {
        let input = PathBuf::from("music/song.wav");

        let output = Config::derive_output_path(&input, true);

        assert_eq!(output, PathBuf::from("music/song_reversed.wav"));
    }

    #[test]
    fn defaults_volume_multiplier_to_one() {
        let cli = Cli {
            input_file: PathBuf::from("song.wav"),
            output_file: None,
            volume_multiplier: None,
            reverse: false,
        };

        let config = Config::try_from(cli).unwrap();

        assert_eq!(config.volume_multiplier, 1.0);
    }

    #[test]
    fn preserves_provided_volume_multiplier() {
        let cli = Cli {
            input_file: PathBuf::from("song.wav"),
            output_file: None,
            volume_multiplier: Some(2.5),
            reverse: false,
        };

        let config = Config::try_from(cli).unwrap();

        assert_eq!(config.volume_multiplier, 2.5);
    }

    #[test]
    fn generates_output_file_when_not_provided() {
        let cli = Cli {
            input_file: PathBuf::from("song.wav"),
            output_file: None,
            volume_multiplier: None,
            reverse: false,
        };

        let config = Config::try_from(cli).unwrap();

        assert_eq!(config.output_file, PathBuf::from("song_modified.wav"));
    }

    #[test]
    fn respects_explicit_output_file() {
        let cli = Cli {
            input_file: PathBuf::from("song.wav"),
            output_file: Some(PathBuf::from("custom.wav")),
            volume_multiplier: None,
            reverse: false,
        };

        let config = Config::try_from(cli).unwrap();

        assert_eq!(config.output_file, PathBuf::from("custom.wav"));
    }

    #[test]
    fn preserves_reverse_flag() {
        let cli = Cli {
            input_file: PathBuf::from("song.wav"),
            output_file: None,
            volume_multiplier: None,
            reverse: true,
        };

        let config = Config::try_from(cli).unwrap();

        assert!(config.reverse);
    }
}
