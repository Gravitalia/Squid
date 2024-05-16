//! database manager.
//! supports read, write, memtable.

use crate::{
    ttl::TTL, Attributes, FILE_EXT, MAX_ENTRIES_PER_FILE, SOURCE_DIRECTORY,
};
use serde::Serialize;
use squid_error::{Error, ErrorType, IoError};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    marker::PhantomData,
    path::PathBuf,
    sync::Arc,
};
use tokio::sync::{mpsc::Sender, RwLock};
#[cfg(feature = "logging")]
use tracing::trace;

/// Structure representing the database world.
#[derive(Serialize, PartialEq, Debug)]
pub struct World<T>(pub Vec<T>)
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static;

/// Structure representing one instance of the database.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Instance<
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static,
> {
    /// File writing new entries.
    /// There is no need to re-open the file each time.
    pub(super) file: File,
    /// Opened file UUID.
    pub(super) file_name: String,
    /// Index to link an ID to a file.
    /// This allows the file to be targeted for modification or deletion.
    pub(super) index: BTreeMap<String, String>,
    /// TTL manager.
    pub(super) ttl: Option<Arc<RwLock<TTL<T>>>>,
    /// Data saved on disk.
    pub entries: Vec<T>,
    /// Caching of data to be written to avoid overload and bottlenecks.
    pub(super) memtable: Vec<T>,
    /// After how many kb the data is written hard to the disk.
    /// Set to 0 to deactivate the memory table.
    pub(super) memtable_flush_size_in_kb: usize,
    /// MPSC consumer used to know expired sentences.
    /// Created by yourself using [`tokio::sync::mpsc`].
    pub(crate) sender: Option<Sender<T>>,
    pub(super) phantom: PhantomData<T>,
}

