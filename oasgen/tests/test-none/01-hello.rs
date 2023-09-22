use oasgen::{openapi, OaSchema, Server};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, OaSchema)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

/// Send a code to /hello
#[openapi(operation_id = "getHello")]
async fn send_code(_body: SendCode) -> SendCodeResponse {
    SendCodeResponse {
        found_account: false,
    }
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::none().get("/hello", send_code).freeze();

    let spec = serde_yaml::to_string(&*server.openapi).unwrap();
    assert_eq!(spec.trim(), include_str!("01-hello.yaml"));
}
