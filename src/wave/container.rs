use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use buffer::*;
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt, WriteBytesExt};
use codecs::Codec;
use codecs::Codec::*;
use error::*;
use sample::*;
use traits::{Chunk, Container};
use wave::{RIFF, WAVE, FMT, FACT, DATA};
use wave::chunks::*;
use wave::chunks::WaveChunk::*;

/// Struct containing all necessary information for encoding and decoding
/// bytes to an `AudioBuffer`.
pub struct WaveContainer {
  codec:            Codec,
  pub bit_depth:    u32,
  pub sample_rate:  u32,
  pub channels:     u32,
  pub block_size:   u32,
  pub order:        SampleOrder,
  pub samples:      Vec<Sample>
}

impl Container for WaveContainer {
  fn open<R: Read + Seek>(reader: &mut R) -> AudioResult<WaveContainer> {
    // Read and validate riff header
    let mut riff_header: [u8; 12] = [0u8; 12];
    try!(reader.read(&mut riff_header));
    if &riff_header[0..4]  != RIFF
    || &riff_header[8..12] != WAVE {
      return Err(AudioError::Format(
        "Not valid WAVE".to_string()
      ));
    }
    let file_size : u32 = LittleEndian::read_u32(&riff_header[4..8]) - 4;
    let mut buffer: Cursor<Vec<u8>> = Cursor::new(vec![0u8; file_size as usize]);
    try!(reader.read(buffer.get_mut()));

    // Read all supported chunks
    let mut container =
      WaveContainer {
        codec:          Codec::LPCM_I16_LE,
        bit_depth:      0u32,
        sample_rate:    0u32,
        channels:       1u32,
        block_size:     0u32,
        order:          SampleOrder::Interleaved,
        samples:        Vec::with_capacity(1024)
      };
    let mut chunk_header      : [u8; 8] = [0u8; 8];
    let mut read_fmt_chunk    : bool    = false;
    let mut read_fact_chunk   : bool    = false;
    let mut read_data_chunk   : bool    = false;
    while buffer.position() < file_size as u64 {
      try!(buffer.read(&mut chunk_header));
      let chunk_size: usize =
        LittleEndian::read_u32(&chunk_header[4..8]) as usize;
      let pos: usize = buffer.position() as usize;
      match identify(&chunk_header[0..4]).ok() {
        Some(Format) => {
          let chunk_bytes = &(buffer.get_ref()[pos .. pos + chunk_size]);
          let fmt_chunk = try!(FormatChunk::read(&chunk_bytes));
          container.bit_depth       = fmt_chunk.bit_depth    as u32;
          container.sample_rate     = fmt_chunk.sample_rate;
          container.channels        = fmt_chunk.num_channels as u32;
          container.block_size      = fmt_chunk.block_size   as u32;
          container.order           =
            if container.channels == 1 {
              SampleOrder::Mono
            } else {
              SampleOrder::Interleaved
            };
          container.codec           =
            try!(determine_codec(fmt_chunk.format_tag,
                                 fmt_chunk.bit_depth));
          read_fmt_chunk            = true;
          if fmt_chunk.format_tag == FormatTag::Pcm {
            // Don't need to check for fact chunk if PCM
            read_fact_chunk = true;
          }
        },
        Some(Fact) => {
          // Don't actually use it, but we do need to check if it exists.
          // let chunk_bytes   = &(buffer.get_ref()[pos .. pos + chunk_size]);
          // let num_samples_per_channel = LittleEndian::read_u32(&chunk_bytes[0..4]);
          read_fact_chunk   = true;
        }
        Some(Data) => {
          if !read_fmt_chunk {
            return Err(AudioError::Format(
              "File is not valid WAVE \
              (Format chunk does not occur before Data chunk)".to_string()
            ))
          }
          let chunk_bytes   = &(buffer.get_ref()[pos .. pos + chunk_size]);
          container.samples = try!(read_codec(chunk_bytes, container.codec));
          read_data_chunk   = true;
        },
        None => {}
      }
      try!(buffer.seek(SeekFrom::Current(chunk_size as i64)));
    }

    // Check if required chunks were read
    if !read_fmt_chunk {
      return Err(AudioError::Format(
        "File is not valid WAVE (Missing required Format chunk)".to_string()
      ))
    }
    if !read_fact_chunk {
      return Err(AudioError::Format(
        "File is not valid WAVE \
        (Missing Fact chunk for non-PCM data)".to_string()
      ))
    }
    if !read_data_chunk {
      return Err(AudioError::Format(
        "File is not valid WAVE (Missing required Data chunk)".to_string()
      ))
    }
    Ok(container)
  }
  fn create<W: Write>(writer: &mut W, audio: &AudioBuffer, codec: Codec) -> AudioResult<()> {
    // Determine if codec is supported by container and if data is non-PCM.
    let data_non_pcm: bool = try!(is_supported(codec));
    // Encode audio samples using codec.
    let data: Vec<u8> = try!(write_codec(audio, codec));
    let fmt_chunk_size = FormatChunk::calculate_size(audio, codec);
    let mut total_bytes: u32 = 12 + (8 + fmt_chunk_size)
                                  + (8 + data.len() as u32);
    // Files encoded with non-PCM data must include a fact chunk.
    if data_non_pcm {
      total_bytes += 12;
    }

    // Write the riff header to the writer.
    try!(writer.write(RIFF));
    try!(writer.write_u32::<LittleEndian>(total_bytes - 8));
    try!(writer.write(WAVE));
    // Write fmt chunk to the writer.
    try!(FormatChunk::write(writer, audio, codec));
    // Write fact chunk to writer if data is non-PCM
    if data_non_pcm {
      try!(FactChunk::write(writer, audio));
    }
    // Write data chunk to the writer.
    try!(DataChunk::write(writer, &data));
    Ok(())
  }
}

