mod shared;

use shared::{
    request::{login, signup},
    setup::pool,
};

#[tokio::test]
async fn onboarding_flow() {
    let login = || login("user1", "Aa!1aaaa");
    let signup = || signup("user1", "user1@test.com", "Aa!1aaaa");

    let pool = pool().await;

    t!( send!(pool login())  => status!(401) );
    t!( send!(pool signup()) => status!(201) );
    t!( send!(pool login())  => status!(200) );
}

#[tokio::test]
async fn double_signup() {
    let signup = || signup("user1", "user1@test.com", "Aa!1aaaa");

    let pool = pool().await;

    t!( send!(pool signup()) => status!(201) );
    t!( send!(pool signup()) => status!(409) );
}
