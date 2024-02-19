use crate::SOURCE_FILE;
use lz4::{Decoder, EncoderBuilder};
use std::{
    fs::File,
    io::{self, Result},
};

/// Compresses data using `lz4`.
pub fn compress(mut source: &[u8]) -> Result<()> {
    let output_file = File::create(SOURCE_FILE)?;

    let mut encoder = EncoderBuilder::new()
        .level(3)
        .favor_dec_speed(true)
        .build(output_file)?;
    io::copy(&mut source, &mut encoder)?;

    let (_, result) = encoder.finish();

    result
}

/// Decompress data using `lz4`.
pub fn decompress(file: Option<&'static str>) -> Result<Vec<u8>> {
    let input_file: File = File::open(file.unwrap_or(SOURCE_FILE))?;
    let mut decoder = Decoder::new(input_file)?;
    let mut output = Vec::new();
    io::copy(&mut decoder, &mut output)?;

    Ok(output)
}
