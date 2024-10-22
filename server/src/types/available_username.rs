use std::sync::{Arc, LazyLock};

use anyhow::Context;
use sqlx::SqlitePool;

use crate::{
    check::username_exists,
    error::{AuthError, HandlerError},
    misc::LockManager,
    types::Username,
};

/// A struct representing a username that has passed availability checks and is locked for use.
///
/// The `AvailableUsername` struct ensures that a username is not in the database
/// and temporarily locked for the duration of its usage. This lock is necessary to avoid race conditions
/// during username registration processes, where multiple requests might try to register the same username
/// simultaneously.
///
/// ### Purpose of the Lock
///
/// The purpose of the lock is **not** to verify the username's existence in the database. That check is
/// performed separately by querying the database. Instead, this lock is a **temporary in-memory mechanism**
/// that ensures once a username is confirmed to be available (i.e., it does not exist in the database),
/// it is locked and prevented from being validated by other tasks or requests until the current operation
/// completes.
///
/// Without this lock, two or more simultaneous requests could check the database, see that the username
/// is available, and attempt to register the same username at nearly the same time, causing a race condition.
/// By locking the username after it is validated, we ensure that while the `AvailableUsername` exists,
/// no other part of the system can check or attempt to register that username. The lock is released
/// automatically when the `AvailableUsername` instance is dropped.
///
/// ### Separation of Concerns
/// - **Database Check**: The database is responsible for determining if the username already exists
/// in persistent storage.
/// - **Locking Mechanism**: The lock exists in-memory and serves only to prevent other parts of the
/// application from operating on the same username while it is being processed.
///
/// The lock prevents race conditions between multiple requests by serializing access to the username.
/// Once a username is locked and is being processed, all other attempts to check or register the same
/// username must wait asynchronously until the lock is released, ensuring there is no overlap or conflict.
pub struct AvailableUsername(Username);

/// A global lock manager for username locks, ensuring that only one task can
/// acquire the lock for a given username at any time. The lock manager is
/// initialized lazily and is shared across the application.
static USERNAME_LOCKS: LazyLock<Arc<LockManager<Username>>> = LazyLock::new(|| Default::default());

impl AvailableUsername {
    /// Acquires a lock on the given username and checks its availability.
    ///
    /// This function ensures that while a username is being validated for availability,
    /// no other task can attempt to use or validate the same username.
    /// If the username exists in the database, the lock is released, and an error is returned.
    pub async fn acquire(pool: &SqlitePool, username: Username) -> Result<Self, HandlerError> {
        USERNAME_LOCKS.lock(&username).await;

        if username_exists(pool, &username)
            .await
            .context("AvailableUsername")?
        {
            USERNAME_LOCKS.release(&username);
            return Err(AuthError::UsernameTaken(username).into());
        }

        Ok(Self(username))
    }
}

impl Drop for AvailableUsername {
    /// Releases the lock on the username when the `AvailableUsername` instance is dropped.
    ///
    /// This ensures that other tasks can acquire the lock for this username after
    /// this instance goes out of scope.
    fn drop(&mut self) {
        USERNAME_LOCKS.release(&self.0);
    }
}
