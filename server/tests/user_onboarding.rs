mod shared;

use shared::{
    request::{login, signup},
    setup::pool,
};
use test_proc_macros::{email, password, username};

#[tokio::test]
async fn onboarding_flow() {
    #[cfg(feature = "tracing")]
    shared::setup::tracing_init();

    let username = username!("user1");
    let email = email!("user1@test.com");
    let password = password!("Aa!1aaaa");

    let pool = pool().await;

    t!( send!(pool login(username, password))  => status!(401) );
    t!( send!(pool signup(username, email, password)) => status!(201) );
    t!( send!(pool login(username, password))  => status!(200) );
}

#[tokio::test]
async fn double_signup() {
    #[cfg(feature = "tracing")]
    shared::setup::tracing_init();

    let username = username!("user1");
    let email = email!("user1@test.com");
    let password = password!("Aa!1aaaa");

    let pool = pool().await;

    t!( send!(pool signup(username, email, password)) => status!(201) );
    t!( send!(pool signup(username, email, password)) => status!(409) );
}

#[tokio::test]
async fn username_taken() {
    #[cfg(feature = "tracing")]
    shared::setup::tracing_init();

    let username = username!("user1");

    let email1 = email!("user_1@test.com");
    let email2 = email!("user_2@test.com");

    let password1 = password!("Aa!1aaaa");
    let password2 = password!("Bb!2bbbb");

    let pool = pool().await;

    fixture! {
        pool;
        signup(username, email1, password1);
    }

    t!( send!(pool signup(username, email2, password2)) => status!(409) );
}

#[tokio::test]
async fn email_taken() {
    #[cfg(feature = "tracing")]
    shared::setup::tracing_init();

    let pool = pool().await;

    let email = email!("user3@test.com");

    let username1 = username!("user3a");
    let username2 = username!("user3b");

    let password1 = password!("Aa!1aaaa");
    let password2 = password!("Bb!2bbbb");

    fixture! {
        pool;
        signup(username1, email, password1);
    }

    t!( send!(pool signup(username2, email, password2)) => status!(409) );
}
