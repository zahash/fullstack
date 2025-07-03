mod shared;

use shared::{
    request::{login, signup},
    setup::data_access,
};
use test_proc_macros::{email, password, username};

#[tokio::test]
async fn wrong_password() {
    let username = username!("user1");
    let email = email!("user1@test.com");

    let password = password!("Aa!1aaaa");
    let wrong_password = password!("Bb!2bbbb");

    let data_access = data_access().await;

    fixture!(
        data_access;
        signup(username, email, password);
    );

    t!( send!(data_access login(username, wrong_password)) => status!(401) );
}

#[tokio::test]
async fn user_not_found() {
    let username = username!("user1");
    let password = password!("Aa!1aaaa");

    let data_access = data_access().await;

    t!( send!(data_access login(username, password)) => status!(401) );
}
