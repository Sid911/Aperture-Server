use std::fs::{self, Metadata};

use mime::Mime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
// #[serde("flatten")]
pub struct LocalEntry {
    pub file_uuid: String,
    pub file_name: String,
    pub file_size: u64,
    pub dir_path: String,
    pub client_path: String,
    pub relative_path: String,
    pub blurhash: Option<String>,
    pub file_location: String,
    pub metadata: SerializedMetadata,
}

impl LocalEntry {
    pub fn new(
        device_uuid: String,
        file_name: String,
        file_size: u64,
        file_location: String,
        mime: Option<Mime>,
        blurhash: Option<String>,
        dir_path: String,
        client_path: String,
        relative_path: String,
    ) -> Self {
        let metadata = fs::metadata(file_location.clone()).unwrap();
        let serialized_meta = SerializedMetadata::from(metadata, mime);
        Self {
            file_uuid: device_uuid,
            file_name,
            file_size,
            file_location,
            metadata: serialized_meta,
            blurhash,
            dir_path,
            client_path,
            relative_path
        }
    }

    fn get_metadata(&self) -> &SerializedMetadata {
        &self.metadata
    }

    fn set_metadata(&mut self, metadata: SerializedMetadata) {
        self.metadata = metadata;
    }
}

#[cfg(unix)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedMetadata {
    file_type: Option<String>,
    modified: Option<u64>,
    accessed: Option<u64>,
    created: Option<u64>,
    len: Option<u64>,
}

#[cfg(unix)]
impl SerializedMetadata {
    pub fn from(metadata: Metadata, mime: Option<Mime>) -> Self {
        let file_type = mime.map(|ft| ft.to_string());

        SerializedMetadata {
            file_type,
            modified: metadata
                .modified()
                .ok()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
            accessed: metadata
                .accessed()
                .ok()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
            created: metadata
                .created()
                .ok()
                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()),
            len: Some(metadata.len()),
        }
    }
}

// This thing is not tested yet on windows
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

#[cfg(windows)]
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedMetadata {
    file_type: Option<String>,
    modified: Option<u64>,
    accessed: Option<u64>,
    created: Option<u64>,
    len: Option<u64>,
}

#[cfg(windows)]
impl SerializedMetadata {
    pub fn from(metadata: Metadata, mime: Option<Mime>) -> Self {
        let file_type = mime.map(|ft| ft.to_string());

        SerializedMetadata {
            file_type,
            modified: Some(metadata.last_write_time()),
            accessed: Some(metadata.last_access_time()),
            created: Some(metadata.creation_time()),
            len: Some(metadata.file_size()),
        }
    }
}
