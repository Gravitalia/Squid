#![forbid(unsafe_code)]
#![deny(dead_code, unused_imports, unused_mut, missing_docs)]
//! # squid-db
//!
//! internal database used by Squid to store tokenized texts.

/// Compresses bytes to reduce size.
#[cfg(feature = "compress")]
mod compress;
mod ttl;

use serde::Serialize;
use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    fs::{create_dir, read_dir, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    marker::PhantomData,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use tokio::sync::{mpsc::Sender, RwLock as AsyncRwLock};
#[cfg(feature = "logging")]
use tracing::trace;
use ttl::TTL;

const SOURCE_DIRECTORY: &str = "./data/";
const FILE_EXT: &str = "bin";
const MAX_ENTRIES_PER_FILE: usize = 10_000;

/// Database errors.
#[derive(Debug)]
pub enum DbError {
    /// Main directory haven't been created.
    DirCreationFailed,
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
    /// Failed unwrap Rwlock or Mutex for writing.
    FailedWriting,
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DbError::DirCreationFailed => write!(f, "The directory could not be created."),
            DbError::Unspecified => write!(f, "Unknown error"),
            #[cfg(feature = "compress")]
            DbError::FailedCompression => write!(f, "An error occurred during compression"),
            DbError::FailedDeserialization => write!(f, "An error occurred during deserialization"),
            DbError::FailedSerialization => write!(f, "An error occurred during serialization, check the serde implementation"),
            DbError::FailedReading => write!(f, "The data was not read correctly"),
            DbError::FailedWriting => write!(f, "Cannot get Rwlock write"),
        }
    }
}

impl Error for DbError {}

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

/// Structure representing the database world.
#[derive(Serialize, PartialEq, Debug)]
pub struct World<T>(pub Vec<T>)
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + std::marker::Send
        + std::marker::Sync
        + 'static;

/// Structure representing one instance of the database.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Instance<
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + std::marker::Send
        + std::marker::Sync
        + 'static
        + Attributes,
