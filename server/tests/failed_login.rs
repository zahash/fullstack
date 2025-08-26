mod shared;

use shared::TestClient;
use test_proc_macros::{email, password, username};

#[tokio::test]
async fn wrong_password() {
    #[cfg(feature = "tracing")]
    shared::tracing_init();

    let username = username!("user1");
    let email = email!("user1@test.com");

    let password = password!("Aa!1aaaa");
    let wrong_password = password!("Bb!2bbbb");

    let mut client = TestClient::default().await;

    client
        .send(request!(
            POST "/signup";
            "host" => "localhost"
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&email={}&password={}", username, email, password)
        ))
        .await
        .status(201);

    client
        .send(request!(
            POST "/login";
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&password={}", username, wrong_password)
        ))
        .await
        .status(401);
}

#[tokio::test]
async fn user_not_found() {
    #[cfg(feature = "tracing")]
    shared::tracing_init();

    let username = username!("user1");
    let password = password!("Aa!1aaaa");

    let mut client = TestClient::default().await;

    client
        .send(request!(
            POST "/login";
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&password={}", username, password)
        ))
        .await
        .status(401);
}
