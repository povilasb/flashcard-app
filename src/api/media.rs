//! Validated image uploads saved alongside the media served by Axum.

use axum::{extract::Multipart, http::StatusCode, Json};
use image::{ImageFormat, ImageReader, Limits};
use serde::Serialize;
use std::{
    io::{Cursor, Write},
    path::Path,
};

pub const MEDIA_DIR: &str = "db/media";
pub const MAX_UPLOAD_BYTES: usize = 10 * 1024 * 1024;

type UploadError = (StatusCode, String);

#[derive(Serialize)]
pub struct UploadResponse {
    filename: String,
}

pub async fn upload(
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), UploadError> {
    let field = multipart
        .next_field()
        .await
        .map_err(|e| (e.status(), e.to_string()))?
        .ok_or((StatusCode::BAD_REQUEST, "An image field is required".into()))?;
    if field.name() != Some("image") {
        return Err((StatusCode::BAD_REQUEST, "Expected an image field".into()));
    }
    let bytes = field
        .bytes()
        .await
        .map_err(|e| (e.status(), e.to_string()))?;
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, "Image exceeds 10 MiB".into()));
    }
    if multipart
        .next_field()
        .await
        .map_err(|e| (e.status(), e.to_string()))?
        .is_some()
    {
        return Err((StatusCode::BAD_REQUEST, "Upload one image at a time".into()));
    }

    let filename = tokio::task::spawn_blocking(move || {
        let extension = validate_image(&bytes)?;
        save_image(Path::new(MEDIA_DIR), &bytes, extension).map_err(internal_error)
    })
    .await
    .map_err(internal_error)??;

    Ok((StatusCode::CREATED, Json(UploadResponse { filename })))
}

fn validate_image(bytes: &[u8]) -> Result<&'static str, UploadError> {
    let format = image::guess_format(bytes).map_err(|_| {
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Expected PNG, JPEG, GIF, or WebP".into(),
        )
    })?;
    let extension = match format {
        ImageFormat::Png => "png",
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Gif => "gif",
        ImageFormat::WebP => "webp",
        _ => {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Expected PNG, JPEG, GIF, or WebP".into(),
            ))
        }
    };
    let mut limits = Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(64 * 1024 * 1024);
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    reader.decode().map_err(|_| {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            "Invalid image or image exceeds decoding limits (8192 px per side, 64 MiB)".into(),
        )
    })?;
    Ok(extension)
}

fn save_image(directory: &Path, bytes: &[u8], extension: &str) -> std::io::Result<String> {
    std::fs::create_dir_all(directory)?;
    let filename = format!("{}.{}", uuid::Uuid::new_v4(), extension);
    let path = directory.join(&filename);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    if let Err(error) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(error);
    }
    Ok(filename)
}

fn internal_error(error: impl std::fmt::Display) -> UploadError {
    log::error!("Image upload failed: {error}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Unable to save image".into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_content_and_saves_unique_files() {
        let mut data = Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(1, 1)
            .write_to(&mut data, ImageFormat::Png)
            .unwrap();
        let directory = std::env::temp_dir().join(uuid::Uuid::new_v4().to_string());

        let extension = validate_image(data.get_ref()).unwrap();
        let first = save_image(&directory, data.get_ref(), extension).unwrap();
        let second = save_image(&directory, data.get_ref(), extension).unwrap();

        assert_eq!(extension, "png");
        assert_ne!(first, second);
        assert_eq!(
            std::fs::read(directory.join(first)).unwrap(),
            *data.get_ref()
        );
        assert!(validate_image(b"<svg onload='alert(1)'></svg>").is_err());
        assert!(validate_image(b"\x89PNG\r\n\x1a\ntruncated").is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
