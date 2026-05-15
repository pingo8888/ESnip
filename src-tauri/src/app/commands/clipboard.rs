use std::borrow::Cow;

use arboard::{Clipboard, ImageData};
use base64::{engine::general_purpose, Engine as _};

#[tauri::command]
pub(crate) fn copy_png_to_clipboard(png_base64: String) -> Result<(), String> {
    let png_bytes = general_purpose::STANDARD
        .decode(png_base64.trim())
        .map_err(|error| error.to_string())?;
    let rgba_image = image::load_from_memory_with_format(&png_bytes, image::ImageFormat::Png)
        .map_err(|error| error.to_string())?
        .to_rgba8();
    let (width, height) = rgba_image.dimensions();

    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
    clipboard
        .set_image(ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Owned(rgba_image.into_raw()),
        })
        .map_err(|error| error.to_string())
}