> {
    /// File writing new entries.
    /// There is no need to re-open the file each time.
    file: File,
    /// Opened file UUID.
    file_name: String,
    /// Index to link an ID to a file.
    /// This allows the file to be targeted for modification or deletion.
    index: BTreeMap<String, String>,
    /// TTL manager.
    ttl: Option<Arc<RwLock<TTL<T>>>>,
    /// Data saved on disk.
    pub entries: Vec<T>,
    /// Caching of data to be written to avoid overload and bottlenecks.
    memtable: Vec<T>,
    /// After how many kb the data is written hard to the disk.
    /// Set to 0 to deactivate the memory table.
    memtable_flush_size_in_kb: usize,
    /// MPSC consumer used to know expired sentences.
    /// Created by yourself using [`tokio::sync::mpsc`].
    sender: Option<Sender<T>>,
    phantom: PhantomData<T>,
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
    /// Create a new database instance.
    ///
    /// # Examples
    /// ```rust
    /// use serde::{Deserialize, Serialize};
    /// use squid_db::{Instance, Attributes};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Entity {
    ///     data: String,
    /// }
    ///
    /// impl Attributes for Entity {}
    ///
    /// let instance: Instance<Entity> = Instance::new(0).unwrap();
    /// //... then you can do enything with the instance.
    /// ```
    pub fn new(memtable_flush_size_in_kb: usize) -> Result<Self, DbError> {
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

        Ok(Self {
            file,
            file_name,
            index,
            ttl: None,
            entries: entires.0,
            memtable: Vec::new(),
            memtable_flush_size_in_kb,
            sender: None,
            phantom: PhantomData,
        })
    }

    /// Set [`tokio::sync::mpsc::Sender`] to send expire
    /// event in channel.
    pub fn sender(&mut self, sender: Sender<T>) {
        self.sender = Some(sender);
    }

    /// Start TTL manager.
    /// This can results in higher memory consumption.
    ///
    /// # Examples
    /// ```no_run,rust
    /// use serde::{Deserialize, Serialize};
    /// use squid_db::{Instance, Attributes};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Entity {
    ///     id: String,
    ///     data: String,
    ///     love: bool,
    ///     lifetime: u64,
    /// }
    ///
    /// impl Attributes for Entity {
    ///     fn id(&self) -> String {
    ///         self.id.clone()
    ///     }
    ///
    ///     fn ttl(&self) -> Option<u64> {
    ///         Some(self.lifetime)
    ///     }
    /// }
    ///
    /// let mut instance: Instance<Entity> = Instance::new(0).unwrap();
    ///
    /// instance.set(Entity {
    ///     id: "U1".to_string(),
    ///     data: "I do not know if my french teaher like me...".to_string(),
    ///     love: false,
    ///     lifetime: 0, // permanent sentence.
    /// });
    ///
    /// instance.set(Entity {
    ///     id: "U2".to_string(),
    ///     data: "It starts with A! My love?".to_string(),
    ///     love: true,
    ///     lifetime: 500, // because love only lasts 500 seconds.
    /// });
    ///
    /// instance.start_ttl();
    /// ```
    pub async fn start_ttl(self) -> Arc<AsyncRwLock<Instance<T>>> {
        let this = Arc::new(AsyncRwLock::new(self));
        let ttl_manager =
            Arc::new(RwLock::new(ttl::TTL::new(Arc::clone(&this))));

        let (ttl, instance) = (Arc::clone(&ttl_manager), Arc::clone(&this));
        tokio::task::spawn(async move {
            for entry in &instance.write().await.entries {
                if let Some(expire) = entry.ttl() {
                    let _ = ttl.write().unwrap().add_entry(entry.id(), expire);
                }
            }
        });

        ttl_manager.write().unwrap().init();
        this.write().await.ttl = Some(ttl_manager);

        this
    }

    /// d
    pub fn get(&self, id: String) -> Result<Option<T>, DbError> {
        if let Some(file_name) = self.index.get(&id) {
            let data = load_file::<T>(file_name.to_string())?.0;

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
    /// use squid_db::{Instance, Attributes};
    ///
    /// #[derive(Serialize, Deserialize)]
    /// struct Entity {
    ///     data: String,
    ///     love_him: bool,
    /// }
    ///
    /// impl Attributes for Entity {}
    ///
    /// let mut instance: Instance<Entity> = Instance::new(0).unwrap();
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
        if let Some(timestamp) = data.ttl() {
            self.ttl
                .as_ref()
                .and_then(|ttl| ttl.write().ok())
                .map(|mut ttl| ttl.add_entry(data.id(), timestamp))
                .transpose()?;
        }

        #[cfg(feature = "logging")]
        trace!(id = data.id(), "Added new entry.");

        match self.memtable_flush_size_in_kb {
            0 => {
                #[cfg(not(feature = "compress"))]
                let encoded = bincode::serialize(&data)
                    .map_err(|_| DbError::FailedSerialization)?;

                self.index.insert(data.id(), self.file_name.clone());
                self.save(&encoded)?
            },
            max_kb_size => {
                self.memtable.push(data);

                if max_kb_size
                    < (self.memtable.len() * std::mem::size_of::<T>()) / 1000
                {
                    self.flush().map_err(|_| DbError::Unspecified)?
                }
            },
        }

        Ok(())
    }

    /// Deletes a record from the data based on its unique identifier.
    pub fn delete(&mut self, id: &str) -> Result<(), DbError> {
        if let Some(file_name) = self.index.get(id) {
            let file =
                File::open(PathBuf::from(SOURCE_DIRECTORY).join(file_name))
                    .map_err(|_| DbError::FailedReading)?;
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
                    .map_err(|_| DbError::Unspecified)?;

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
    fn save(&mut self, buf: &[u8]) -> Result<(), DbError> {
        let mut line_count = io::BufReader::new(&self.file).lines().count();
        let mut buffer: Vec<u8> = vec![];

        buffer.extend_from_slice(buf);
        buffer.extend_from_slice(b"\n");

        self.file
            .write_all(&buffer)
            .map_err(|_| DbError::Unspecified)?;

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
    pub fn flush(&mut self) -> Result<(), DbError> {
        let line_count = io::BufReader::new(&self.file).lines().count();

        if line_count + self.memtable.len() > MAX_ENTRIES_PER_FILE {
            // If we just write all, number of lines will exceed maximum allowed.
            // So, we will split into two different files.
            let mut buffer: Vec<u8> = Vec::with_capacity(self.memtable.len());

            let mut file_limit = MAX_ENTRIES_PER_FILE - line_count;
            for n in 0..file_limit {
                let data = &self.memtable[n];

                buffer.extend_from_slice(
                    &bincode::serialize(&data)
                        .map_err(|_| DbError::FailedSerialization)?,
                );
                buffer.extend_from_slice(b"\n");

                // Insert new hard entry into index.
                self.index.insert(data.id(), self.file_name.clone());
            }

            self.file
                .write_all(&buffer)
                .map_err(|_| DbError::Unspecified)?;
            self.file.flush().map_err(|_| DbError::Unspecified)?;

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

                buffer.extend_from_slice(
                    &bincode::serialize(&data)
                        .map_err(|_| DbError::FailedSerialization)?,
                );
                buffer.extend_from_slice(b"\n");

                // Insert new hard entry into index.
                self.index.insert(data.id(), self.file_name.clone());
            }

            self.file
                .write_all(&buffer)
                .map_err(|_| DbError::Unspecified)?;
        } else {
            let mut buffer: Vec<u8> = Vec::with_capacity(self.memtable.len());

            for data in &self.memtable {
                buffer.extend_from_slice(
                    &bincode::serialize(&data)
                        .map_err(|_| DbError::FailedSerialization)?,
                );
                buffer.extend_from_slice(b"\n");

                // Insert new hard entry into index.
                self.index.insert(data.id(), self.file_name.clone());
            }

            self.file
                .write_all(&buffer)
                .map_err(|_| DbError::Unspecified)?;

            self.memtable.clear();
        }

        Ok(())
    }
}

/// Loads a specific data file rather than the whole set.
#[inline(always)]
fn load_file<T>(mut name: String) -> Result<World<T>, DbError>
where
    T: serde::de::DeserializeOwned
        + serde::Serialize
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
        .map_err(|_| DbError::Unspecified)?;

    let reader = BufReader::new(&file);
    let mut world: World<T> = World(Vec::new());

    for line in reader.lines() {
        let line_data: T = bincode::deserialize(
            line.map_err(|_| DbError::FailedReading)?.as_bytes(),
        )
        .map_err(|_| DbError::FailedDeserialization)?;

        world.0.push(line_data);
    }

    Ok(world)
}

/// Loads data from the file.
#[inline(always)]
fn load<T>(
) -> Result<(World<T>, BTreeMap<String, String>, Option<File>, String), DbError>
where
    T: serde::de::DeserializeOwned
        + serde::Serialize
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
        .map_err(|_| DbError::FailedReading)?
        .collect::<Result<Vec<_>, io::Error>>()
        .map_err(|_| DbError::FailedReading)?
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
                    .map_err(|_| DbError::Unspecified)?,
            );
            file_name = entry.file_name().into_string().unwrap_or_default();
        }

        world.0.append(&mut data);
    }

    Ok((world, index, uncomplete_file, file_name))
}
