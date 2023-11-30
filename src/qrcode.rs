use image::codecs::jpeg::JpegEncoder;
use qrcode_generator::QrCodeEcc;
use std::io::Cursor;

#[derive(Clone)]
pub enum QrCodeFormat {
    INVALID,
    JPG,
    PNG,
    SVG,
}

#[derive(Clone)]
pub struct QrCodePayload {
    pub text: String,
    pub format: QrCodeFormat,
}

pub fn generate_qrcode(payload: QrCodePayload) -> Result<Vec<u8>, &'static str> {
    let buf: Result<Vec<u8>, &str> = match payload.format {
        QrCodeFormat::JPG => {
            let img = qrcode_generator::to_image_buffer(payload.text, QrCodeEcc::Low, 1024)
                .expect("failed to generate Qr Code.");

            let mut bytes: Vec<u8> = Vec::new();
            let encoder = JpegEncoder::new_with_quality(Cursor::new(&mut bytes), 85);
            let _ = img.write_with_encoder(encoder);

            Ok(bytes)
        },
        QrCodeFormat::PNG => {
            let png = qrcode_generator::to_png_to_vec_from_str(payload.text, QrCodeEcc::Low, 1024);

            if png.is_ok() {
                Ok(png.unwrap())
            } else {
                Err("Failed to generate PNG image")
            }
        },
        QrCodeFormat::SVG => {
            Ok(qrcode_generator::to_svg_to_string(payload.text, QrCodeEcc::Low, 1024, None::<&str>)
                .unwrap().into())
        },
        _ => Err("Invalid image format requested")
    };

    if buf.is_ok() {
        return buf;
    } else {
        return Err("Something went wrong");
    }
}

