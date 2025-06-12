use server_core::DataAccess;

use crate::{
    fixture,
    shared::request::{login, signup},
};

pub async fn _sample_fixture(data_access: &DataAccess) {
    fixture! {
        data_access;
        signup("user1", "user1@test.com", "pass1");
        login("user1", "pass1");
    }
}
