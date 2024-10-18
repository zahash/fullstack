use crate::{
    fixture,
    shared::{
        request::{login, signup},
        setup::pool,
    },
};

#[tokio::test]
async fn sample_fixture() {
    let pool = pool().await;

    fixture! {
        pool;
        signup("user1", "pass1");
        login("user1", "pass1");
    }
}
