use image::codecs::jpeg::JpegEncoder;
use qrcode_generator::QrCodeEcc;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use worker::{
    event, Context, Cors, Env, Headers, Method, Request, Response, Result, RouteContext, Router,
};

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let is_prod = match env.var("APP_ENV")?.to_string().as_str() {
        "development" | "test" => false,
        _ => true,
    };

    let mut cors = Cors::default()
        .with_max_age(86400)
        .with_origins(vec!["https://kerka.com.br", "http://localhost:3000"])
        .with_allowed_headers(vec!["content-type"])
        .with_exposed_headers(vec!["content-type"])
        .with_methods(vec![Method::Post, Method::Options]);

    if !is_prod {
        cors = cors.with_origins(vec!["*://localhost:*"])
    }

    Router::new()
        .options("/v1/qrcode/:format", |_, _| Response::ok("Hello, friend"))
        .post_async("/v1/qrcode/:format", generate_qrcode)
        .run(req, env)
        .await?
        .with_cors(&cors)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GenerateQrCodeInput {
    text_content: String,
    bg_color: String,
    fg_color: String,
}

async fn generate_qrcode(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let input = req.json::<GenerateQrCodeInput>().await?;

    let mut headers = Headers::new();

    let default_fmt = String::from("default");
    let format = ctx.param("format").unwrap_or(&default_fmt).as_str();
    match format {
        "svg" => {
            let buff = qrcode_generator::to_svg_to_string(
                input.text_content,
                QrCodeEcc::Low,
                256,
                None::<&str>,
            )
            .unwrap();
            let _ = headers.append("content-type", "image/svg");

            Ok(Response::ok(buff).unwrap().with_headers(headers))
        }
        "png" => {
            let buff =
                qrcode_generator::to_png_to_vec_from_str(input.text_content, QrCodeEcc::Low, 256)
                    .unwrap();
            let _ = headers.append("content-type", "image/png");

            Ok(Response::from_bytes(buff).unwrap().with_headers(headers))
        }
        "jpg" | "jpeg" => {
            let img = qrcode_generator::to_image_buffer(input.text_content, QrCodeEcc::Low, 256)
                .expect("failed to generate Qr Code.");

            let mut bytes: Vec<u8> = Vec::new();
            let encoder = JpegEncoder::new_with_quality(Cursor::new(&mut bytes), 85);
            let _ = img.write_with_encoder(encoder);

            let _ = headers.append("content-type", "image/jpeg");

            Ok(Response::from_bytes(bytes)
                .expect("Failed to generate Qr Code.")
                .with_headers(headers))
        }
        _ => Response::error("Failed to generate Qr Code.", 400),
    }
}
