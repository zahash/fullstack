mod shared;

use shared::{
    request::{login, signup},
    setup::data_access,
};
use test_proc_macros::{email, password, username};

#[tokio::test]
async fn onboarding_flow() {
    let username = username!("user1");
    let email = email!("user1@test.com");
    let password = password!("Aa!1aaaa");

    let data_access = data_access().await;

    t!( send!(data_access login(username, password))  => status!(401) );
    t!( send!(data_access signup(username, email, password)) => status!(201) );
    t!( send!(data_access login(username, password))  => status!(200) );
}

#[tokio::test]
async fn double_signup() {
    let username = username!("user1");
    let email = email!("user1@test.com");
    let password = password!("Aa!1aaaa");

    let data_access = data_access().await;

    t!( send!(data_access signup(username, email, password)) => status!(201) );
    t!( send!(data_access signup(username, email, password)) => status!(409) );
}

#[tokio::test]
async fn username_taken() {
    let username = username!("user1");

    let email1 = email!("user_1@test.com");
    let email2 = email!("user_2@test.com");

    let password1 = password!("Aa!1aaaa");
    let password2 = password!("Bb!2bbbb");

    let data_access = data_access().await;

    fixture! {
        data_access;
        signup(username, email1, password1);
    }

    t!( send!(data_access signup(username, email2, password2)) => status!(409) );
}

#[tokio::test]
async fn email_taken() {
    let data_access = data_access().await;

    let email = email!("user3@test.com");

    let username1 = username!("user3a");
    let username2 = username!("user3b");

    let password1 = password!("Aa!1aaaa");
    let password2 = password!("Bb!2bbbb");

    fixture! {
        data_access;
        signup(username1, email, password1);
    }

    t!( send!(data_access signup(username2, email, password2)) => status!(409) );
}
