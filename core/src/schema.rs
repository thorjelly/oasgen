use std::collections::HashMap;

use indexmap::IndexMap;
use oa::AdditionalProperties;
use openapiv3 as oa;
use openapiv3::{ArrayType, ObjectType, ReferenceOr, Schema, SchemaData, SchemaKind, Type};

#[cfg(feature = "actix")]
mod actix;

#[cfg(feature = "axum")]
mod axum;

#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "cookies")]
mod cookies;
#[cfg(feature = "phonenumber")]
mod phonenumber;
#[cfg(feature = "sqlx")]
mod sqlx;
#[cfg(feature = "time")]
mod time;

mod http;
#[cfg(feature = "sid")]
mod sid;

pub trait HasReferences {
    fn all_references() -> IndexMap<&'static str, oa::Schema>;
}

impl HasReferences for () {
    fn all_references() -> IndexMap<&'static str, oa::Schema> {
        IndexMap::new()
    }
}

pub trait OaSchema {
    type References: HasReferences;

    fn schema_name() -> Option<&'static str> {
        None
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        None
    }

    fn schema() -> Option<Schema> {
        None
    }

    fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
        None
    }
}

#[macro_export]
macro_rules! impl_oa_schema {
    ($t:ty,$schema:expr) => {
        impl $crate::OaSchema for $t {
            type References = ();

            fn schema_ref() -> Option<$crate::ReferenceOr<$crate::Schema>> {
                Some($crate::ReferenceOr::Item($schema))
            }

            fn schema() -> Option<$crate::Schema> {
                Some($schema)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_oa_schema_passthrough {
    ($t:ty) => {
        impl<T> $crate::OaSchema for $t
        where
            T: $crate::OaSchema,
        {
            type References = T::References;

            fn schema_name() -> Option<&'static str> {
                T::schema_name()
            }

            fn schema_ref() -> Option<$crate::ReferenceOr<$crate::Schema>> {
                T::schema_ref()
            }

            fn schema() -> Option<$crate::Schema> {
                T::schema()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_oa_schema_query {
    ($t:ty) => {
        impl<T> $crate::OaSchema for $t
        where
            T: $crate::OaSchema,
        {
            type References = ();

            fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
                let schema = T::schema()?;
                Some(
                    schema
                        .properties()?
                        .into_iter()
                        .map(|(name, s)| {
                            ReferenceOr::Item(oa::Parameter::Query {
                                parameter_data: oa::ParameterData {
                                    name: name.clone(),
                                    description: None,
                                    required: schema.required(&name),
                                    deprecated: None,
                                    format: oa::ParameterSchemaOrContent::Schema(s.clone()),
                                    example: None,
                                    examples: Default::default(),
                                    explode: None,
                                    extensions: Default::default(),
                                },
                                allow_reserved: false,
                                style: oa::QueryStyle::Form,
                                allow_empty_value: None,
                            })
                        })
                        .collect::<Vec<_>>(),
                )
            }
        }
    };
}

#[macro_export]
macro_rules! impl_oa_schema_header {
    ($t:ty) => {
        impl<T> $crate::OaSchema for $t
        where
            T: $crate::OaSchema,
        {
            type References = ();

            fn parameters() -> Option<Vec<ReferenceOr<oa::Parameter>>> {
                let schema = T::schema()?;
                Some(
                    schema
                        .properties()?
                        .into_iter()
                        .map(|(name, s)| {
                            ReferenceOr::Item(oa::Parameter::Header {
                                parameter_data: oa::ParameterData {
                                    name: name.clone(),
                                    description: None,
                                    required: schema.required(&name),
                                    deprecated: None,
                                    format: oa::ParameterSchemaOrContent::Schema(s.clone()),
                                    example: None,
                                    examples: Default::default(),
                                    explode: None,
                                    extensions: Default::default(),
                                },
                                style: oa::HeaderStyle::Simple,
                            })
                        })
                        .collect::<Vec<_>>(),
                )
            }
        }
    };
}

#[macro_export]
macro_rules! impl_oa_schema_none {
    ($t:ty $(, $arg:ident)*) => {
        impl<$($arg),*> $crate::OaSchema for $t {
            type References = ();
        }
    };
}

impl_oa_schema_none!(());

impl_oa_schema!(bool, Schema::new_bool());

impl_oa_schema!(usize, Schema::new_integer());
impl_oa_schema!(u32, Schema::new_integer());
impl_oa_schema!(i32, Schema::new_integer());
impl_oa_schema!(u64, Schema::new_integer());
impl_oa_schema!(i64, Schema::new_integer());
impl_oa_schema!(u16, Schema::new_integer());
impl_oa_schema!(i16, Schema::new_integer());
impl_oa_schema!(f32, Schema::new_number());
impl_oa_schema!(f64, Schema::new_number());

impl_oa_schema!(String, Schema::new_string());

impl<T> OaSchema for Vec<T>
where
    T: OaSchema,
{
    type References = (T,);

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        Some(ReferenceOr::Item(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                items: T::schema_ref().map(|r| r.boxed()),
                ..ArrayType::default()
            })),
        }))
    }

    fn schema() -> Option<Schema> {
        if let Some(schema) = T::schema() {
            Some(Schema::new_array(schema))
        } else {
            Some(Schema {
                schema_data: SchemaData::default(),
                schema_kind: SchemaKind::Type(Type::Array(ArrayType {
                    items: None,
                    ..ArrayType::default()
                })),
            })
        }
    }
}

impl<T> OaSchema for Option<T>
where
    T: OaSchema,
{
    type References = T::References;

    fn schema_name() -> Option<&'static str> {
        T::schema_name()
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        T::schema_ref()
    }

    fn schema() -> Option<Schema> {
        T::schema().map(|mut schema| {
            schema.schema_data.nullable = true;
            schema
        })
    }
}

