use crate::{OaSchema, Schema};

impl<T> OaSchema for sid::Sid<T> {
    type References = ();
    fn schema() -> Option<Schema> {
        Some(Schema::new_string())
    }
}
