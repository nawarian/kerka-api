use image::codecs::jpeg::JpegEncoder;
use qrcode_generator::QrCodeEcc;
use std::collections::HashMap;
use std::io::Cursor;

use worker::{Headers, Request, Response, RouteContext};

#[derive(Clone)]
enum QrCodeFormat {
    INVALID,
    JPG,
    PNG,
    SVG,
}

#[derive(Clone)]
struct QrCodePayload {
    pub text: String,
    pub format: QrCodeFormat,
}

fn generate_qrcode(payload: QrCodePayload) -> Result<Vec<u8>, &'static str> {
    let buf: Result<Vec<u8>, &str> = match payload.format {
        QrCodeFormat::JPG => {
            let img = qrcode_generator::to_image_buffer(payload.text, QrCodeEcc::Low, 1024)
                .expect("failed to generate Qr Code.");

            let mut bytes: Vec<u8> = Vec::new();
            let encoder = JpegEncoder::new_with_quality(Cursor::new(&mut bytes), 85);
            let _ = img.write_with_encoder(encoder);

            Ok(bytes)
        }
        QrCodeFormat::PNG => {
            let png = qrcode_generator::to_png_to_vec_from_str(payload.text, QrCodeEcc::Low, 1024);

            if png.is_ok() {
                Ok(png.unwrap())
            } else {
                Err("Failed to generate PNG image")
            }
        }
        QrCodeFormat::SVG => Ok(qrcode_generator::to_svg_to_string(
            payload.text,
            QrCodeEcc::Low,
            1024,
            None::<&str>,
        )
        .unwrap()
        .into()),
        _ => Err("Invalid image format requested"),
    };

    if buf.is_ok() {
        return buf;
    } else {
        return Err("Something went wrong");
    }
}

pub async fn handle_get_qrcode_format(
    req: Request,
    ctx: RouteContext<()>,
) -> worker::Result<Response> {
    let url = req.url().expect("Failed to fetch request's URL params");
    let query: HashMap<String, String> = url.query_pairs().into_owned().collect();

    let text_content = query.get("t").expect("Param 't' is mandatory");

    let mut headers = Headers::new();

    let default_fmt = String::from("default");
    let format = ctx.param("format").unwrap_or(&default_fmt).as_str();

    let payload = QrCodePayload {
        text: text_content.to_string(),
        format: match format {
            "jpg" => QrCodeFormat::JPG,
            "png" => QrCodeFormat::PNG,
            "svg" => QrCodeFormat::SVG,
            _ => QrCodeFormat::INVALID,
        },
    };

    if matches!(payload.format, QrCodeFormat::INVALID) {
        return Response::error("Invalid image format", 503);
    }

    let buff = generate_qrcode(payload.clone());
    match buff.is_ok() {
        true => {
            let _ = headers.append(
                "content-type",
                match payload.format {
                    QrCodeFormat::JPG => "image/jpeg",
                    QrCodeFormat::PNG => "image/png",
                    QrCodeFormat::SVG => "image/svg",
                    QrCodeFormat::INVALID => "image/invalid",
                },
            );
            let _ = headers.append("cache-control", "public, max-age=432000, immutable"); // 5 days cache

            Ok(Response::from_bytes(buff.unwrap())
                .unwrap()
                .with_headers(headers))
        }
        false => Response::error("Something went wrong", 503),
    }
}
