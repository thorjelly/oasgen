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

#[openapi]
async fn vec(_body: Json<Vec<VecRequestItem>>) -> Json<Vec<VecResponseItem>> {
    Json(vec![VecResponseItem { foo: false }])
}

#[openapi]
async fn map(_body: Json<HashMap<i32, MapRequestItem>>) -> Json<HashMap<i32, MapResponseItem>> {
    Json([(1, MapResponseItem { foo: false })].into())
}

fn main() {
    let server = Server::actix().post("/vec", vec).post("/map", map).freeze();

    let spec = serde_yaml::to_string(&*server.openapi).unwrap();
    assert_eq!(spec.trim(), include_str!("02-references.yaml"));
}
