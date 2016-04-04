//! G.711
//!
//! References
//! - [codeproject](http://www.codeproject.com/Articles/14237/Using-the-G-standard)
//! - [hazelware](http://hazelware.luggle.com/tutorials/mulawcompression.html)
//! - [g711.c](http://www.opensource.apple.com/source/tcl/tcl-20/tcl_ext/snack/snack/generic/g711.c)

use codecs::Codec;
use codecs::Codec::*;
use error::*;
use sample::*;

/// µ-law to A-law conversion look-up table.
///
/// Copied from CCITT G.711 specifications.
#[allow(dead_code)]
const ULAW_TO_ALAW: [u8; 128] = [
  1,    1,    2,    2,    3,    3,    4,    4,
  5,    5,    6,    6,    7,    7,    8,    8,
  9,    10,   11,   12,   13,   14,   15,   16,
  17,   18,   19,   20,   21,   22,   23,   24,
  25,   27,   29,   31,   33,   34,   35,   36,
  37,   38,   39,   40,   41,   42,   43,   44,
  46,   48,   49,   50,   51,   52,   53,   54,
  55,   56,   57,   58,   59,   60,   61,   62,
  64,   65,   66,   67,   68,   69,   70,   71,
  72,   73,   74,   75,   76,   77,   78,   79,
  80,   82,   83,   84,   85,   86,   87,   88,
  89,   90,   91,   92,   93,   94,   95,   96,
  97,   98,   99,   100,  101,  102,  103,  104,
  105,  106,  107,  108,  109,  110,  111,  112,
  113,  114,  115,  116,  117,  118,  119,  120,
  121,  122,  123,  124,  125,  126,  127,  128
];

/// A-law to µ-law conversion look-up table.
///
/// Copied from CCITT G.711 specifications.
#[allow(dead_code)]
const ALAW_TO_ULAW: [u8; 128] = [
  1,    3,    5,    7,    9,    11,   13,   15,
  16,   17,   18,   19,   20,   21,   22,   23,
  24,   25,   26,   27,   28,   29,   30,   31,
  32,   32,   33,   33,   34,   34,   35,   35,
  36,   37,   38,   39,   40,   41,   42,   43,
  44,   45,   46,   47,   48,   48,   49,   49,
  50,   51,   52,   53,   54,   55,   56,   57,
  58,   59,   60,   61,   62,   63,   64,   64,
  65,   66,   67,   68,   69,   70,   71,   72,
  73,   74,   75,   76,   77,   78,   79,   80,
  80,   81,   82,   83,   84,   85,   86,   87,
  88,   89,   90,   91,   92,   93,   94,   95,
  96,   97,   98,   99,   100,  101,  102,  103,
  104,  105,  106,  107,  108,  109,  110,  111,
  112,  113,  114,  115,  116,  117,  118,  119,
  120,  121,  122,  123,  124,  125,  126,  127
];

const ALAW_TO_LINEAR: [i16; 256] = [
   -5504, -5248, -6016, -5760, -4480, -4224, -4992, -4736,
   -7552, -7296, -8064, -7808, -6528, -6272, -7040, -6784,
   -2752, -2624, -3008, -2880, -2240, -2112, -2496, -2368,
   -3776, -3648, -4032, -3904, -3264, -3136, -3520, -3392,
   -22016,-20992,-24064,-23040,-17920,-16896,-19968,-18944,
   -30208,-29184,-32256,-31232,-26112,-25088,-28160,-27136,
   -11008,-10496,-12032,-11520,-8960, -8448, -9984, -9472,
   -15104,-14592,-16128,-15616,-13056,-12544,-14080,-13568,
   -344,  -328,  -376,  -360,  -280,  -264,  -312,  -296,
   -472,  -456,  -504,  -488,  -408,  -392,  -440,  -424,
   -88,   -72,   -120,  -104,  -24,   -8,    -56,   -40,
   -216,  -200,  -248,  -232,  -152,  -136,  -184,  -168,
   -1376, -1312, -1504, -1440, -1120, -1056, -1248, -1184,
   -1888, -1824, -2016, -1952, -1632, -1568, -1760, -1696,
   -688,  -656,  -752,  -720,  -560,  -528,  -624,  -592,
   -944,  -912,  -1008, -976,  -816,  -784,  -880,  -848,
    5504,  5248,  6016,  5760,  4480,  4224,  4992,  4736,
    7552,  7296,  8064,  7808,  6528,  6272,  7040,  6784,
    2752,  2624,  3008,  2880,  2240,  2112,  2496,  2368,
    3776,  3648,  4032,  3904,  3264,  3136,  3520,  3392,
    22016, 20992, 24064, 23040, 17920, 16896, 19968, 18944,
    30208, 29184, 32256, 31232, 26112, 25088, 28160, 27136,
    11008, 10496, 12032, 11520, 8960,  8448,  9984,  9472,
    15104, 14592, 16128, 15616, 13056, 12544, 14080, 13568,
    344,   328,   376,   360,   280,   264,   312,   296,
    472,   456,   504,   488,   408,   392,   440,   424,
    88,    72,   120,   104,    24,     8,    56,    40,
    216,   200,   248,   232,   152,   136,   184,   168,
    1376,  1312,  1504,  1440,  1120,  1056,  1248,  1184,
    1888,  1824,  2016,  1952,  1632,  1568,  1760,  1696,
    688,   656,   752,   720,   560,   528,   624,   592,
    944,   912,  1008,   976,   816,   784,   880,   848
];

