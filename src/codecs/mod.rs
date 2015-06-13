use buffer::*;
use error::{AudioResult, AudioError};

pub mod lpcm;
pub use codecs::lpcm::LPCM as LPCM;

/// An enumeration of all supported audio codecs
pub enum Codec {
  LPCM
}

/// A trait for all functions required by all codecs
pub trait AudioCodec {
  fn read(bytes: &mut Vec<u8>, bit_rate: &u32, channels: &u32) -> AudioResult<Vec<Sample>>;
}

/*
/// Decodes bytes with the provided codec
pub fn read_codec(codec: Codec, bytes: Vec<u8>) -> AudioResult<Vec<Sample>> {
  match codec {
    Codec::LPCM => LPCM::read(bytes)
  }
}
*/