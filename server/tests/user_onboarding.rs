mod macros;
mod setup;

use setup::pool;
use tower::ServiceExt;

#[tokio::test]
async fn onboarding_flow() {
    let pool = pool().await;

    let login = || {
        request!(
            POST "/login";
            "content-type" => "application/x-www-form-urlencoded";
            "username=user1&password=pass1&remember=false"
        )
    };

    let signup = || {
        request!(
            POST "/signup";
            "content-type" => "application/x-www-form-urlencoded";
            "username=user1&password=pass1"
        )
    };

    t!( send!(pool login)  => status!(404) );
    t!( send!(pool signup) => status!(201) );
    t!( send!(pool login)  => status!(200) );
}

#[tokio::test]
async fn double_signup() {
    let pool = pool().await;

    let signup = || {
        request!(
            POST "/signup";
            "content-type" => "application/x-www-form-urlencoded";
            "username=user1&password=pass1"
        )
    };

    t!( send!(pool signup) => status!(201) );
    t!( send!(pool signup) => status!(409) );
}