// Private functions

/// This function reads the four byte identifier for each WAVE chunk.
#[inline]
fn identify(bytes: &[u8]) -> AudioResult<WaveChunk> {
  match &[bytes[0], bytes[1], bytes[2], bytes[3]] {
    FMT  => Ok(Format),
    FACT => Ok(Fact),
    DATA => Ok(Data),
    err @ _ =>
      Err(AudioError::Format(
        format!("Do not recognize WAVE chunk with identifier {:?}", err)
      ))
  }
}

/// Determines if codec is supported by container. Since WAVE encoding also
/// depends on whether the codec is integer-based PCM, the return value
/// represeents if the codec as such.
fn is_supported(codec: Codec) -> AudioResult<bool> {
  match codec {
    LPCM_U8      |
    LPCM_I16_LE  |
    LPCM_I24_LE  |
    LPCM_I32_LE  => Ok(false),
    LPCM_F32_LE  |
    LPCM_F64_LE  |
    G711_ALAW    |
    G711_ULAW    => Ok(true),
    c @ _ =>
      return Err(AudioError::Unsupported(
        format!("Wave does not support the {:?} codec", c)
      ))
  }
}

/// Returns the `Codec` used by the read audio attributes.
fn determine_codec(format_tag: FormatTag, bit_depth: u16) -> AudioResult<Codec> {
  match (format_tag, bit_depth) {
    (FormatTag::Pcm,    8) => Ok(LPCM_U8),
    (FormatTag::Pcm,   16) => Ok(LPCM_I16_LE),
    (FormatTag::Pcm,   24) => Ok(LPCM_I24_LE),
    (FormatTag::Pcm,   32) => Ok(LPCM_I32_LE),
    (FormatTag::ALaw,   8) => Ok(G711_ALAW),
    (FormatTag::MuLaw,  8) => Ok(G711_ULAW),
    (FormatTag::Float, 32) => Ok(LPCM_F32_LE),
    (FormatTag::Float, 64) => Ok(LPCM_F64_LE),
    (_, _) =>
      return Err(AudioError::Unsupported(
        "Audio encoded with unsupported codec".to_string()
      ))
  }
}

/// Returns samples read using the given codec. If the container does not
/// support a codec, an error is returned.
#[inline]
fn read_codec(bytes: &[u8], codec: Codec) -> AudioResult<Vec<Sample>> {
  match is_supported(codec) {
    Ok(_)  => ::codecs::decode(bytes, codec),
    Err(e) => Err(e)
  }
}

/// Returns samples as bytes created using the given codec. If the container
/// does not support a codec, an error is returned.
#[inline]
fn write_codec(audio: &AudioBuffer, codec: Codec) -> AudioResult<Vec<u8>> {
  match is_supported(codec) {
    Ok(_)  => ::codecs::encode(audio, codec),
    Err(e) => Err(e)
  }
}
