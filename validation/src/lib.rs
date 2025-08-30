pub fn validate_username<T: AsRef<str>>(username: T) -> Result<T, &'static str> {
    let username_ref = username.as_ref();

    if username_ref.len() < 2 || username_ref.len() > 30 {
        return Err("username must be between 2-30 in length");
    }

    if username_ref
        .chars()
        .any(|c| !c.is_ascii_alphanumeric() && c != '_')
    {
        return Err("username must only contain `A-Z` `a-z` `0-9` and `_`");
    }

    Ok(username)
}

pub fn validate_password<T: AsRef<str>>(password: T) -> Result<T, &'static str> {
    let password_ref = password.as_ref();

    if !password_ref.chars().any(|c| c.is_lowercase()) {
        return Err("password must contain at least one lowercase letter");
    }

    if !password_ref.chars().any(|c| c.is_uppercase()) {
        return Err("password must contain at least one uppercase letter");
    }

    if !password_ref.chars().any(|c| c.is_ascii_digit()) {
        return Err("password must contain at least one digit");
    }

    if !password_ref
        .chars()
        .any(|c| r#"!@#$%^&*()_-+={}[]|\:;"'<>,.?/~`"#.contains(c))
    {
        return Err("password must contain at least one special character");
    }

    if password_ref.len() < 8 {
        return Err("password must be at least 8 characters long");
    }

    Ok(password)
}
