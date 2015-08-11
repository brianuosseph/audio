# audio [![Build Status](https://travis-ci.org/brianuosseph/audio.svg?branch=master)](https://travis-ci.org/brianuosseph/audio)
A Rust audio coding library.

## TODO
- Better support for alternative WAVE sample formats
  - When exactly should the encoder write using `WAVE_FORMAT_EXTENSIBLE`? Should the user be able to specify this using a separate codec (like in Audacity)?
- Research conversion of Sample to `f32` (Precision issues come up with converstion to `i32`)
- Better support for RIFF and IFF metadata tags
- Integrate `crate rust-id3` for handling ID3 metadata
- Better integration tests
- Refactor unit tests in `aiff` and `wave` modules
- Write examples
- Look into using `Container::open` and `Container::create` as part of the public API
- Possibly add a "from_buffer" function
- Possibly add `open_as` and `load_as` for reading data as a different audio format
- Come up with a name!
- Explore other audio formats

## Decoding

| Audio Format | Codec | Data formats |
| ------ | ----- | --------- |
| WAVE | PCM | u8, alaw, ulaw, i16, i24, i32, f32, f64 |
| AIFF | PCM | u8, i8, alaw, ulaw, i16, i24, i32, f32, f64 |

## Encoding

| Audio Format | Codec | Bit Rates |
| ------ | ----- | --------- |
| WAVE | PCM | u8, alaw, ulaw, i16, i24, i32 |
| AIFF | PCM | u8, i8, alaw, ulaw, i16, i24, i32, f32, f64 |
