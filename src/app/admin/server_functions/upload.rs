//! Reading and validating one screenshot upload.
//!
//! Nothing the browser says about the file is trusted. The filename and the
//! declared content type are ignored entirely, and neither is stored: the
//! format and the dimensions are whatever the decoder finds in the bytes.
//! Decoder limits are installed before the image is decoded, so a small file
//! that describes an enormous image is refused instead of allocated.

use crate::{
    app::admin::error::AdminError,
    domain::{
        MAX_SCREENSHOT_BYTES, MAX_SCREENSHOT_EDGE, ProjectSlug, ScreenshotAltText,
        ScreenshotMediaType, ScreenshotSize, ScreenshotSizeError,
    },
};
use image::{ImageError, ImageFormat, ImageReader, Limits};
use leptos::server_fn::codec::MultipartData;
use std::io::Cursor;

/// Four bytes for every pixel of the largest image the limits allow, which is
/// what decoding a [`MAX_SCREENSHOT_EDGE`] square into RGBA needs. The unit
/// test below keeps it in step with that edge.
const MAX_DECODED_BYTES: u64 = 4 * 4096 * 4096;

/// The form fields one upload carries. The image is last, so the parts that
/// decide whether it is wanted at all arrive first.
const SLUG_FIELD: &str = "slug";
const ALT_FIELD: &str = "alt";
const IMAGE_FIELD: &str = "image";

/// One upload after validation: the Project it belongs to, its description,
/// and the original bytes with the format and size the decoder confirmed.
pub struct UploadedScreenshot {
    pub slug: ProjectSlug,
    pub alt: ScreenshotAltText,
    pub media_type: ScreenshotMediaType,
    pub size: ScreenshotSize,
    pub bytes: Vec<u8>,
}

/// Reads one screenshot from a multipart body.
///
/// # Errors
///
/// Returns the Owner-facing reason the upload was refused. No variant carries
/// a decoder or database message.
pub async fn read(data: MultipartData) -> Result<UploadedScreenshot, AdminError> {
    let mut parts = data.into_inner().ok_or(AdminError::Internal)?;
    let mut slug = None;
    let mut alt = None;
    let mut bytes = None;

    while let Some(field) = parts
        .next_field()
        .await
        .map_err(|_| AdminError::UnreadableScreenshot)?
    {
        let name = field.name().map(str::to_owned);
        match name.as_deref() {
            Some(SLUG_FIELD) => {
                slug = Some(field.text().await.map_err(|_| AdminError::Internal)?);
            }
            Some(ALT_FIELD) => {
                alt = Some(field.text().await.map_err(|_| AdminError::Internal)?);
            }
            Some(IMAGE_FIELD) => {
                bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| AdminError::UnreadableScreenshot)?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let slug = slug
        .ok_or(AdminError::ProjectNotFound)?
        .parse::<ProjectSlug>()
        .map_err(|_| AdminError::ProjectNotFound)?;
    let alt = alt
        .unwrap_or_default()
        .parse::<ScreenshotAltText>()
        .map_err(|_| AdminError::InvalidAltText)?;

    let bytes = bytes.filter(|bytes| !bytes.is_empty());
    let bytes = bytes.ok_or(AdminError::NoScreenshotChosen)?;
    if bytes.len() > MAX_SCREENSHOT_BYTES {
        return Err(AdminError::ScreenshotTooLarge);
    }
    let (media_type, size) = decoded(&bytes)?;

    Ok(UploadedScreenshot {
        slug,
        alt,
        media_type,
        size,
        bytes,
    })
}

/// The format and dimensions the bytes actually hold.
///
/// The image is decoded rather than merely inspected, so a file with a valid
/// header and a broken body is refused before it is stored. The original bytes
/// are what gets stored; nothing here re-encodes them.
fn decoded(bytes: &[u8]) -> Result<(ScreenshotMediaType, ScreenshotSize), AdminError> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| AdminError::UnreadableScreenshot)?;

    let media_type = match reader.format() {
        Some(ImageFormat::Png) => ScreenshotMediaType::Png,
        Some(ImageFormat::Jpeg) => ScreenshotMediaType::Jpeg,
        Some(ImageFormat::WebP) => ScreenshotMediaType::Webp,
        _ => return Err(AdminError::UnsupportedScreenshotFormat),
    };

    let mut limits = Limits::no_limits();
    limits.max_image_width = Some(MAX_SCREENSHOT_EDGE);
    limits.max_image_height = Some(MAX_SCREENSHOT_EDGE);
    limits.max_alloc = Some(MAX_DECODED_BYTES);
    reader.limits(limits);
    let image = reader.decode().map_err(|error| match error {
        ImageError::Limits(_) => AdminError::ScreenshotTooManyPixels,
        _ => AdminError::UnreadableScreenshot,
    })?;

    let size =
        ScreenshotSize::try_from((image.width(), image.height())).map_err(|error| match error {
            ScreenshotSizeError::TooLarge => AdminError::ScreenshotTooManyPixels,
            ScreenshotSizeError::Empty => AdminError::UnreadableScreenshot,
        })?;
    Ok((media_type, size))
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err_eq, assert_ok};

    /// A one-pixel opaque PNG.
    const PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    /// The smallest GIF, standing in for every format the portfolio does not
    /// serve. It is a valid image, so only the format check rejects it.
    const GIF: &[u8] = &[
        0x47, 0x49, 0x46, 0x38, 0x39, 0x61, 0x01, 0x00, 0x01, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00,
        0x00, 0xFF, 0xFF, 0xFF, 0x21, 0xF9, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x2C, 0x00, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x02, 0x02, 0x44, 0x01, 0x00, 0x3B,
    ];

    #[test]
    fn the_allocation_limit_covers_the_largest_allowed_image() {
        assert_eq!(
            MAX_DECODED_BYTES,
            4 * u64::from(MAX_SCREENSHOT_EDGE) * u64::from(MAX_SCREENSHOT_EDGE)
        );
    }

    #[test]
    fn a_png_reports_its_own_format_and_size() {
        let (media_type, size) = assert_ok!(decoded(PNG));

        assert_eq!(media_type, ScreenshotMediaType::Png);
        assert_eq!((size.width(), size.height()), (1, 1));
    }

    #[test]
    fn an_unsupported_format_is_named_as_such() {
        assert_err_eq!(decoded(GIF), AdminError::UnsupportedScreenshotFormat);
    }

    /// A file that starts like a PNG but carries nothing usable has to fail on
    /// its contents, not on its extension or its declared type.
    #[test]
    fn a_truncated_image_is_unreadable() {
        let truncated = PNG.get(..24).unwrap_or_default();

        assert_err_eq!(decoded(truncated), AdminError::UnreadableScreenshot);
    }

    #[test]
    fn text_that_is_not_an_image_is_unsupported() {
        assert_err_eq!(
            decoded(b"<svg xmlns='http://www.w3.org/2000/svg'></svg>"),
            AdminError::UnsupportedScreenshotFormat
        );
    }
}
