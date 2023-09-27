#![allow(unused)]

// We have to wrap the example in `mod` beacuse examples fail compilation without a `main`, and
// forwarding to an inner mod fixes the issue.
#[cfg(feature = "actix")]
mod inner {
    use actix_web::{http::Method, web, App, HttpResponse, HttpServer};
    use oasgen::{openapi, OaSchema, Server};
    use serde::{Deserialize, Serialize};
    use std::fs::File;

    #[derive(Deserialize, OaSchema, Debug)]
    pub struct Echo {
        pub echo: String,
    }

    #[derive(OaSchema, Deserialize)]
    pub struct SendCode {
        pub mobile: String,
    }

    #[derive(OaSchema, Deserialize)]
    pub struct VerifyCode {
        pub mobile: String,
        pub code: String,
    }

    #[derive(Serialize, OaSchema, Debug)]
    pub struct SendCodeResponse {
        pub found_account: bool,
    }

    #[derive(Serialize, OaSchema, Debug)]
    pub struct EchoResponse {
        pub echo: String,
    }

    #[openapi]
    async fn get_echo(query: web::Query<Echo>) -> web::Json<EchoResponse> {
        web::Json(EchoResponse {
            echo: query.echo.clone(),
        })
    }

    #[openapi]
    async fn post_echo(query: web::Query<Echo>) -> HttpResponse {
        println!("{}", query.echo);
        HttpResponse::Ok().finish()
    }

    #[openapi]
    async fn send_code(_body: web::Json<SendCode>) -> web::Json<SendCodeResponse> {
        web::Json(SendCodeResponse {
            found_account: false,
        })
    }

    #[openapi]
    async fn verify_code(_body: web::Json<VerifyCode>) -> HttpResponse {
        HttpResponse::Ok().finish()
    }

    #[tokio::main]
    pub async fn main() -> std::io::Result<()> {
        let host = "0.0.0.0";
        let port = 5000;
        let host = format!("{}:{}", host, port);

        let server = Server::actix()
            .get("/echo", get_echo)
            .post("/echo", post_echo)
            .post("/send-code", send_code)
            .post("/verify-code", verify_code)
            .route_json_spec("/openapi.json")
            .route_yaml_spec("/openapi.yaml")
            .write_and_exit_if_env_var_set("./openapi.yaml")
            .freeze();

        println!("Listening on {}", host);
        HttpServer::new(move || {
            let spec = server.openapi.clone();
            App::new()
                .route(
                    "/healthcheck",
                    web::get().to(|| async { HttpResponse::Ok().body("Ok") }),
                )
                .service(server.clone().into_service())
        })
        .bind(host)?
        .run()
        .await
    }
}

fn main() {
    #[cfg(feature = "actix")]
    inner::main().unwrap()
}