const ULAW_TO_LINEAR: [i16; 256] = [
   -32124,-31100,-30076,-29052,-28028,-27004,-25980,-24956,
   -23932,-22908,-21884,-20860,-19836,-18812,-17788,-16764,
   -15996,-15484,-14972,-14460,-13948,-13436,-12924,-12412,
   -11900,-11388,-10876,-10364, -9852, -9340, -8828, -8316,
    -7932, -7676, -7420, -7164, -6908, -6652, -6396, -6140,
    -5884, -5628, -5372, -5116, -4860, -4604, -4348, -4092,
    -3900, -3772, -3644, -3516, -3388, -3260, -3132, -3004,
    -2876, -2748, -2620, -2492, -2364, -2236, -2108, -1980,
    -1884, -1820, -1756, -1692, -1628, -1564, -1500, -1436,
    -1372, -1308, -1244, -1180, -1116, -1052,  -988,  -924,
     -876,  -844,  -812,  -780,  -748,  -716,  -684,  -652,
     -620,  -588,  -556,  -524,  -492,  -460,  -428,  -396,
     -372,  -356,  -340,  -324,  -308,  -292,  -276,  -260,
     -244,  -228,  -212,  -196,  -180,  -164,  -148,  -132,
     -120,  -112,  -104,   -96,   -88,   -80,   -72,   -64,
      -56,   -48,   -40,   -32,   -24,   -16,    -8,    -1,
    32124, 31100, 30076, 29052, 28028, 27004, 25980, 24956,
    23932, 22908, 21884, 20860, 19836, 18812, 17788, 16764,
    15996, 15484, 14972, 14460, 13948, 13436, 12924, 12412,
    11900, 11388, 10876, 10364,  9852,  9340,  8828,  8316,
     7932,  7676,  7420,  7164,  6908,  6652,  6396,  6140,
     5884,  5628,  5372,  5116,  4860,  4604,  4348,  4092,
     3900,  3772,  3644,  3516,  3388,  3260,  3132,  3004,
     2876,  2748,  2620,  2492,  2364,  2236,  2108,  1980,
     1884,  1820,  1756,  1692,  1628,  1564,  1500,  1436,
     1372,  1308,  1244,  1180,  1116,  1052,   988,   924,
      876,   844,   812,   780,   748,   716,   684,   652,
      620,   588,   556,   524,   492,   460,   428,   396,
      372,   356,   340,   324,   308,   292,   276,   260,
      244,   228,   212,   196,   180,   164,   148,   132,
      120,   112,   104,    96,    88,    80,    72,    64,
       56,    48,    40,    32,    24,    16,     8,     0
];

/// Convert an 8-bit A-law value to a 16-bit LPCM sample.
#[inline]
pub fn alaw_to_linear(alaw_value: u8) -> i16 {
  ALAW_TO_LINEAR[(alaw_value) as usize]
}

/// Convert an 8-bit µ-law value to a 16-bit LPCM sample.
#[inline]
pub fn ulaw_to_linear(ulaw_value: u8) -> i16 {
  ULAW_TO_LINEAR[ulaw_value as usize]
}

/// Convert a 16-bit LPCM sample to an 8-bit A-law value.
#[allow(overflowing_literals, unused_comparisons)]
pub fn linear_to_alaw(sample: i16) -> u8 {
  let mut pcm_value = sample;
  let sign          = (pcm_value & 0x8000) >> 8;

  if sign != 0 {
    pcm_value = -pcm_value;
  }

  // Clip at 15-bits
  if pcm_value > 0x7fff {
    pcm_value = 0x7fff;
  }

  let mut exponent: i16 = 7;
  let mut mask          = 0x4000;

  while pcm_value & mask == 0 && exponent > 0 {
    exponent -= 1;
    mask >>= 1;
  }

  let manitssa: i16 =
    if exponent == 0 {
      (pcm_value >> 4) & 0x0f
    }
    else {
      (pcm_value >> (exponent + 3)) & 0x0f
    };

  let alaw_value = sign | exponent << 4 | manitssa;

  (alaw_value ^ 0xd5) as u8
}

