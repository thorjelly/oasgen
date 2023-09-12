use axum::body::{Body, Full};
use axum::handler::Handler;
use axum::routing;
use axum::routing::MethodRouter;
use futures::future::{ok, Ready};
use http::Method;
use indexmap::IndexMap;
use openapiv3::OpenAPI;
use std::borrow::Borrow;
use std::sync::Arc;

use oasgen_core::{OaOperation, OaSchema};

use crate::Format;

use super::Server;

macro_rules! route_method {
    ($variant:ident, $method:ident) => {
        pub fn $method<F, T, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: Handler<T, S, Body>,
            T: 'static,
            F: OaOperation<Signature> + Copy + Send,
        {
            self.add_handler_to_spec(path, Method::$variant, &handler);
            self.add_route(path, routing::$method(handler));
            self
        }
    };
}

pub struct Router<S>(IndexMap<String, MethodRouter<S>>);

impl<S> Default for Router<S> {
    fn default() -> Self {
        Self(IndexMap::new())
    }
}

impl<S> Server<Router<S>, OpenAPI>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn axum() -> Self {
        Self::new()
    }

    fn add_route(&mut self, path: &str, route: MethodRouter<S>) {
        if path.contains('{') {
            eprintln!(
                "WARNING: Path parameters are specified with `:name` with axum, not `{{name}}`."
            );
        }
        match self.router.0.get_mut(path) {
            Some(method_router) => {
                let existing = std::mem::take(method_router);
                *method_router = existing.merge(route);
            }
            None => {
                self.router.0.insert(path.to_string(), route);
            }
        }
    }

    route_method!(GET, get);
    route_method!(POST, post);
    route_method!(PUT, put);
    route_method!(DELETE, delete);
    route_method!(HEAD, head);
    route_method!(OPTIONS, options);
    route_method!(TRACE, trace);
    route_method!(PATCH, patch);
}

impl<S> Server<Router<S>, Arc<OpenAPI>>
where
    S: Clone + Send + Sync + 'static,
{
    pub fn into_router(self) -> axum::Router<S> {
        use axum::response::IntoResponse;

        let mut router = axum::Router::new();
        for (path, inner) in self.router.0 {
            router = router.route(&path, inner);
        }

        if let Some(json_route) = &self.json_route {
            let spec = self.openapi.as_ref();
            let bytes = serde_json::to_vec(spec).unwrap();
            router = router.route(
                &json_route,
                routing::get(|| async {
                    (
                        [(
                            http::header::CONTENT_TYPE,
                            http::HeaderValue::from_str("application/json").unwrap(),
                        )],
                        bytes,
                    )
                        .into_response()
                }),
            );
        }

        if let Some(yaml_route) = &self.yaml_route {
            let spec = self.openapi.as_ref();
            let yaml = serde_yaml::to_string(spec).unwrap();
            router = router.route(
                &yaml_route,
                routing::get(|| async {
                    (
                        [(
                            http::header::CONTENT_TYPE,
                            http::HeaderValue::from_str("text/yaml").unwrap(),
                        )],
                        yaml,
                    )
                        .into_response()
                }),
            );
        }

        #[cfg(feature = "swagger-ui")]
        if let Some(mut path) = self.swagger_ui_route {
            let swagger = self
                .swagger_ui
                .expect("Swagger UI route set but no Swagger UI is configured.");
            let handler = routing::get(|uri: http::Uri| async move {
                match swagger.handle_url(&uri) {
                    Some(response) => {
                        let (headers, body) = response.into_parts();
                        axum::response::Response::from_parts(
                            headers,
                            axum::body::boxed(Full::from(body)),
                        )
                    }
                    None => axum::response::Response::builder()
                        .status(http::StatusCode::NOT_FOUND)
                        .body(axum::body::boxed(Body::empty()))
                        .unwrap(),
                }
            });
            router = router.route(&format!("{}", &path), handler.clone());
            router = router.route(&format!("{}*rest", &path), handler)
        }
        router
    }
}
