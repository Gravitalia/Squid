use lz4::{Decoder, EncoderBuilder};
use std::io::{self, Result};

/// Compresses data using `lz4`.
pub fn compress(mut source: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = EncoderBuilder::new()
        .level(3)
        .favor_dec_speed(true)
        .build(Vec::new())?;
    io::copy(&mut source, &mut encoder)?;

    let (buffer, _) = encoder.finish();

    Ok(buffer)
}

/// Decompress data using `lz4`.
pub fn decompress(buf: &[u8]) -> Result<Vec<u8>> {
    //let input_file: File = File::open(SOURCE_FILE)?;
    let mut decoder = Decoder::new(buf)?;
    let mut output = Vec::new();
    io::copy(&mut decoder, &mut output)?;

    Ok(output)
}
