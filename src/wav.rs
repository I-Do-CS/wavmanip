use std::io::{Read, Write};

use anyhow::{Result, ensure};

const MAX_CHUNK_SIZE: u32 = 256 * 1024 * 1024; // 256 MB

#[derive(Debug, Clone)]
struct ChunkHeader {
    chunk_id: [u8; 4],
    chunk_size: u32,
}
impl ChunkHeader {
    fn to_bytes(&self) -> [u8; 8] {
        let mut bytes = [0u8; 8];

        bytes[0..4].copy_from_slice(&self.chunk_id);
        bytes[4..8].copy_from_slice(&self.chunk_size.to_le_bytes());

        bytes
    }
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
impl RiffHeader {
    fn to_bytes(&self) -> [u8; 12] {
        let mut bytes: [u8; 12] = [0; 12];
        bytes[0..8].copy_from_slice(&self.chunk_header.to_bytes());
        bytes[8..12].copy_from_slice(&self.format);

        bytes
    }
}

#[derive(Debug)]
struct FmtChunk {
    chunk_header: ChunkHeader,
    data: Vec<u8>,
}
impl FmtChunk {
    fn audio_format(&self) -> u16 {
        u16::from_le_bytes(self.data[0..2].try_into().unwrap())
    }
    fn num_channels(&self) -> u16 {
        u16::from_le_bytes(self.data[2..4].try_into().unwrap())
    }
    fn sample_rate(&self) -> u32 {
        u32::from_le_bytes(self.data[4..8].try_into().unwrap())
    }
    fn byte_rate(&self) -> u32 {
        u32::from_le_bytes(self.data[8..12].try_into().unwrap())
    }
    fn block_align(&self) -> u16 {
        u16::from_le_bytes(self.data[12..14].try_into().unwrap())
    }
    fn bits_per_sample(&self) -> u16 {
        u16::from_le_bytes(self.data[14..16].try_into().unwrap())
    }
}

#[derive(Debug)]
struct UnknownChunk {
    chunk_header: ChunkHeader,
    data: Vec<u8>,
}

#[derive(Debug)]
struct DataChunk {
    chunk_header: ChunkHeader,
    data: Vec<u8>,
}

#[derive(Debug)]
enum Chunk {
    Fmt(FmtChunk),
    Data(DataChunk),
    Unknown(UnknownChunk),
}
impl Chunk {
    fn chunk_size(&self) -> u32 {
        match self {
            Chunk::Fmt(fmt) => fmt.data.len() as u32,
            Chunk::Data(data) => data.data.len() as u32,
            Chunk::Unknown(chunk) => chunk.data.len() as u32,
        }
    }

    fn total_size(&self) -> u32 {
        let payload = self.chunk_size();

        8 + payload + (payload % 2)
    }
}

#[derive(Debug)]
pub struct WavFile {
    riff_header: RiffHeader,
    chunks: Vec<Chunk>,
    volume_multiplier: f32,
    reverse: bool,
}
impl WavFile {
    pub fn reverse_samples(&mut self) {
        self.reverse = true;
    }

    pub fn undo_reverse_samples(&mut self) {
        self.reverse = false;
    }

    pub fn is_reversed(&self) -> bool {
        self.reverse
    }

    pub fn volume_multiplier(&self) -> f32 {
        self.volume_multiplier
    }

    pub fn adjust_volume(&mut self, multiplier: f32) {
        self.volume_multiplier = multiplier;
    }

    pub fn to_writer<W: Write>(&self, writer: &mut W) -> Result<()> {
        let riff_header = RiffHeader {
            chunk_header: ChunkHeader {
                chunk_id: self.riff_header.chunk_header.chunk_id,
                chunk_size: self.riff_size(),
            },
            format: self.riff_header.format,
        };

        // Write the Riff header
        writer.write_all(&riff_header.to_bytes())?;

        // Write the rest chunk by chunk
        for chunk in &self.chunks {
            match chunk {
                Chunk::Fmt(fmt) => {
                    writer.write_all(&fmt.chunk_header.to_bytes())?;
                    writer.write_all(&fmt.data)?;

                    if fmt.chunk_header.chunk_size % 2 != 0 {
                        writer.write_all(&[0])?;
                    }
                }

                Chunk::Data(data) => {
                    writer.write_all(&data.chunk_header.to_bytes())?;
                    writer.write_all(&data.data)?;

                    if data.chunk_header.chunk_size % 2 != 0 {
                        writer.write_all(&[0])?;
                    }
                }

                Chunk::Unknown(chunk) => {
                    writer.write_all(&chunk.chunk_header.to_bytes())?;
                    writer.write_all(&chunk.data)?;

                    if chunk.chunk_header.chunk_size % 2 != 0 {
                        writer.write_all(&[0])?;
                    }
                }
            }
        }

        Ok(())
    }

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
                    chunks.push(Chunk::Fmt(FmtChunk { chunk_header, data }));
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
            volume_multiplier: 1.0,
            reverse: false,
        })
    }

    fn riff_size(&self) -> u32 {
        4 + self.chunks.iter().map(Chunk::total_size).sum::<u32>()
    }
}
