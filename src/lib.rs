mod endpoints;

use worker::{event, Context, Cors, Env, Method, Request, Response, Result, Router};

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
        .get_async(
            "/v1/qrcode/:format",
            endpoints::qrcode::handle_get_qrcode_format,
        )
        .run(req, env)
        .await?
        .with_cors(&cors)
}