/// Convert a 16-bit LPCM sample to an 8-bit µ-law value.
pub fn linear_to_ulaw(sample: i16) -> u8 {
  let mut pcm_value = sample;
  let sign          = (pcm_value >> 8) & 0x80;

  if sign != 0 {
    pcm_value = -pcm_value;
  }

  if pcm_value > 32635 {
    pcm_value = 32635;
  }

  pcm_value += 0x84;

  let mut exponent: i16 = 7;
  let mut mask          = 0x4000;

  while pcm_value & mask == 0 {
    exponent -=  1;
    mask     >>= 1;
  }

  let manitssa: i16 = (pcm_value >> (exponent + 3)) & 0x0f;
  let ulaw_value    = sign | exponent << 4 | manitssa;

  (!ulaw_value) as u8
}

pub fn write(samples: &[Sample], codec: Codec) -> AudioResult<Vec<u8>> {
  let num_bytes = samples.len();
  let mut bytes = vec![0u8; num_bytes];

  match codec {
    G711_ALAW => {
      for (i, sample) in samples.iter().enumerate() {
        bytes[i] = linear_to_alaw(i16::from_sample(*sample));
      }
    },

    G711_ULAW => {
      for (i, sample) in samples.iter().enumerate() {
        bytes[i] = linear_to_ulaw(i16::from_sample(*sample));
      }
    },

    c => {
      return Err(AudioError::Unsupported(
        format!("Unsupported codec {} was passed into the G711 decoder", c)
      ))
    }
  }

  Ok(bytes)
}

pub fn read_sample(bytes: &[u8], codec: Codec) -> AudioResult<Sample> {
  let required_num_bytes = codec.bit_depth() / 8;

  if bytes.len() != required_num_bytes {
    return Err(AudioError::Unsupported(
      "Missing some bytes for sample decode".to_string()))
  }

  let sample =
    match codec {
      G711_ALAW => {
        alaw_to_linear(bytes[0]).to_sample()
      },

      G711_ULAW => {
        ulaw_to_linear(bytes[0]).to_sample()
      },

      c => {
        return Err(AudioError::Unsupported(
          format!("Unsupported codec {} was passed into the G711 decoder", c)
        ))
      }
    };

  Ok(sample)
}

#[cfg(test)]
mod coding {
  mod encode {
    use ::buffer::*;
    use ::codecs::Codec::*;
    use ::codecs::g711;
    use ::codecs::g711::*;

    #[test]
    fn unsupported_codec() {
      let audio  = AudioBuffer::from_samples(44100, 1, vec![0f32; 4]);
      let codecs =
        vec![
          LPCM_U8,
          LPCM_I8,
          LPCM_I16_LE,
          LPCM_I16_BE,
          LPCM_I24_LE,
          LPCM_I24_BE,
          LPCM_I32_LE,
          LPCM_I32_BE,
          LPCM_F32_LE,
          LPCM_F32_BE,
          LPCM_F64_LE,
          LPCM_F64_BE
        ];

      for unsupported_codec in codecs.iter() {
        assert!(g711::write(&audio.samples, *unsupported_codec).is_err());
      }
    }


    #[test]
    fn i16_to_alaw() {
      assert_eq!(213, linear_to_alaw(0));
      assert_eq!(213, linear_to_alaw(1));
      assert_eq!(213, linear_to_alaw(2));
      assert_eq!(85,  linear_to_alaw(-3));
      assert_eq!(85,  linear_to_alaw(-4));
    }

    #[test]
    fn i16_to_ulaw() {
      assert_eq!(0xff, linear_to_ulaw(0));
      assert_eq!(0x7f, linear_to_ulaw(-1));
    }
  }

  mod decode {
    use ::codecs::Codec::*;
    use ::codecs::g711;
    use ::codecs::g711::*;

    #[test]
    fn unsupported_codec() {
      let bytes  = vec![0u8; 4];
      let codecs =
        vec![
          LPCM_U8,
          LPCM_I8,
          LPCM_I16_LE,
          LPCM_I16_BE,
          LPCM_I24_LE,
          LPCM_I24_BE,
          LPCM_I32_LE,
          LPCM_I32_BE,
          LPCM_F32_LE,
          LPCM_F32_BE,
          LPCM_F64_LE,
          LPCM_F64_BE
        ];

      for unsupported_codec in codecs.iter() {
        assert!(g711::read_sample(&bytes, *unsupported_codec).is_err());
      }
    }

    #[test]
    fn alaw_to_i16() {
      assert_eq!( 8, alaw_to_linear(213));
      assert_eq!(-8, alaw_to_linear(85));
    }

    #[test]
    fn ulaw_to_i16() {
      assert_eq!( 0, ulaw_to_linear(0xff));
      assert_eq!(-1, ulaw_to_linear(0x7f));
    }
  }
}
