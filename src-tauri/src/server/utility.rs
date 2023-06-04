use std::{fs, path::Path};

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

// move a file
fn move_file(source_path: &str, destination_path: &str) -> Result<(), std::io::Error> {
    // Create the destination directory if it doesn't exist
    let destination_dir = Path::new(destination_path).parent().unwrap();
    if !destination_dir.exists() {
        fs::create_dir_all(destination_dir)?;
    }

    // Move the file
    fs::rename(source_path, destination_path)?;
    Ok(())
}
