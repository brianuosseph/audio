//! The Audio Interchange File Format
//!
//! AIFF files use the Interchange File Format (IFF) is a generic
//! file container format that uses chunks to store data.
//! Traditionally all bytes are stored in big-endian format,
//! but some files are 

//! References
//! - [McGill University](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/AIFF/AIFF.html)
//! - [AIFF Spec](http://www-mmsp.ece.mcgill.ca/Documents/AudioFormats/AIFF/Docs/AIFF-1.3.pdf)

mod chunks;
pub mod decoder;
pub mod encoder;

pub use aiff::decoder::Decoder as Decoder;
pub use aiff::encoder::Encoder as Encoder;

/*
#[cfg(test)]
mod tests {
	#[test]
	fn test_read_write_eq() {
		use super::*;
		
		let folder: String = String::from_str("tests/aiff/");
		let files = vec![
			"i16-pcm-mono.aiff",
			"i16-pcm-stereo.aiff",
			"Warrior Concerto - no meta.aiff"
		];

		for file in files.iter() {
			let mut path: String = folder.clone();
			path.push_str(*file);

			let audio = decoder::read_file(&Path::new(path.as_slice())).unwrap();
			let total_samples = audio.samples.len();
			let channels = audio.channels;
			let bit_rate = audio.bit_rate;
			let sample_rate = audio.sample_rate;
			let sample_order = audio.order;

			let written = encoder::write_file(&audio, &Path::new("tmp.aiff")).unwrap();
			assert!(written);

			let verify = decoder::read_file(&Path::new("tmp.aiff")).unwrap();

			// Assert written file is same length as read file!
			assert_eq!(total_samples, verify.samples.len());
			assert_eq!(channels, verify.channels);
			assert_eq!(bit_rate, verify.bit_rate);
			assert_eq!(sample_rate, verify.sample_rate);
			assert_eq!(sample_order, verify.order);
		}
	}
}
*/