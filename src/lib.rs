use image::codecs::jpeg::JpegEncoder;
use qrcode_generator::QrCodeEcc;
use std::{collections::HashMap, io::Cursor};
use worker::{
    event, Context, Cors, Env, Headers, Method, Request, Response, Result, RouteContext, Router,
};

fn is_origin_allowed(origin: String) -> bool {
    match origin.as_str() {
        "https://kerka.com.br" | "http://localhost:3000" => true,
        _ => false,
    }
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let headers = req.headers();
    let origin = headers
        .get("origin")
        .unwrap_or(Some(String::from("https://kerka.com.br")))
        .unwrap();
    if !is_origin_allowed(origin.clone()) {
        return Response::error("Bad request", 400);
    }

    let cors = Cors::default()
        .with_origins(vec![origin])
        .with_allowed_headers(vec!["content-type"])
        .with_exposed_headers(vec!["content-type"])
        .with_methods(vec![Method::Get, Method::Options]);

    Router::new()
        .options("/v1/qrcode/:format", |_, _| Response::ok("Hello, friend"))
        .get_async("/v1/qrcode/:format", generate_qrcode)
        .run(req, env)
        .await?
        .with_cors(&cors)
}

async fn generate_qrcode(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let url = req.url().expect("Failed to fetch request's URL params");
    let query: HashMap<String, String> = url.query_pairs().into_owned().collect();

    let text_content = query.get("t").expect("Param 't' is mandatory");

    let mut headers = Headers::new();

    let default_fmt = String::from("default");
    let format = ctx.param("format").unwrap_or(&default_fmt).as_str();
    match format {
        "svg" => {
            let buff =
                qrcode_generator::to_svg_to_string(text_content, QrCodeEcc::Low, 256, None::<&str>)
                    .unwrap();
            let _ = headers.append("content-type", "image/svg");
            let _ = headers.append("cache-control", "public, max-age=432000, immutable"); // 5 days cache

            Ok(Response::ok(buff).unwrap().with_headers(headers))
        }
        "png" => {
            let buff = qrcode_generator::to_png_to_vec_from_str(text_content, QrCodeEcc::Low, 1024)
                .unwrap();
            let _ = headers.append("content-type", "image/png");
            let _ = headers.append("cache-control", "public, max-age=432000, immutable"); // 5 days cache

            Ok(Response::from_bytes(buff).unwrap().with_headers(headers))
        }
        "jpg" | "jpeg" => {
            let img = qrcode_generator::to_image_buffer(text_content, QrCodeEcc::Low, 1024)
                .expect("failed to generate Qr Code.");

            let mut bytes: Vec<u8> = Vec::new();
            let encoder = JpegEncoder::new_with_quality(Cursor::new(&mut bytes), 85);
            let _ = img.write_with_encoder(encoder);

            let _ = headers.append("content-type", "image/jpeg");
            let _ = headers.append("cache-control", "public, max-age=432000, immutable"); // 5 days cache

            Ok(Response::from_bytes(bytes)
                .expect("Failed to generate Qr Code.")
                .with_headers(headers))
        }
        _ => Response::error("Failed to generate Qr Code.", 400),
    }
}
