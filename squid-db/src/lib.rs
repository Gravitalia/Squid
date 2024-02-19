//! # squid-db
//!
//! internal database used by Squid to store tokenized texts.

#![forbid(unsafe_code)]
#![deny(dead_code, unused_imports, unused_mut, missing_docs)]

/// Compresses bytes to reduce size.
#[cfg(feature = "compress")]
mod compress;

use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt,
    fs::{write, OpenOptions},
    io::Read,
};

const SOURCE_FILE: &str = "./data.bin";

/// Database errors.
#[derive(Debug)]
pub enum DbError {
    /// An error with absolutely no details.
    Unspecified,
    /// The compression failed.
    #[cfg(feature = "compress")]
    FailedCompression,
    /// The deserialization failed.
    FailedDeserialization,
    /// The serialization failed.
    FailedSerialization,
    /// Error while reading data.
    FailedReading,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DbError::Unspecified => write!(f, "Unknown error"),
            #[cfg(feature = "compress")]
            DbError::FailedCompression => write!(f, "An error occurred during compression"),
            DbError::FailedDeserialization => write!(f, "An error occurred during deserialization"),
            DbError::FailedSerialization => write!(f, "An error occurred during serialization, check the serde implementation"),
            DbError::FailedReading => write!(f, "The data was not read correctly"),
        }
    }
}

impl Error for DbError {}

/// Structure representing the database world.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct World<T>(Vec<T>)
where
    T: serde::Serialize;

/// Add a new entry to the database.
///
/// # Examples
/// ```rust
/// use serde::{Deserialize, Serialize};
/// use squid_db::set;
///
/// #[derive(Serialize, Deserialize, PartialEq, Debug)]
/// struct Entity {
///     data: String,
/// }
///
/// let mut new_data = Entity { data: "I really like my classmate, Julien".to_string() };
/// set(new_data);
///
/// new_data = Entity { data: "But I do not speak to Julien".to_string() };
/// set(new_data);
/// ```
pub fn set<T>(data: T) -> Result<(), DbError>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    let mut world: World<T> = load()?;
    world.0.insert(0, data);

    let encoded: Vec<u8> =
        bincode::serialize(&world).map_err(|_| DbError::FailedSerialization)?;

    #[cfg(feature = "compress")]
    compress::compress(&encoded).map_err(|_| DbError::FailedCompression)?;
    #[cfg(not(feature = "compress"))]
    save(&encoded)?;

    Ok(())
}

/// Loads data from the file.
fn load<T>() -> Result<World<T>, DbError>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(SOURCE_FILE)
        .map_err(|_| DbError::Unspecified)?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|_| DbError::FailedReading)?;

    if buffer.is_empty() {
        Ok(World(vec![]))
    } else {
        #[cfg(feature = "compress")]
        let result = bincode::deserialize(
            &compress::decompress(None)
                .map_err(|_| DbError::FailedCompression)?,
        )
        .map_err(|_| DbError::FailedDeserialization)?;

        #[cfg(not(feature = "compress"))]
        let result = bincode::deserialize(&buffer)
            .map_err(|_| DbError::FailedDeserialization)?;

        Ok(result)
    }
}

/// Save data in a file.
#[allow(dead_code)]
fn save(source: &[u8]) -> Result<(), DbError> {
    write(SOURCE_FILE, source).map_err(|_| DbError::Unspecified)?;
    Ok(())
}
