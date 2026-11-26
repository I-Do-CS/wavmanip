# WavManip

**A Rust command-line project for working with RIFF/WAV audio files.**

`Rust 2024` · `Binary I/O` · `Clap` · `anyhow`

WavManip is an independent systems-programming project that models the WAV binary format directly, including RIFF headers, chunk headers, format metadata and audio data.

## Highlights

- RIFF/WAVE signature validation and little-endian encoding and decoding.
- Typed format, data and unknown chunks with word-alignment padding.
- Generic Rust Read/Write interfaces, file-size limits and contextual errors.
- Command-line argument validation, output-path derivation and unit tests.

## Build and run

Install a stable Rust toolchain supporting edition 2024.

```bash
git clone https://github.com/I-Do-CS/wavmanip.git
cd wavmanip
cargo build --release
cargo test
cargo run -- --help
cargo run -- input.wav copied.wav
```

Use a distinct output path. The current development version focuses on parsing and serialization; volume and reversal options are reserved for future sample processing.

## Source layout

| File | Responsibility |
| --- | --- |
| [cli.rs](src/cli.rs) | Command-line interface |
| [config.rs](src/config.rs) | Configuration and output naming |
| [wav.rs](src/wav.rs) | RIFF/WAV model, reader and writer |
| [lib.rs](src/lib.rs) | File validation and orchestration |
| [main.rs](src/main.rs) | Entry point |

Independent project by [Ali Ghelichkhani](https://github.com/I-Do-CS).
