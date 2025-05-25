use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct ValidationResult {
    valid: bool,
    error: Option<String>,
}

#[wasm_bindgen]
impl ValidationResult {
    #[wasm_bindgen(constructor)]
    pub fn new(valid: bool, error: Option<String>) -> ValidationResult {
        ValidationResult { valid, error }
    }

    #[wasm_bindgen(getter)]
    pub fn valid(&self) -> bool {
        self.valid
    }

    #[wasm_bindgen(getter)]
    pub fn error(&self) -> Option<String> {
        self.error.clone()
    }
}

#[wasm_bindgen]
pub fn validate_password(password: &str) -> ValidationResult {
    match validation::validate_password(password) {
        Ok(_) => ValidationResult::new(true, None),
        Err(err) => ValidationResult::new(false, Some(err.to_string())),
    }
}

#[wasm_bindgen]
pub fn validate_username(username: String) -> ValidationResult {
    match validation::validate_username(username) {
        Ok(_) => ValidationResult::new(true, None),
        Err(err) => ValidationResult::new(false, Some(err.to_string())),
    }
}
