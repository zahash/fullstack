use std::{hash::Hash, sync::Arc};

use dashmap::{DashMap, DashSet};
use tokio::sync::Notify;

/// A simple lock manager that manages exclusive locks for values of type `T`.
///
/// The `LockManager` ensures that only one task can hold a lock on a value `T` at a time.
/// This structure is particularly useful for implementing a locking mechanism
/// that allows tasks to wait for a lock to be released rather than failing
/// immediately when a lock cannot be acquired.
pub struct LockManager<T> {
    locks: DashSet<T>,
    waiters: DashMap<T, Arc<Notify>>,
}

impl<T: Eq + Hash> Default for LockManager<T> {
    fn default() -> Self {
        Self {
            locks: Default::default(),
            waiters: Default::default(),
        }
    }
}

impl<T: Clone + Eq + Hash> LockManager<T> {
    /// Acquires a lock for the given value.
    ///
    /// If the lock is already held by another task, the current task will wait asynchronously
    /// until the lock is released. If the lock is available, it is immediately acquired.
    pub async fn lock(&self, val: &T) {
        if self.locks.insert(val.clone()) {
            return;
        }

        let notify = self
            .waiters
            .entry(val.clone())
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone();

        notify.notified().await;

        self.locks.insert(val.clone());
    }
}

impl<T: Eq + Hash> LockManager<T> {
    /// Releases the lock for the given value.
    ///
    /// Once the lock is released, any tasks waiting for this lock will be notified
    /// and the next waiting task can acquire the lock.
    pub fn release(&self, val: &T) {
        if self.locks.remove(val).is_some() {
            if let Some((_, notify)) = self.waiters.remove(val) {
                notify.notify_one();
            }
        }
    }
}
