use std::{fs::File, io::Read};

use anyhow::{Result, ensure};

const MAX_CHUNK_SIZE: u32 = 256 * 1024 * 1024; // 256 MB

#[derive(Debug, Clone)]
struct ChunkHeader {
    chunk_id: [u8; 4],
    chunk_size: u32,
}
impl From<&[u8; 8]> for ChunkHeader {
    fn from(header_bytes: &[u8; 8]) -> Self {
        let chunk_id: [u8; 4] = header_bytes[0..4].try_into().unwrap();
        let chunk_size: u32 = u32::from_le_bytes(header_bytes[4..8].try_into().unwrap());

        Self {
            chunk_id,
            chunk_size,
        }
    }
}

#[derive(Debug)]
struct RiffHeader {
    chunk_header: ChunkHeader,
    format: [u8; 4],
}

#[derive(Debug)]
pub struct FmtChunk {
    pub chunk_header: ChunkHeader,
    pub audio_format: u16,
    pub num_channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
}

#[derive(Debug)]
pub struct UnknownChunk {
    pub chunk_header: ChunkHeader,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct DataChunk {
    pub chunk_header: ChunkHeader,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum Chunk {
    Fmt(FmtChunk),
    Data(DataChunk),
    Unknown(UnknownChunk),
}

#[derive(Debug)]
pub struct WavFile {
    riff_header: RiffHeader,
    chunks: Vec<Chunk>,
}
impl WavFile {
    pub fn from_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let mut riff_header_buf = [0u8; 12];
        reader.read_exact(&mut riff_header_buf)?;

        ensure!(
            &riff_header_buf[0..4] == b"RIFF" && &riff_header_buf[8..12] == b"WAVE",
            "invalid WAV signature"
        );

        let riff_header = RiffHeader {
            chunk_header: ChunkHeader::from(&riff_header_buf[..8].try_into().unwrap()),
            format: riff_header_buf[8..12].try_into().unwrap(),
        };

        // Read chunks from reader
        let mut chunks: Vec<Chunk> = Vec::new();
        let mut header_buf: [u8; 8] = [0; 8];
        loop {
            // Check for eof and break out of loop
            match reader.read_exact(&mut header_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            };

            let chunk_header = ChunkHeader::from(&header_buf);
            let has_padding = chunk_header.chunk_size % 2 != 0;

            // Sanity check chunk size before allocating
            ensure!(chunk_header.chunk_size <= MAX_CHUNK_SIZE, "chunk too large");
            let mut data = vec![0u8; chunk_header.chunk_size as usize];
            reader.read_exact(&mut data)?;

            match &chunk_header.chunk_id {
                b"fmt " => {
                    ensure!(data.len() >= 16, "fmt chunk too small");

                    let audio_format = u16::from_le_bytes(data[0..2].try_into().unwrap());
                    let num_channels = u16::from_le_bytes(data[2..4].try_into().unwrap());
                    let sample_rate = u32::from_le_bytes(data[4..8].try_into().unwrap());
                    let byte_rate = u32::from_le_bytes(data[8..12].try_into().unwrap());
                    let block_align = u16::from_le_bytes(data[12..14].try_into().unwrap());
                    let bits_per_sample = u16::from_le_bytes(data[14..16].try_into().unwrap());

                    chunks.push(Chunk::Fmt(FmtChunk {
                        chunk_header,
                        audio_format,
                        num_channels,
                        sample_rate,
                        byte_rate,
                        block_align,
                        bits_per_sample,
                    }));
                }
                b"data" => chunks.push(Chunk::Data(DataChunk { chunk_header, data })),
                _ => chunks.push(Chunk::Unknown(UnknownChunk { chunk_header, data })),
            }

            if has_padding {
                // Read and discard paddings if it exists
                let mut padding = [0u8; 1];
                reader.read_exact(&mut padding)?;
            }
        }

        // Ensure the existence of fmt chunk
        ensure!(
            chunks.iter().any(|c| matches!(c, Chunk::Fmt(_))),
            "corrupt reader: missing fmt chunk"
        );

        Ok(Self {
            riff_header,
            chunks,
        })
    }
}

// EVERYTHING IS PRIVATE