impl<T, E> OaSchema for Result<T, E>
where
    T: OaSchema,
{
    type References = T::References;

    fn schema_name() -> Option<&'static str> {
        T::schema_name()
    }

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        T::schema_ref()
    }

    fn schema() -> Option<Schema> {
        T::schema()
    }
}

impl<K, V> OaSchema for HashMap<K, V>
where
    V: OaSchema,
{
    type References = (V,);

    fn schema_ref() -> Option<ReferenceOr<Schema>> {
        Some(ReferenceOr::Item(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Object(ObjectType {
                additional_properties: Some(AdditionalProperties::Schema(Box::new(
                    V::schema_ref()?
                ))),
                ..ObjectType::default()
            })),
        }))
    }

    fn schema() -> Option<Schema> {
        Some(Schema {
            schema_data: SchemaData::default(),
            schema_kind: SchemaKind::Type(Type::Object(ObjectType {
                additional_properties: V::schema()
                    .map(|s| AdditionalProperties::Schema(Box::new(ReferenceOr::Item(s)))),
                ..ObjectType::default()
            })),
        })
    }
}

macro_rules! construct_has_references {
    ($($arg:ident),+) => {
        impl<$($arg),+> HasReferences for ($($arg,)+)
            where
                $($arg: OaSchema),+,
        {
            fn all_references() -> IndexMap<&'static str, oa::Schema> {
                let mut map = [$(($arg::schema_name(), $arg::schema())),+]
                    .into_iter()
                    .filter_map(|(n, s)| Some((n?, s?)))
                    .collect::<IndexMap<_, _>>();

                $(map.extend($arg::References::all_references());)+

                map
            }
        }
    }
}

construct_has_references!(A1);
construct_has_references!(A1, A2);
construct_has_references!(A1, A2, A3);
construct_has_references!(A1, A2, A3, A4);
construct_has_references!(A1, A2, A3, A4, A5);
construct_has_references!(A1, A2, A3, A4, A5, A6);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8, A9);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15);
construct_has_references!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);

#[cfg(feature = "uuid")]
impl_oa_schema!(uuid::Uuid, Schema::new_string().with_format("uuid"));

impl_oa_schema!(serde_json::Value, Schema::new_object());
