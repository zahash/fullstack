mod shared;

use shared::{
    request::{login, signup},
    setup::pool,
};
use test_proc_macros::{email, password, username};

#[tokio::test]
async fn wrong_password() {
    let username = username!("user1");
    let email = email!("user1@test.com");

    let password = password!("Aa!1aaaa");
    let wrong_password = password!("Bb!2bbbb");

    let pool = pool().await;

    fixture!(
        pool;
        signup(username, email, password);
    );

    t!( send!(pool login(username, wrong_password)) => status!(401) );
}

#[tokio::test]
async fn user_not_found() {
    let username = username!("user1");
    let password = password!("Aa!1aaaa");

    let pool = pool().await;

    t!( send!(pool login(username, password)) => status!(401) );
}
