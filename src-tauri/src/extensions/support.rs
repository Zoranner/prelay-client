use std::{fs, io::Write, path::Path};

use atomic_write_file::AtomicWriteFile;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use prelay_protocol::ExtensionFile;

use crate::relay::client::ClientError;

pub(crate) fn decode_extension_file(file: &ExtensionFile) -> Result<Vec<u8>, ClientError> {
    BASE64.decode(&file.content_base64).map_err(|_| {
        ClientError::new(
            "invalid_response",
            "extension install bundle contains invalid Base64 content",
        )
    })
}

pub(crate) fn atomic_write(path: &Path, contents: &[u8]) -> Result<(), ClientError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(storage_error)?;
    }
    let mut file = AtomicWriteFile::open(path).map_err(storage_error)?;
    file.write_all(contents).map_err(storage_error)?;
    file.commit().map_err(storage_error)
}

pub(crate) fn storage_error(error: std::io::Error) -> ClientError {
    ClientError::new(
        "local_extensions_error",
        format!("无法写入扩展文件：{error}"),
    )
}
