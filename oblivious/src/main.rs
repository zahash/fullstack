// the server stores data encrypted with public key of the user, along with the user's public key
// users must prove they own the data by solving cryptographic challenge issued by the server

// challenge:
// the server sends a random nonce to the user using a stateless signed token
// the user signs the nonce with their private key, and sends it back to the server
//      along with the original challenge token
// the server gets the signed nonce from the user and verifies it against the nonce
//      in the original challenge token using the user's public key

// use JWK format to send public key

// use std::{env::home_dir, path::PathBuf};

// #[derive(Debug, clap::Parser)]
// struct Serve {
//     /// The port number on which the server will listen for incoming connections.
//     /// Example: `8080`
//     #[arg(long, env("PORT"))]
//     #[cfg_attr(debug_assertions, arg(default_value_t = 8080))]
//     port: u16,

//     /// The directory where the user files are located.
//     /// Example: `./oblivious` or `/var/www/oblivious`
//     #[arg(long, env("DIR"), default_value_os_t = root_dir())]
//     dir: PathBuf,
// }

// fn root_dir() -> PathBuf {
//     home_dir()
//         .map(|p| p.join("oblivious"))
//         .unwrap_or_else(|| PathBuf::from("oblivious"))
// }

mod registration;
mod thumbprint;

fn main() {}

#[inline(always)]
fn exit(err: impl std::error::Error) -> ! {
    eprintln!("{err}");
    std::process::exit(1)
}
