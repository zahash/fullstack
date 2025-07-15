mod access_token;
mod email;
mod health;
mod login;
mod logout;
mod private;
mod signup;
mod username;

pub use access_token::{check_access_token, generate_access_token};
pub use email::check_email_availability;
pub use health::{health, sysinfo};
pub use login::login;
pub use logout::logout;
pub use private::private;
pub use signup::signup;
pub use username::check_username_availability;

#[cfg(feature = "smtp")]
mod verify_email;

#[cfg(feature = "smtp")]
pub use verify_email::{check_email_verification_token, initiate_email_verification};
