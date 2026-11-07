use serde::Serialize;

const MAX_IMAGE_BYTES: usize = 2 * 1024 * 1024;
const MAX_IMAGE_SIDE: u32 = 2_000;
const MAX_IMAGE_PIXELS: u64 = 4_000_000;

#[derive(Debug, Clone, Serialize)]
pub struct VisionWord {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OcrOutput {
    pub text: String,
    pub words: Vec<VisionWord>,
}

fn validate_png(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() < 24
        || bytes.len() > MAX_IMAGE_BYTES
        || bytes.get(..8) != Some(b"\x89PNG\r\n\x1a\n")
        || bytes.get(12..16) != Some(b"IHDR")
    {
        return Err("Screen OCR requires a PNG no larger than 2 MiB.".into());
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap_or_default());
    let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap_or_default());
    if width == 0
        || height == 0
        || width > MAX_IMAGE_SIDE
        || height > MAX_IMAGE_SIDE
        || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS
    {
        return Err(
            "Screen OCR image dimensions exceed the 2,000-pixel or 4-megapixel limit.".into(),
        );
    }
    Ok(())
}

#[cfg(windows)]
mod windows_ocr {
    use windows::{
        Graphics::Imaging::BitmapDecoder,
        Media::Ocr::OcrEngine,
        Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
        Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
    };

    use super::{OcrOutput, VisionWord};

    struct OcrInitGuard;

    impl Drop for OcrInitGuard {
        fn drop(&mut self) {
            unsafe { RoUninitialize() };
        }
    }

    pub(super) fn recognize_png(bytes: &[u8]) -> Result<OcrOutput, String> {
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }
            .map_err(|_| "Windows OCR could not initialize on this device.".to_string())?;
        let _guard = OcrInitGuard;

        let stream = InMemoryRandomAccessStream::new()
            .map_err(|_| "Windows OCR could not create an image stream.".to_string())?;
        let writer = DataWriter::CreateDataWriter(
            &stream
                .GetOutputStreamAt(0)
                .map_err(|_| "Windows OCR could not open the image stream.".to_string())?,
        )
        .map_err(|_| "Windows OCR could not create an image writer.".to_string())?;
        writer
            .WriteBytes(bytes)
            .map_err(|_| "Windows OCR could not read the PNG bytes.".to_string())?;
        writer
            .StoreAsync()
            .map_err(|_| "Windows OCR could not prepare the image.".to_string())?
            .join()
            .map_err(|_| "Windows OCR could not prepare the image.".to_string())?;
        writer
            .FlushAsync()
            .map_err(|_| "Windows OCR could not flush the image.".to_string())?
            .join()
            .map_err(|_| "Windows OCR could not flush the image.".to_string())?;
        stream
            .Seek(0)
            .map_err(|_| "Windows OCR could not seek the image stream.".to_string())?;

        let decoder = BitmapDecoder::CreateAsync(&stream)
            .map_err(|_| "Windows OCR could not decode the PNG.".to_string())?
            .join()
            .map_err(|_| "Windows OCR could not decode the PNG.".to_string())?;
        let bitmap = decoder
            .GetSoftwareBitmapAsync()
            .map_err(|_| "Windows OCR could not load the decoded image.".to_string())?
            .join()
            .map_err(|_| "Windows OCR could not load the decoded image.".to_string())?;
        let engine = OcrEngine::TryCreateFromUserProfileLanguages().map_err(|_| {
            "Windows OCR is unavailable. Check that a supported language pack is installed."
                .to_string()
        })?;
        let result = engine
            .RecognizeAsync(&bitmap)
            .map_err(|_| "Windows OCR could not analyze the image.".to_string())?
            .join()
            .map_err(|_| "Windows OCR could not analyze the image.".to_string())?;

        let mut words = Vec::new();
        let lines = result
            .Lines()
            .map_err(|_| "Windows OCR returned no readable text.".to_string())?;
        for line_index in 0..lines.Size().unwrap_or(0) {
            let line = lines
                .GetAt(line_index)
                .map_err(|_| "Windows OCR returned invalid text regions.".to_string())?;
            let line_words = line
                .Words()
                .map_err(|_| "Windows OCR returned invalid text regions.".to_string())?;
            for word_index in 0..line_words.Size().unwrap_or(0) {
                let word = line_words
                    .GetAt(word_index)
                    .map_err(|_| "Windows OCR returned invalid text regions.".to_string())?;
                let rect = word
                    .BoundingRect()
                    .map_err(|_| "Windows OCR returned invalid text bounds.".to_string())?;
                let text = word
                    .Text()
                    .map_err(|_| "Windows OCR returned invalid text.".to_string())?
                    .to_string();
                if !text.trim().is_empty() && words.len() < 2_000 {
                    words.push(VisionWord {
                        text,
                        x: f64::from(rect.X),
                        y: f64::from(rect.Y),
                        width: f64::from(rect.Width),
                        height: f64::from(rect.Height),
                    });
                }
            }
        }
        let text = result
            .Text()
            .map_err(|_| "Windows OCR returned no readable text.".to_string())?
            .to_string();
        Ok(OcrOutput { text, words })
    }
}

pub fn recognize_png(bytes: &[u8]) -> Result<OcrOutput, String> {
    validate_png(bytes)?;
    #[cfg(windows)]
    {
        windows_ocr::recognize_png(bytes)
    }
    #[cfg(not(windows))]
    {
        Err("Windows OCR is available only in the Windows desktop build.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_IMAGE_BYTES, validate_png};

    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0; 24];
        bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        bytes[12..16].copy_from_slice(b"IHDR");
        bytes[16..20].copy_from_slice(&width.to_be_bytes());
        bytes[20..24].copy_from_slice(&height.to_be_bytes());
        bytes
    }

    #[test]
    fn screen_ocr_rejects_bad_or_oversized_images() {
        assert!(validate_png(&png_header(1_920, 1_080)).is_ok());
        assert!(validate_png(&png_header(0, 100)).is_err());
        assert!(validate_png(&png_header(2_001, 100)).is_err());
        assert!(validate_png(&png_header(2_000, 2_000)).is_ok());
        assert!(validate_png(&vec![0; MAX_IMAGE_BYTES + 1]).is_err());
    }
}
