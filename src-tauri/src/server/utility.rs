use rocket_multipart_form_data::TextField;
use sha2::{Digest, Sha256};

// Checks all the options if all of them have value or not
pub fn verify_required_data<T>(vector: &[Option<T>]) -> bool {
    let mut has_value = true;
    for field in vector {
        if field.is_none() {
            has_value = false;
            break;
        }
    }
    return has_value;
}

pub trait TextFieldExt {
    fn first_text(&self) -> Option<String>;
}

impl TextFieldExt for Option<&Vec<TextField>> {
    fn first_text(&self) -> Option<String> {
        self.unwrap().first().map(|t| t.text.clone())
    }
}

// hash a string
pub fn gen_sha_256_hash(string: &String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(string.as_bytes());

    let result = hasher.finalize();
    format!("{:x}", result)
}
