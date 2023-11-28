use qrcode_generator::QrCodeEcc;
use worker::*;
use serde::{ Deserialize, Serialize };

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let r = Router::new();

    r.post_async("/v1/qrcode/:format", generate_qrcode)
        .run(req, env).await
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct GenerateQrCodeInput {
    textContent: String,
    bgColor: String,
    fgColor: String,
}

async fn generate_qrcode(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let input = req.json::<GenerateQrCodeInput>().await?;

    let cors = Cors::new()
        .with_allowed_headers(vec!["https://kerka.com.br"])
        .with_methods(vec![Method::Post]);

    let mut headers = Headers::new();

    let default_fmt = String::from("default");
    let format = ctx.param("format").unwrap_or(&default_fmt).as_str();
    match format {
        "svg" => {
            let buff = qrcode_generator::to_svg_to_string(input.textContent, QrCodeEcc::Low, 256, None::<&str>).unwrap();
            let _ = headers.append("content-type", "image/svg");

            Ok(Response::ok(buff).unwrap().with_cors(&cors).unwrap().with_headers(headers))
        },
        "png" => {
            let buff = qrcode_generator::to_png_to_vec_from_str(input.textContent, QrCodeEcc::Low, 256).unwrap();
            let _ = headers.append("content-type", "image/png");

            Ok(Response::from_bytes(buff).unwrap().with_cors(&cors).unwrap().with_headers(headers))
        },
        _ => Response::error("Failed to generate Qr Code.", 400)
    }
}
