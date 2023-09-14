use crate::{impl_oa_schema_none, impl_oa_schema_passthrough};

impl_oa_schema_passthrough!(axum::Json<T>);

impl_oa_schema_none!(axum::extract::Extension<T>, T);
impl_oa_schema_none!(axum::extract::State<T>, T);
impl_oa_schema_none!(axum::extract::ConnectInfo<T>, T);
impl_oa_schema_none!(axum::http::Response<T>, T);
impl_oa_schema_none!(axum::http::Request<T>, T);
impl_oa_schema_none!(axum::http::HeaderMap<T>, T);

// TODO fill this out
impl_oa_schema_none!(axum::extract::Query<T>, T);

// TODO fill this out
impl_oa_schema_none!(axum::extract::Path<T>, T);

impl_oa_schema_none!(axum::http::request::Parts);
