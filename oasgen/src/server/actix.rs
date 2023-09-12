use actix_web::{
    delete, web, Error, FromRequest, Handler, HttpResponse, Resource, Responder, Scope,
};
use futures::future::{ok, Ready};
use http::Method;
use openapiv3::OpenAPI;
use std::sync::{Arc, RwLock};

use oasgen_core::{OaOperation, OaSchema};

use crate::Format;

use super::Server;

#[derive(Default)]
pub struct ActixRouter(Vec<InnerResourceFactory<'static>>);

impl Clone for ActixRouter {
    fn clone(&self) -> Self {
        ActixRouter(self.0.iter().map(|f| f.manual_clone()).collect::<Vec<_>>())
    }
}

/// ResourceFactory is a no-argument closure that returns a user-provided view handler.
///
/// Because `actix_web::Resource : !Clone`, we can't store the `Resource` directly in the `Server`
/// struct (since we need `Server: Clone`, because `Server` is cloned for every server thread by actix_web).
/// This trait essentially adds `Clone` to these closures.
pub trait ResourceFactory<'a>: Send + Fn() -> Resource {
    fn manual_clone(&self) -> InnerResourceFactory<'static>;
}

impl<'a, T> ResourceFactory<'a> for T
where
    T: 'static + Clone + Fn() -> Resource + Send,
{
    fn manual_clone(&self) -> InnerResourceFactory<'static> {
        Box::new(self.clone())
    }
}

pub type InnerResourceFactory<'a> = Box<dyn ResourceFactory<'a, Output = Resource>>;

fn build_inner_resource<F, Args>(
    path: String,
    method: Method,
    handler: F,
) -> InnerResourceFactory<'static>
where
    F: Handler<Args> + 'static + Copy + Send,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
{
    Box::new(move || {
        actix_web::Resource::new(path.clone())
            .route(actix_web::web::route().method(method.clone()).to(handler))
    })
}

macro_rules! route_method {
    ($variant:ident, $method:ident) => {
        pub fn $method<F, Args, Signature>(mut self, path: &str, handler: F) -> Self
        where
            F: actix_web::Handler<Args> + OaOperation<Signature> + Copy + Send,
            Args: actix_web::FromRequest + 'static,
            F::Output: actix_web::Responder + 'static,
            <F as actix_web::Handler<Args>>::Output: OaSchema,
        {
            self.add_handler_to_spec(path, Method::$variant, &handler);
            self.router.0.push(build_inner_resource(
                path.to_string(),
                Method::$variant,
                handler,
            ));
            self
        }
    };
}

impl Server<ActixRouter> {
    pub fn actix() -> Self {
        Self::new()
    }

    route_method!(GET, get);
    route_method!(POST, post);
    route_method!(PUT, put);
    route_method!(DELETE, delete);
    route_method!(HEAD, head);
    route_method!(CONNECT, connect);
    route_method!(OPTIONS, options);
    route_method!(TRACE, trace);
    route_method!(PATCH, patch);
}

impl Server<ActixRouter, Arc<OpenAPI>> {
    pub fn into_service(self) -> Scope {
        let mut scope = web::scope(&self.prefix.unwrap_or_default());
        for resource in self.router.0 {
            scope = scope.service(resource());
        }
        if let Some(path) = self.json_route {
            scope = scope.service(
                web::resource(&path).route(web::get().to(OaSpecJsonHandler(self.openapi.clone()))),
            );
        }
        if let Some(path) = self.yaml_route {
            scope = scope.service(
                web::resource(&path).route(web::get().to(OaSpecYamlHandler(self.openapi.clone()))),
            );
        }
        scope
    }
}

#[derive(Clone)]
struct OaSpecJsonHandler(Arc<openapiv3::OpenAPI>);

impl actix_web::dev::Handler<()> for OaSpecJsonHandler {
    type Output = Result<HttpResponse, Error>;
    type Future = Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        ok(HttpResponse::Ok().json(&*self.0))
    }
}

#[derive(Clone)]
struct OaSpecYamlHandler(Arc<openapiv3::OpenAPI>);

impl actix_web::dev::Handler<()> for OaSpecYamlHandler {
    type Output = Result<HttpResponse, Error>;
    type Future = Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        let yaml = serde_yaml::to_string(&*self.0).unwrap();
        ok(HttpResponse::Ok()
            .insert_header(("Content-Type", "text/yaml"))
            .body(yaml))
    }
}
