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

    status!( pool 404 login() );
    status!( pool 201 signup() );
    status!( pool 200 login() );
}
