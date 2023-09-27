use actix_web::{
    delete, web, Error, FromRequest, Handler, HttpResponse, Resource, Responder, Route, Scope,
};
use futures::future::{ok, Ready};
use http::Method;
use openapiv3::OpenAPI;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, RwLock},
};

use oasgen_core::{OaOperation, OaSchema};

use crate::Format;

use super::Server;

#[derive(Default)]
pub struct ActixRouter(HashMap<String, Vec<InnerRouteFactory<'static>>>);

impl Clone for ActixRouter {
    fn clone(&self) -> Self {
        ActixRouter(
            self.0
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.into_iter().map(|r| r.manual_clone()).collect::<Vec<_>>(),
                    )
                })
                .collect::<HashMap<_, _>>(),
        )
    }
}

/// RouteFactory is a no-argument closure that returns a user-provided view handler.
///
/// Because `actix_web::Route : !Clone`, we can't store the `Route` directly in the `Server`
/// struct (since we need `Server: Clone`, because `Server` is cloned for every server thread by actix_web).
/// This trait essentially adds `Clone` to these closures.
pub trait RouteFactory<'a>: Send + Fn() -> Route {
    fn manual_clone(&self) -> InnerRouteFactory<'static>;
}

impl<'a, T> RouteFactory<'a> for T
where
    T: 'static + Clone + Fn() -> Route + Send,
{
    fn manual_clone(&self) -> InnerRouteFactory<'static> {
        Box::new(self.clone())
    }
}

pub type InnerRouteFactory<'a> = Box<dyn RouteFactory<'a, Output = Route>>;

fn build_inner_route<F, Args>(method: Method, handler: F) -> InnerRouteFactory<'static>
where
    F: Handler<Args> + 'static + Copy + Send,
    Args: FromRequest + 'static,
    F::Output: Responder + 'static,
{
    Box::new(move || actix_web::web::route().method(method.clone()).to(handler))
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
            let inner_route = build_inner_route(Method::$variant, handler);
            match self.router.0.entry(path.to_string()) {
                Entry::Occupied(mut e) => {
                    e.get_mut().push(inner_route);
                }
                Entry::Vacant(e) => {
                    e.insert(vec![inner_route]);
                }
            }
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
        for (path, routes) in self.router.0 {
            let mut resource = Resource::new(path);
            for route in routes {
                resource = resource.route(route());
            }
            scope = scope.service(resource);
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
