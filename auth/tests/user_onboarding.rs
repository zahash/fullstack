mod shared;

use shared::TestClient;
use test_proc_macros::{email, password, username};

#[tokio::test]
async fn onboarding_flow() {
    #[cfg(feature = "tracing")]
    shared::tracing_init();

    let username = username!("user1");
    let email = email!("user1@test.com");
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
            format!("username={}&password={}", username, password)
        ))
        .await
        .status(200);
}

#[tokio::test]
async fn double_signup() {
    #[cfg(feature = "tracing")]
    shared::tracing_init();

    let username = username!("user1");
    let email = email!("user1@test.com");
    let password = password!("Aa!1aaaa");

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
            POST "/signup";
            "host" => "localhost"
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&email={}&password={}", username, email, password)
        ))
        .await
        .status(409);
}

#[tokio::test]
async fn username_taken() {
    #[cfg(feature = "tracing")]
    shared::tracing_init();

    let username = username!("user1");

    let email1 = email!("user_1@test.com");
    let email2 = email!("user_2@test.com");

    let password1 = password!("Aa!1aaaa");
    let password2 = password!("Bb!2bbbb");

    let mut client = TestClient::default().await;

    client
        .send(request!(
            POST "/signup";
            "host" => "localhost"
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&email={}&password={}", username, email1, password1)
        ))
        .await
        .status(201);

    client
        .send(request!(
            POST "/signup";
            "host" => "localhost"
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&email={}&password={}", username, email2, password2)
        ))
        .await
        .status(409)
        .json_body::<serde_json::Value>(|body| {
            assert_eq!(
                body.get("kind"),
                Some(&serde_json::Value::from("username.exists"))
            );
        })
        .await;
}

#[tokio::test]
async fn email_taken() {
    #[cfg(feature = "tracing")]
    shared::tracing_init();

    let mut client = TestClient::default().await;

    let email = email!("user3@test.com");

    let username1 = username!("user3a");
    let username2 = username!("user3b");

    let password1 = password!("Aa!1aaaa");
    let password2 = password!("Bb!2bbbb");

    client
        .send(request!(
            POST "/signup";
            "host" => "localhost"
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&email={}&password={}", username1, email, password1)
        ))
        .await
        .status(201);

    client
        .send(request!(
            POST "/signup";
            "host" => "localhost"
            "content-type" => "application/x-www-form-urlencoded";
            format!("username={}&email={}&password={}", username2, email, password2)
        ))
        .await
        .status(409)
        .json_body::<serde_json::Value>(|body| {
            assert_eq!(
                body.get("kind"),
                Some(&serde_json::Value::from("email.exists"))
            );
        })
        .await;
}
