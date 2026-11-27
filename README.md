# WavManip

A Rust command-line tool that reads a RIFF/WAVE file, parses its chunks and writes it back out. It handles the binary layout directly, from little-endian headers to the padding after odd-length chunks.

## Try it

With a stable Rust toolchain that supports the 2024 edition, run from the repository root:

```bash
cargo run --locked -- input.wav roundtrip.wav
cargo test --locked
```

Use a separate output path for the rewritten file.

## Read the code

- [wav.rs](src/wav.rs) contains the format model. `WavFile::from_reader` and `WavFile::to_writer` work with Rust’s `Read` and `Write` traits. The model keeps unknown chunk payloads so it can write them back out.
- [lib.rs](src/lib.rs) validates the input file and connects the parser and writer.
- [cli.rs](src/cli.rs) and [config.rs](src/config.rs) handle arguments and output naming.
