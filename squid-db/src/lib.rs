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
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
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
pub struct World<T>(pub Vec<T>)
where
    T: serde::Serialize;

/// Structure representing one instance of the database.
#[derive(Debug)]
pub struct Instance<T: serde::Serialize> {
    /// File.
    file: File,
    /// The data in the file represented in the memory.
    pub data: World<T>,
}

impl<T> Instance<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    /// Create a new database instance.
    ///
    /// # Examples
    /// ```rust
    /// use serde::{Deserialize, Serialize};
    /// use squid_db::Instance;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Entity {
    ///     data: String,
    /// }
    ///
    /// let instance: Instance<Entity> = Instance::new().unwrap();
    /// //... then you can do enything with the instance.
    /// ```
    pub fn new() -> Result<Self, DbError> {
        let loaded_file = load()?;

        Ok(Self {
            file: loaded_file.0,
            data: loaded_file.1,
        })
    }

    /// Add a new entry to the database.
    ///
    /// # Examples
    /// ```rust
    /// use serde::{Deserialize, Serialize};
    /// use squid_db::Instance;
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Entity {
    ///     data: String,
    ///     love_him: bool,
    /// }
    ///
    /// let mut instance: Instance<Entity> = Instance::new().unwrap();
    ///
    /// instance.set(Entity {
    ///     data: "I really like my classmate, Julien".to_string(),
    ///     love_him: false,
    /// });
    ///
    /// instance.set(Entity {
    ///     data: "But I do not speak to Julien".to_string(),
    ///     love_him: true,
    /// });
    /// ```
    pub fn set(&mut self, data: T) -> Result<(), DbError> {
        #[cfg(feature = "compress")]
        let encoded = compress::compress(
            &bincode::serialize(&self.data.0)
                .map_err(|_| DbError::FailedSerialization)?,
        )
        .map_err(|_| DbError::FailedCompression)?;

        #[cfg(not(feature = "compress"))]
        /*let encoded = bincode::serialize(&self.data.0)
        .map_err(|_| DbError::FailedSerialization)?;*/
        let encoded = bincode::serialize(&data)
            .map_err(|_| DbError::FailedSerialization)?;

        // Add new data on cache.
        self.data.0.insert(0, data);

        //self.save_all(&encoded)?;
        self.save(&encoded)?;

        Ok(())
    }

    /// Save all data in a file.
    /// If any data is omitted during registration, it is deleted.
    #[inline(always)]
    #[allow(unused)]
    fn save_all(&mut self, buf: &[u8]) -> Result<(), DbError> {
        std::fs::write(SOURCE_FILE, buf).map_err(|_| DbError::Unspecified)?;
        Ok(())
    }

    /// Append one data to the file.
    #[inline(always)]
    #[allow(unused)]
    fn save(&mut self, buf: &[u8]) -> Result<(), DbError> {
        let mut buffer: Vec<u8> = vec![];

        buffer.extend_from_slice(buf);
        buffer.extend_from_slice(b"\n");

        self.file
            .write_all(&buffer)
            .map_err(|_| DbError::Unspecified)?;

        Ok(())
    }
}

/// Loads data from the file.
#[inline(always)]
fn load<T>() -> Result<(File, World<T>), DbError>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(SOURCE_FILE)
        .map_err(|_| DbError::Unspecified)?;

    let reader = BufReader::new(&file);
    let mut world: World<T> = World(Vec::new());

    for line in reader.lines() {
        world.0.push(
            bincode::deserialize(
                line.map_err(|_| DbError::FailedReading)?.as_bytes(),
            )
            .map_err(|_| DbError::FailedDeserialization)?,
        );
    }

    /*
        #[cfg(feature = "compress")]
        let result = bincode::deserialize(
            &compress::decompress(&buffer[..])
                .map_err(|_| DbError::FailedCompression)?,
        )
        .map_err(|_| DbError::FailedDeserialization)?;

        #[cfg(not(feature = "compress"))]
        let result = bincode::deserialize(&buffer[..])
            .map_err(|_| DbError::FailedDeserialization)?;

        Ok((file, result))
    */

    Ok((file, world))
}
