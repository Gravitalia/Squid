//! Compression manager.

use flate2::{
    write::{ZlibDecoder, ZlibEncoder},
    Compression,
};
use std::io::{Error, Write};

enum Algorithm {
    Zlib,
}

pub(crate) fn compress(buffer: &[u8]) -> Result<Vec<u8>, Error> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());

    encoder.write_all(buffer)?;

    let result = encoder.finish()?;

    Ok(result)
}

pub(crate) fn decompress(buffer: &[u8]) -> Result<Vec<u8>, Error> {
    let mut writer = Vec::new();
    let mut decoder = ZlibDecoder::new(writer);

    decoder.write_all(&buffer)?;

    writer = decoder.finish()?;

    Ok(writer)
}