impl<T> Instance<T>
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static,
{
    /// Get entry from its unique identifier.
    pub fn get(&self, id: String) -> Result<Option<T>, Error> {
        if let Some(file_name) = self.index.get(&id) {
            let data = crate::load_file::<T>(file_name.to_string())?.0;

            Ok(data.into_iter().find(|entry| entry.id() == id))
        } else {
            Ok(None)
        }
    }

    /// Add a new entry to the database.
    ///
    /// # Examples
    /// ```rust
    /// use serde::{Deserialize, Serialize};
    /// use squid_db::{Builder, Instance, Attributes};
    /// use std::sync::Arc;
    /// use tokio::sync::RwLock;
    ///
    /// #[derive(Serialize, Deserialize, Default)]
    /// struct Entity {
    ///     data: String,
    ///     love_him: bool,
    /// }
    ///
    /// impl Attributes for Entity {}
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let instance: Arc<tokio::sync::RwLock<Instance<Entity>>> =
    ///         Builder::default().build().await.unwrap();
    ///
    ///     instance.write().await.set(Entity {
    ///         data: "I really like my classmate, Julien".to_string(),
    ///         love_him: false,
    ///     });
    /// 
    ///     instance.write().await.set(Entity {
    ///         data: "But I do not speak to Julien".to_string(),
    ///         love_him: true,
    ///     });
    /// }
    /// ```
    pub async fn set(&mut self, data: T) -> Result<(), Error> {
        if let Some(timestamp) = data.ttl() {
            if let Some(ttl) = &self.ttl {
                ttl.write().await.add_entry(data.id(), timestamp)?;
            }
        }

        #[cfg(feature = "logging")]
        trace!(id = data.id(), "Added new entry.");

        match self.memtable_flush_size_in_kb {
            0 => {
                #[cfg(not(feature = "compress"))]
                let encoded = bincode::serialize(&data).map_err(|error| {
                    Error::new(
                        ErrorType::InputOutput(IoError::DeserializationError),
                        Some(error),
                        Some(
                            "during `bincode` serialization to set new entry"
                                .to_string(),
                        ),
                    )
                })?;

                self.index.insert(data.id(), self.file_name.clone());
                self.save(&encoded)?
            },
            max_kb_size => {
                self.memtable.push(data);

                if max_kb_size
                    < (self.memtable.len() * std::mem::size_of::<T>()) / 1000
                {
                    self.flush().map_err(|error| {
                        Error::new(
                            ErrorType::Unspecified,
                            Some(Box::new(error)),
                            Some("while flushing database".to_string()),
                        )
                    })?
                }
            },
        }

        Ok(())
    }

    /// Deletes a record from the data based on its unique identifier.
    pub fn delete(&mut self, id: &str) -> Result<(), Error> {
        if let Some(file_name) = self.index.get(id) {
            let file =
                File::open(PathBuf::from(SOURCE_DIRECTORY).join(file_name))
                    .map_err(|error| {
                        Error::new(
                            ErrorType::InputOutput(IoError::ReadingError),
                            Some(Box::new(error)),
                            Some(
                                "cannot open file to delete entry".to_string(),
                            ),
                        )
                    })?;
            let reader = BufReader::new(file);

            let lines: Vec<Vec<u8>> = reader
                .lines()
                .map_while(Result::ok)
                .map(|entry| entry.as_bytes().to_vec())
                .collect();

            let index_to_delete = lines.iter().position(|line| {
                if let Ok(data) = bincode::deserialize::<T>(line) {
                    return data.id() == id;
                }
                false
            });

            if let Some(index) = index_to_delete {
                let mut file = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(PathBuf::from(SOURCE_DIRECTORY).join(file_name))
                    .map_err(|error| {
                        Error::new(
                            ErrorType::Unspecified,
                            Some(Box::new(error)),
                            Some(
                                "during file opening to delete row".to_string(),
                            ),
                        )
                    })?;

                lines.iter().enumerate().for_each(|(i, line)| {
                    if i != index {
                        writeln!(file, "{}", String::from_utf8_lossy(line))
                            .unwrap_or_default();
                    }
                });

                #[cfg(feature = "logging")]
                trace!(id = id, file = file_name, "Entry deleted.",);
            }
        } else {
            self.memtable.retain(|entry| entry.id() != id);
        }

        Ok(())
    }

    /// Append one data to the file.
    #[inline(always)]
    #[allow(unused)]
    fn save(&mut self, buf: &[u8]) -> Result<(), Error> {
        let mut line_count = io::BufReader::new(&self.file).lines().count();
        let mut buffer: Vec<u8> = vec![];

        buffer.extend_from_slice(buf);
        buffer.extend_from_slice(b"\n");

        self.file.write_all(&buffer).map_err(|error| {
            Error::new(
                ErrorType::Unspecified,
                Some(Box::new(error)),
                Some("saving context".to_string()),
            )
        })?;

        if line_count + 1 >= MAX_ENTRIES_PER_FILE {
            self.file_name = uuid::Uuid::new_v4().to_string();
            let path = PathBuf::from(SOURCE_DIRECTORY)
                .join(format!("{}.{}", self.file_name, FILE_EXT));

            self.file = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&path)
                .unwrap_or_else(|_| {
                    panic!(
                        "failed to create new file on {}",
                        path.to_string_lossy()
                    )
                });
        }

        Ok(())
    }

    /// Saves the data contained in the buffer to the hard disk.
    pub fn flush(&mut self) -> Result<(), Error> {
        let line_count = io::BufReader::new(&self.file).lines().count();

        if line_count + self.memtable.len() > MAX_ENTRIES_PER_FILE {
            // If we just write all, number of lines will exceed maximum allowed.
            // So, we will split into two different files.
            let mut buffer: Vec<u8> = Vec::with_capacity(self.memtable.len());

            let mut file_limit = MAX_ENTRIES_PER_FILE - line_count;
            for n in 0..file_limit {
                let data = &self.memtable[n];

                buffer.extend_from_slice(&bincode::serialize(&data).map_err(
                    |error| {
                        Error::new(
                            ErrorType::InputOutput(IoError::SerializationError),
                            Some(Box::new(error)),
                            Some(
                                "cannot serialize to flush database"
                                    .to_string(),
                            ),
                        )
                    },
                )?);
                buffer.extend_from_slice(b"\n");

                // Insert new hard entry into index.
                self.index.insert(data.id(), self.file_name.clone());
            }

            self.file.write_all(&buffer).map_err(|error| {
                Error::new(
                    ErrorType::Unspecified,
                    Some(Box::new(error)),
                    Some("flush writing".to_string()),
                )
            })?;
            self.file.flush().map_err(|error| {
                Error::new(
                    ErrorType::Unspecified,
                    Some(Box::new(error)),
                    Some("re-flush on flush over flush".to_string()),
                )
            })?;

            self.file_name = uuid::Uuid::new_v4().to_string();
            let path = PathBuf::from(SOURCE_DIRECTORY)
                .join(format!("{}.{}", self.file_name, FILE_EXT));

            self.file = OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&path)
                .unwrap_or_else(|_| {
                    panic!(
                        "failed to create new file on {}",
                        path.to_string_lossy()
                    )
                });

            for _ in
                1..(line_count + self.memtable.len() - MAX_ENTRIES_PER_FILE)
            {
                file_limit += 1;
                let data = &self.memtable[file_limit];

                buffer.extend_from_slice(&bincode::serialize(&data).map_err(
                    |error| {
                        Error::new(
                            ErrorType::InputOutput(IoError::SerializationError),
                            Some(Box::new(error)),
                            Some(
                                "cannot serialize to flush database"
                                    .to_string(),
                            ),
                        )
                    },
                )?);
                buffer.extend_from_slice(b"\n");

                // Insert new hard entry into index.
                self.index.insert(data.id(), self.file_name.clone());
            }

            self.file.write_all(&buffer).map_err(|error| {
                Error::new(
                    ErrorType::Unspecified,
                    Some(Box::new(error)),
                    Some("flush writing".to_string()),
                )
            })?;
        } else {
            let mut buffer: Vec<u8> = Vec::with_capacity(self.memtable.len());

            for data in &self.memtable {
                buffer.extend_from_slice(&bincode::serialize(&data).map_err(
                    |error| {
                        Error::new(
                            ErrorType::InputOutput(IoError::SerializationError),
                            Some(Box::new(error)),
                            Some(
                                "cannot serialize to flush database"
                                    .to_string(),
                            ),
                        )
                    },
                )?);
                buffer.extend_from_slice(b"\n");

                // Insert new hard entry into index.
                self.index.insert(data.id(), self.file_name.clone());
            }

            self.file.write_all(&buffer).map_err(|error| {
                Error::new(
                    ErrorType::Unspecified,
                    Some(Box::new(error)),
                    Some("again flush writing".to_string()),
                )
            })?;

            self.memtable.clear();
        }

        Ok(())
    }
}
