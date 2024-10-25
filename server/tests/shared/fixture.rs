use sqlx::SqlitePool;

use crate::{
    fixture,
    shared::request::{login, signup},
};

pub async fn _sample_fixture(pool: &SqlitePool) {
    fixture! {
        pool;
        signup("user1", "user1@test.com", "pass1");
        login("user1", "pass1");
    }
}
