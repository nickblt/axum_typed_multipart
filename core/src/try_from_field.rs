use crate::field_data::FieldData;
use crate::field_metadata::FieldMetadata;
use crate::typed_multipart_error::TypedMultipartError;
use axum::async_trait;
use axum::extract::multipart::Field;
use std::any::type_name;

/// Types that can be created from an instance of [Field].
///
/// All fields for a given struct must implement this trait to be able to derive
/// the [TryFromMultipart](crate::try_from_multipart::TryFromMultipart) trait.
///
/// ## Example
///
/// ```rust
/// use axum::async_trait;
/// use axum::extract::multipart::Field;
/// use axum_typed_multipart::{TryFromField, TypedMultipartError};
///
/// struct Foo(String);
///
/// #[async_trait]
/// impl TryFromField for Foo {
///     async fn try_from_field(field: Field<'_>) -> Result<Self, TypedMultipartError> {
///         let text = field.text().await?;
///         Ok(Foo(text))
///     }
/// }
/// ```
#[async_trait]
pub trait TryFromField: Sized {
    /// Consume the input [Field] to create the supplied type.
    async fn try_from_field(field: Field<'_>) -> Result<Self, TypedMultipartError>;
}

/// Generate a [TryFromField] implementation for the supplied type using the
/// `str::parse` method on the text representation of the field data.
macro_rules! gen_try_from_field_impl {
    ( $type: ty ) => {
        #[async_trait]
        impl TryFromField for $type {
            async fn try_from_field(field: Field<'_>) -> Result<Self, TypedMultipartError> {
                let field_name = field.name().unwrap().to_string();
                let text = field.text().await?;

                str::parse(&text).map_err(move |_| TypedMultipartError::WrongFieldType {
                    field_name,
                    field_type: type_name::<$type>().to_string(),
                })
            }
        }
    };
}

gen_try_from_field_impl!(i8);
gen_try_from_field_impl!(i16);
gen_try_from_field_impl!(i32);
gen_try_from_field_impl!(i64);
gen_try_from_field_impl!(i128);
gen_try_from_field_impl!(isize);
gen_try_from_field_impl!(u8);
gen_try_from_field_impl!(u16);
gen_try_from_field_impl!(u32);
gen_try_from_field_impl!(u64);
gen_try_from_field_impl!(u128);
gen_try_from_field_impl!(usize);
gen_try_from_field_impl!(f32);
gen_try_from_field_impl!(f64);
gen_try_from_field_impl!(bool); // TODO?: Consider accepting any thruthy value.
gen_try_from_field_impl!(char);

#[async_trait]
impl TryFromField for String {
    async fn try_from_field(field: Field<'_>) -> Result<Self, TypedMultipartError> {
        let text = field.text().await?;
        Ok(text)
    }
}

#[async_trait]
impl TryFromField for Vec<u8> {
    async fn try_from_field(field: Field<'_>) -> Result<Self, TypedMultipartError> {
        let bytes = field.bytes().await?.to_vec();
        Ok(bytes)
    }
}

#[async_trait]
impl<T: TryFromField> TryFromField for FieldData<T> {
    async fn try_from_field(field: Field<'_>) -> Result<Self, TypedMultipartError> {
        let metadata = FieldMetadata::from(&field);
        let contents = T::try_from_field(field).await?;
        Ok(Self { metadata, contents })
    }
}
