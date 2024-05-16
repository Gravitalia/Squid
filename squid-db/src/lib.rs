#![forbid(unsafe_code)]
#![deny(dead_code, unused_imports, unused_mut, missing_docs)]
//! # squid-db
//!
//! internal database used by Squid to store tokenized texts.

#[cfg(feature = "compress")]
mod compress;
mod manager;
mod ttl;

pub use manager::Instance;

use ttl::TTL;
use crate::manager::World;
use squid_error::{Error, ErrorType, IoError};
use std::{
    collections::BTreeMap,
    fs::{create_dir, read_dir, File, OpenOptions},
    io::{self, BufRead, BufReader},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::{mpsc::Sender, RwLock};

const SOURCE_DIRECTORY: &str = "./data/";
const FILE_EXT: &str = "bin";
const MAX_ENTRIES_PER_FILE: usize = 10_000;

/// Attributes required for TTL management.
pub trait Attributes {
    /// Unique identifier for the sentence.
    fn id(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Duration, in seconds, of sentence retention.
    fn ttl(&self) -> Option<u64> {
        None
    }
}

/// [`Builder`] handle database creation.
#[derive(Default)]
pub struct Builder<
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static,
> {
    /// Database name.
    _name: String,
    /// After how many kb the data is written hard to the disk.
    memtable_flush_size_in_kb: usize,
    /// Async MPSC sender.
    sender: Option<Sender<T>>,
    /// Is TTL manager is enabled.
    ttl: bool,
    phantom: PhantomData<T>,
}

impl<T> Builder<T>
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static,
{
    /// Set a name for the database.
    /// Does not affect anything.
    fn _name(mut self, name: String) -> Self {
        self._name = name;
        self
    }

    /// Set the threshold, in kilobytes, for flushing the memory table.
    ///
    /// When set to 0, writing to the memtable is disabled.
    ///
    /// Specifying a non-zero value enables more controlled write management,
    /// reducing the risk of overwriting disk data. However, it comes with
    /// a potential increase in RAM consumption (up to the specified value),
    /// and there's a risk of data loss in the event of a program crash,
    /// especially if crash management hasn't been implemented.
    pub fn memtable_flush_size(mut self, threshold_in_kb: usize) -> Self {
        self.memtable_flush_size_in_kb = threshold_in_kb;
        self
    }

    /// Set [`tokio::sync::mpsc::Sender`] to notify on expired values.
    ///
    /// By providing a sender, you enable the database to communicate expiration
    /// events to other parts of your program or system asynchronously.
    pub fn mpsc_sender(mut self, sender: Sender<T>) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Enables time-to-live (TTL) on entries.
    pub fn with_ttl(mut self) -> Self {
        self.ttl = true;
        self
    }

    /// Build [`squid_db::manager::Instance`].
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
    ///     //... then you can do enything with the instance.
    /// }
    /// ```
    pub async fn build(
        self,
    ) -> Result<Arc<RwLock<manager::Instance<T>>>, Error> {
        let (entires, index, file, mut file_name) = load::<T>()?;

        let file = file.unwrap_or_else(|| {
            file_name = uuid::Uuid::new_v4().to_string();
            let path = PathBuf::from(SOURCE_DIRECTORY)
                .join(format!("{}.{}", file_name, FILE_EXT));

            OpenOptions::new()
                .read(true)
                .append(true)
                .create(true)
                .open(&path)
                .unwrap_or_else(|_| {
                    panic!(
                        "failed to create new file on {}",
                        path.to_string_lossy()
                    )
                })
        });

        let instance = Arc::new(RwLock::new(manager::Instance {
            file,
            file_name,
            index,
            ttl: None,
            entries: entires.0,
            memtable: Vec::new(),
            memtable_flush_size_in_kb: self.memtable_flush_size_in_kb,
            sender: self.sender,
            phantom: PhantomData,
        }));

        if self.ttl {
            let ttl = Arc::new(RwLock::new(TTL::new(Arc::clone(&instance))));

            for entry in &instance.read().await.entries {
                if let Some(expire) = entry.ttl() {
                    let _ = ttl.write().await.add_entry(entry.id(), expire);
                }
            }

            ttl.read().await.init();
        }

        Ok(instance)
    }
}

/// Loads a specific data file rather than the whole set.
#[inline(always)]
fn load_file<T>(mut name: String) -> Result<World<T>, Error>
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static,
{
    if !name.ends_with(FILE_EXT) {
        name = format!("{}.{}", name, FILE_EXT);
    }

    let file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(Path::new(SOURCE_DIRECTORY).join(name))
        .map_err(|error| {
            Error::new(
                ErrorType::Unspecified,
                Some(Box::new(error)),
                Some("while opening file".to_string()),
            )
        })?;

    let reader = BufReader::new(&file);
    let mut world: World<T> = World(Vec::new());

    for line in reader.lines() {
        let line_data: T = bincode::deserialize(
            line.map_err(|error| {
                Error::new(
                    ErrorType::InputOutput(IoError::ReadingError),
                    Some(Box::new(error)),
                    Some("cannot read line before deserialization".to_string()),
                )
            })?
            .as_bytes(),
        )
        .map_err(|error| {
            Error::new(
                ErrorType::InputOutput(IoError::DeserializationError),
                Some(Box::new(error)),
                Some("cannot serialize to read file".to_string()),
            )
        })?;

        world.0.push(line_data);
    }

    Ok(world)
}

/// Reads data from each saved file in the source directory,
/// generates an index, and returns any unfinished files
/// (those with fewer than the specified maximum entries).
#[inline(always)]
fn load<T>(
) -> Result<(World<T>, BTreeMap<String, String>, Option<File>, String), Error>
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static,
{
    let mut world: World<T> = World(Vec::new());
    let mut index: BTreeMap<String, String> = BTreeMap::new();
    let mut uncomplete_file: Option<File> = None;
    let mut file_name = String::default();

    let _ = create_dir(SOURCE_DIRECTORY);

    for entry in read_dir(SOURCE_DIRECTORY)
        .map_err(|error| {
            Error::new(
                ErrorType::InputOutput(IoError::WritingError),
                Some(Box::new(error)),
                Some("cannot read data dir".to_string()),
            )
        })?
        .collect::<Result<Vec<_>, io::Error>>()
        .map_err(|error| {
            Error::new(
                ErrorType::InputOutput(IoError::ReadingError),
                Some(Box::new(error)),
                Some("cannot convert into vector".to_string()),
            )
        })?
    {
        let filename = entry.file_name().into_string().unwrap_or_default();
        let mut data: Vec<T> = load_file(filename.to_string())?.0;

        for line in &data {
            index.insert(line.id(), filename.clone());
        }

        if data.len() < MAX_ENTRIES_PER_FILE {
            uncomplete_file = Some(
                OpenOptions::new()
                    .read(true)
                    .append(true)
                    .open(&Path::new(SOURCE_DIRECTORY).join(filename))
                    .map_err(|error| {
                        Error::new(
                            ErrorType::Unspecified,
                            Some(Box::new(error)),
                            Some("while opening file to load it".to_string()),
                        )
                    })?,
            );
            file_name = entry.file_name().into_string().unwrap_or_default();
        }

        world.0.append(&mut data);
    }

    Ok((world, index, uncomplete_file, file_name))
}
