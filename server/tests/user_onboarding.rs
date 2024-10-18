mod shared;

use shared::{
    request::{login, signup},
    setup::pool,
};

#[tokio::test]
async fn onboarding_flow() {
    let login = login("user1", "pass1");
    let signup = signup("user1", "pass1");

    let pool = pool().await;

    t!( send!(pool login)  => status!(404) );
    t!( send!(pool signup) => status!(201) );
    t!( send!(pool login)  => status!(200) );
}

#[tokio::test]
async fn double_signup() {
    let signup = signup("user1", "pass1");

    let pool = pool().await;

    t!( send!(pool signup) => status!(201) );
    t!( send!(pool signup) => status!(409) );
}
