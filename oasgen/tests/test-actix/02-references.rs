use actix_web::web::Json;
use oasgen::{openapi, OaSchema, Server};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, OaSchema)]
pub struct VecRequestItem {
    pub foo: String,
}

#[derive(Serialize, OaSchema)]
pub struct VecResponseItem {
    pub foo: bool,
}

#[derive(Deserialize, OaSchema)]
pub struct MapRequestItem {
    pub foo: String,
}

#[derive(Serialize, OaSchema)]
pub struct MapResponseItem {
    pub foo: bool,
}

#[derive(Deserialize, OaSchema)]
pub struct SendCode {
    pub mobile: String,
}

#[derive(Serialize, OaSchema)]
pub struct SendCodeResponse {
    pub found_account: bool,
}

#[openapi]
async fn send_code(_body: Json<SendCode>) -> Json<SendCodeResponse> {
    Json(SendCodeResponse {
        found_account: false,
    })
}

#[openapi]
async fn vec_of_map(
    _body: Json<Vec<HashMap<i32, MapRequestItem>>>,
) -> Json<Vec<HashMap<i32, MapResponseItem>>> {
    Json(vec![[(1, MapResponseItem { foo: false })].into()])
}

fn main() {
    use pretty_assertions::assert_eq;
    let server = Server::actix()
        .post("/code", send_code)
        .post("/vec_of_map", vec_of_map)
        .freeze();
    let spec = serde_yaml::to_string(&*server.openapi).unwrap();
    assert_eq!(spec.trim(), include_str!("02-references.yaml"));
}
