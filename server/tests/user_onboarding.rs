mod shared;

use compiletime::{email, password, username};
use shared::{
    request::{login, signup},
    setup::pool,
};

#[tokio::test]
async fn onboarding_flow() {
    let username = username!("user1");
    let password = password!("Aa!1aaaa");
    let email = email!("user1@test.com");

    let login = || login(username, password);
    let signup = || signup(username, email, password);

    let pool = pool().await;

    t!( send!(pool login())  => status!(401) );
    t!( send!(pool signup()) => status!(201) );
    t!( send!(pool login())  => status!(200) );
}

#[tokio::test]
async fn double_signup() {
    let signup = || {
        signup(
            username!("user1"),
            email!("user1@test.com"),
            password!("Aa!1aaaa"),
        )
    };

    let pool = pool().await;

    t!( send!(pool signup()) => status!(201) );
    t!( send!(pool signup()) => status!(409) );
}
