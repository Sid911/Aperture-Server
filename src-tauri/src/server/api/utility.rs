use rocket::log::private::info;
use rocket_multipart_form_data::FileField;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn get_file_meta(path: &std::path::PathBuf) -> Option<std::fs::Metadata> {
    if let Ok(metadata) = std::fs::metadata(path) {
        if metadata.is_file() {
            return Some(metadata);
        }
    }
    None
}

pub async fn save_file_to_documents(
    file: &FileField,
    file_name: &str,
    relative_path: &str,
    device_id: &str,
) -> Option<std::path::PathBuf> {
    let documents_dir = match get_documents_dir().await {
        Ok(dir) => dir,
        Err(e) => {
            error!("Failed to get documents directory: {}", e);
            return None;
        }
    };

    let device_dir = documents_dir.join("Aperture").join(device_id);
    let target_dir = device_dir.join(relative_path);

    if let Err(e) = tokio::fs::create_dir_all(&target_dir).await {
        error!("Failed to create target directory: {}", e);
        return None;
    }

    let target_file_path = target_dir.join(file_name);
    let mut target_file = match tokio::fs::File::create(&target_file_path).await {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to create target file: {}", e);
            return None;
        }
    };

    let file_path = &file.path;
    let mut reader = match tokio::fs::File::open(file_path).await {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open source file: {}", e);
            return None;
        }
    };
    let mut buffer = vec![0; 4096];
    loop {
        match reader.read(&mut buffer).await {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    break;
                }
                if let Err(e) = target_file.write_all(&buffer[..bytes_read]).await {
                    error!("Failed to write to target file: {}", e);
                    return None;
                }
            }
            Err(e) => {
                error!("Failed to read from source file: {}", e);
                return None;
            }
        }
    }

    Some(target_file_path.clone())
}

pub fn is_image_file(file_path: &std::path::Path) -> bool {
    info!("is_image_file : {:?}",file_path);
    let file_extension = file_path
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_lowercase();
    let image_extensions = vec!["jpg", "jpeg", "png", " "]; // Add more image extensions if needed
    image_extensions.contains(&file_extension.as_str())
}

pub async fn generate_blurhash(file_path: &std::path::Path) -> Option<String> {
    let mut file = tokio::fs::File::open(file_path).await.unwrap();
    let mut image_data = Vec::new();
    file.read_to_end(&mut image_data).await.unwrap();

    let width = 32;
    let height = 32;
    let components_x = 4;
    let components_y = 3;

    return Some(blurhash::encode(
        components_x,
        components_y,
        width,
        height,
        &image_data,
    ));
}

pub async fn get_documents_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    match dirs::document_dir() {
        Some(documents_dir) => Ok(documents_dir),
        None => Err("Failed to get documents directory".into()),
    }
}
