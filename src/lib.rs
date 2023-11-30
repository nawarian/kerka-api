mod qrcode;

use std::collections::HashMap;
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
        .with_exposed_headers(vec!["content-type"])
        .with_methods(vec![Method::Get, Method::Options]);

    Router::new()
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

    let payload  = qrcode::QrCodePayload {
        text: text_content.to_string(),
        format: match format {
            "jpg" => qrcode::QrCodeFormat::JPG,
            "png" => qrcode::QrCodeFormat::PNG,
            "svg" => qrcode::QrCodeFormat::SVG,
            _ => qrcode::QrCodeFormat::INVALID,
        },
    };

    if matches!(payload.format, qrcode::QrCodeFormat::INVALID) {
        return Response::error("Invalid image format", 503);
    }

    let buff = qrcode::generate_qrcode(payload.clone());
    match buff.is_ok() {
        true => {
            let _ = headers.append("content-type", match payload.format {
                qrcode::QrCodeFormat::JPG => "image/jpeg",
                qrcode::QrCodeFormat::PNG => "image/png",
                qrcode::QrCodeFormat::SVG => "image/svg",
                qrcode::QrCodeFormat::INVALID => "image/invalid",
            });
            let _ = headers.append("cache-control", "public, max-age=432000, immutable"); // 5 days cache

            Ok(Response::from_bytes(buff.unwrap()).unwrap().with_headers(headers))
        },
        false => Response::error("Something went wrong", 503)
    }
}
