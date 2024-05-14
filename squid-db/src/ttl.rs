//! Handle time to live (TTL) from saved sentences.
//!
//! It divides the sentences to be deleted in this hour into time blocks.
//!
//! After a periodic check, usually every hour, if there are recordings in the
//! current hour, a task is launched to delete the expired recording to the
//! nearest second.

use crate::{Attributes, DbError, Instance};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread::sleep,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::RwLock as AsyncRwLock;

const SECONDS_IN_HOUR: u64 = 3600;

#[derive(Debug, Clone)]
#[allow(unused)]
struct Entry {
    id: String,
    exact_expiration: u64,
}

#[derive(Debug, Clone)]
pub struct TTL<
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static,
> {
    periods: Arc<RwLock<HashMap<u64, Vec<Entry>>>>,
    instance: Arc<AsyncRwLock<Instance<T>>>,
}

impl<T> TTL<T>
where
    T: serde::Serialize
        + serde::de::DeserializeOwned
        + Attributes
        + std::marker::Send
        + std::marker::Sync
        + 'static,
{
    pub fn new(instance: Arc<AsyncRwLock<Instance<T>>>) -> Self {
        Self {
            instance,
            periods: Arc::new(RwLock::new(HashMap::default())),
        }
    }

    pub fn add_entry(
        &mut self,
        id: String,
        timestamp: u64,
    ) -> Result<(), DbError> {
        let actual_hour = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if actual_hour >= timestamp {
            // Remove expired entry.
            let instance = Arc::clone(&self.instance);
            tokio::task::spawn(async move {
                if let Some(sender) = &instance.read().await.sender {
                    if let Ok(Some(data)) =
                        instance.read().await.get(id.clone())
                    {
                        let _ = sender.send(data).await;
                    }
                }
                let _ = instance.write().await.delete(&id);
            });
        } else if actual_hour / SECONDS_IN_HOUR == timestamp / SECONDS_IN_HOUR {
            let instance = Arc::clone(&self.instance);
            tokio::task::spawn(async move {
                sleep(Duration::from_secs(timestamp - actual_hour));

                if let Some(sender) = &instance.read().await.sender {
                    if let Ok(Some(data)) =
                        instance.read().await.get(id.clone())
                    {
                        let _ = sender.send(data).await;
                    }
                }
                let _ = instance.write().await.delete(&id);
            });
        } else {
            self.periods
                .write()
                .map_err(|_| DbError::FailedWriting)?
                .entry(timestamp / SECONDS_IN_HOUR)
                .and_modify(|e| {
                    e.push(Entry {
                        id: id.clone(),
                        exact_expiration: timestamp,
                    })
                })
                .or_insert(vec![Entry {
                    id,
                    exact_expiration: timestamp,
                }]);
        }

        Ok(())
    }

    #[allow(unreachable_code)]
    fn spawn_timers(&self) {
        let periods = Arc::clone(&self.periods);
        let instance = Arc::clone(&self.instance);

        tokio::task::spawn(async move {
            loop {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                // Sleep until next hour.
                sleep(Duration::from_secs(
                    SECONDS_IN_HOUR - (now % SECONDS_IN_HOUR),
                ));

                if let Some(timers) = periods
                    .read()
                    .map_err(|_| DbError::FailedReading)?
                    .get(&(now / SECONDS_IN_HOUR))
                {
                    for timer in timers {
                        let entry = timer.clone();
                        let instance = Arc::clone(&instance);

                        tokio::task::spawn(async move {
                            sleep(Duration::from_secs(
                                entry.exact_expiration
                                    - SystemTime::now()
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                            ));

                            if let Some(sender) = &instance.read().await.sender
                            {
                                if let Ok(Some(data)) =
                                    instance.read().await.get(entry.id.clone())
                                {
                                    let _ = sender.send(data).await;
                                }
                            }
                            let _ = instance.write().await.delete(&entry.id);
                        });
                    }
                }
            }

            Ok::<(), DbError>(())
        });
    }

    // Starts the periodic check and recent counters.
    pub fn init(&self) {
        self.spawn_timers();
    }
}
