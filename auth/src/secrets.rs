use std::{fs, io, path::PathBuf};

use rand::RngCore;
use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct Secrets {
    dir: PathBuf,
}

impl Secrets {
    const DEFAULT_N_BYTES: usize = 32;

    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn get(&self, key: &str) -> Result<Zeroizing<Vec<u8>>, io::Error> {
        let path = self.dir.join(key);
        fs::read(path).map(Zeroizing::new)
    }

    pub fn reset(&self, key: &str) -> Result<(), io::Error> {
        let buf = {
            let mut rng = rand::rng();
            let mut buf = vec![0u8; Secrets::DEFAULT_N_BYTES];
            rng.fill_bytes(&mut buf);
            Zeroizing::new(buf)
        };

        let path = self.dir.join(key);
        fs::write(path, &buf)
    }
}
