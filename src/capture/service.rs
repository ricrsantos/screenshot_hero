use ashpd::desktop::screenshot::Screenshot;
use ashpd::desktop::ResponseError;
use ashpd::Error as AshpdError;

use crate::capture::loader::FileLoader;
use crate::models::ImageData;

#[derive(Debug, PartialEq, Eq)]
pub enum CaptureError {
    PortalUnavailable(String),
    PortalCancelled,
    ImageLoadFailed(String),
}

pub struct CaptureService;

impl CaptureService {
    pub async fn capture() -> Result<Option<ImageData>, CaptureError> {
        let request = Screenshot::request()
            .interactive(true)
            .send()
            .await
            .map_err(|e| CaptureError::PortalUnavailable(e.to_string()))?;

        let screenshot = match request.response() {
            Ok(screenshot) => screenshot,
            Err(AshpdError::Response(ResponseError::Cancelled)) => return Ok(None),
            Err(e) => return Err(CaptureError::PortalUnavailable(e.to_string())),
        };

        process_portal_uri(screenshot.uri().as_str())
    }
}

/// Processes the URI returned by the screenshot portal.
///
/// Returns `Ok(None)` when the URI is empty (user cancelled without a portal error).
pub(crate) fn process_portal_uri(uri: &str) -> Result<Option<ImageData>, CaptureError> {
    if uri.is_empty() {
        return Ok(None);
    }

    FileLoader::load_from_uri(uri)
        .map(Some)
        .map_err(load_error_message)
}

fn load_error_message(error: crate::capture::loader::LoadError) -> CaptureError {
    use crate::capture::loader::LoadError;

    let message = match error {
        LoadError::FileNotFound(path) => format!("file not found: {}", path.display()),
        LoadError::UnsupportedFormat(path) => format!("unsupported format: {}", path.display()),
        LoadError::DecodeFailed(message) => message,
        LoadError::InvalidUri(message) => message,
    };

    CaptureError::ImageLoadFailed(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_portal_uri_empty_returns_none() {
        let result = process_portal_uri("").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_process_portal_uri_invalid_returns_image_load_failed() {
        let result = process_portal_uri("not-a-valid-uri");
        assert!(matches!(result, Err(CaptureError::ImageLoadFailed(_))));
    }

    #[test]
    fn test_process_portal_uri_nonexistent_file_returns_image_load_failed() {
        let result = process_portal_uri("file:///tmp/screenshot-hero-nonexistent-test.png");
        assert!(matches!(result, Err(CaptureError::ImageLoadFailed(_))));
    }
}
