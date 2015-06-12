const RIFF: u32 = 0x52494646;
const WAVE: u32 = 0x57415645;
const FMT:  u32 = 0x666D7420;
const DATA: u32 = 0x64617461;

use std::fmt;
use std::io::{Read, Seek};
use buffer::{SampleOrder};
use containers::{Container, Chunk};
use error::*;

/// Enumeration of supported RIFF chunks
enum ChunkType {
  RiffHeader,
  Format,
  Data
}

/// The Resource Interchange File Format (RIFF) is a generic
/// file container format that uses chunks to store data.
/// All bytes are stored in little-endian format.
pub struct RiffContainer<'r, R: 'r> where R: Read + Seek {
  reader: &'r mut R,
  pub bit_rate: u32,
  pub sample_rate: u32,
  pub channels: u32,
  pub order: SampleOrder,
  pub bytes: Vec<u8>
}

impl<'r, R> Container<'r, R> for RiffContainer<'r, R> where R: Read + Seek {
  fn open(r: &'r mut R) -> AudioResult<RiffContainer<'r, R>> {
    let header_chunk_type = try!(identify(r));
    let header            = try!(RiffHeader::read(r));
    let fmt_chunk_type    = try!(identify(r));
    let fmt_chunk         = try!(FormatChunk::read(r));
    let data_chunk_type   = try!(identify(r));
    let data_chunk        = try!(DataChunk::read(r));
    let sample_order
      = if (fmt_chunk.num_of_channels == 1u16) {
        SampleOrder::MONO
      } else {
        SampleOrder::INTERLEAVED
      };
    Ok(RiffContainer {
      reader:       r,
      bit_rate:     fmt_chunk.bit_rate as u32,
      sample_rate:  fmt_chunk.sample_rate,
      channels:     fmt_chunk.num_of_channels as u32,
      order:        sample_order,
      bytes:        data_chunk.bytes
    })
  }
}

/// This function reads the four byte identifier for each RIFF chunk
///
/// If an unsupported chunk is found instead, the identifier bytes are lost
/// and makes reading the remainder of the file for chunks impossible without
/// skipping the length of the chunk indicated by the next four bytes available
/// in the reader.
fn identify<R>(r: &mut R) -> AudioResult<ChunkType> where R: Read + Seek {
  let buffer: &mut[u8] = &mut[0u8; 4];
  try!(r.read(buffer));
  let identifier  :u32
    = (buffer[0] as u32) << 24
    | (buffer[1] as u32) << 16
    | (buffer[2] as u32) << 8
    |  buffer[3] as u32;
  match identifier {
    RIFF => Ok(ChunkType::RiffHeader),
    FMT  => Ok(ChunkType::Format),
    DATA => Ok(ChunkType::Data),
    _ => Err(AudioError::FormatError("Do not recognize RIFF chunk".to_string()))
  }
}

/// All RIFF containers start with a RIFF chunk, which contains
/// subchunks. The file format and size are specified here.
#[derive(Debug, Clone, Copy)]
pub struct RiffHeader {
  pub size: u32,
  pub format: u32,
}

impl Chunk for RiffHeader {
  fn read<R>(r: &mut R) -> AudioResult<RiffHeader> where R: Read + Seek {
    let buffer: &mut[u8] = &mut[0u8; 8];
    try!(r.read(buffer));
    // Converting to little endian
    let file_size       : u32
      = (buffer[3] as u32) << 24
      | (buffer[2] as u32) << 16
      | (buffer[1] as u32) << 8
      |  buffer[0] as u32;
    let form_type: u32
      = (buffer[7] as u32) << 24
      | (buffer[6] as u32) << 16
      | (buffer[5] as u32) << 8
      |  buffer[4] as u32;
    if form_type != WAVE {
      return Err(AudioError::FormatError("This is not a valid WAV file".to_string()))
    }
    Ok(
      RiffHeader {
        size: file_size,
        format: form_type,
      }
    )
  }
}

/// Enumeration of supported compression codes in the RIFF format chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
  Unknown = 0,
  PCM     = 1
}

impl fmt::Display for CompressionType {
  fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
    write!(fmt, "{}", self)
  }
}

/// The format chunck contains most of the audio realted meta data in `WAV` files
#[derive(Debug, Clone, Copy)]
pub struct FormatChunk {
  pub size: u32,
  pub compression_code: CompressionType,
  pub num_of_channels: u16,
  pub sample_rate: u32,
  pub data_rate: u32,
  pub block_size: u16,
  pub bit_rate: u16,
}

impl Chunk for FormatChunk {
  fn read<R>(r: &mut R) -> AudioResult<FormatChunk> where R: Read + Seek {
    let size_buffer: &mut[u8] = &mut[0u8; 4];
    try!(r.read(size_buffer));
    let size              :u32
      = (size_buffer[3] as u32) << 24
      | (size_buffer[2] as u32) << 16
      | (size_buffer[1] as u32) << 8
      |  size_buffer[0] as u32;
    let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
    try!(r.read(&mut buffer));
    let compression_code : u16
      = (buffer[1] as u16) << 8
      |  buffer[0] as u16;
    let compression_type: CompressionType
      = match compression_code {
        1 => CompressionType::PCM,
        _ => CompressionType::Unknown,  // Not supporting any other type than PCM
      };
    let num_of_channels : u16
      = (buffer[3] as u16) << 8
      |  buffer[2] as u16;
    let sample_rate     : u32
      = (buffer[7] as u32) << 24
      | (buffer[6] as u32) << 16
      | (buffer[5] as u32) << 8
      |  buffer[4] as u32;
    let data_rate       : u32
      = (buffer[11] as u32) << 24
      | (buffer[10] as u32) << 16
      | (buffer[9] as u32)  << 8
      |  buffer[8] as u32;
    let block_size      : u16
      = (buffer[13] as u16) << 8
      |  buffer[12] as u16;
    let bit_rate        : u16
      = (buffer[15] as u16) << 8
      |  buffer[14] as u16;

    // Don't care for other bytes if PCM

    Ok(
      FormatChunk {
        size: size,
        compression_code: compression_type,
        num_of_channels: num_of_channels,
        sample_rate: sample_rate,
        data_rate: data_rate,
        block_size: block_size,
        bit_rate: bit_rate,
      }
    )
  }
}

/// The data chunk contains the coded audio data. Multi-channel data are
/// always interleaved in `WAV` files.
pub struct DataChunk {
  pub size: u32,
  pub bytes: Vec<u8>,
}

impl Chunk for DataChunk {
  fn read<R>(r: &mut R) -> AudioResult<DataChunk> where R: Read + Seek {
    let size_buffer: &mut[u8] = &mut[0u8; 4];
    try!(r.read(size_buffer));
    let size              :u32
      = (size_buffer[3] as u32) << 24
      | (size_buffer[2] as u32) << 16
      | (size_buffer[1] as u32) << 8
      |  size_buffer[0] as u32;
    let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
    try!(r.read(&mut buffer));
    Ok(
      DataChunk {
        size: size,
        bytes: buffer,
      }
    )
  }
}