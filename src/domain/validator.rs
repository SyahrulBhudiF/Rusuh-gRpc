use tonic::{Request, Status};
use validator::{Validate, ValidationErrors};

pub trait FromRequest<T>: Sized {
    fn from_request(request: Request<T>) -> Self;
}

pub trait ValidateFromRequest<T>: FromRequest<T> + Validate {
    fn validate_from_request(request: Request<T>) -> Result<Self, Status> {
        let dto = Self::from_request(request);
        dto.validate()
            .map_err(|e| Status::invalid_argument(format_validation_errors(e)))?;
        Ok(dto)
    }
}

impl<T, U> FromRequest<T> for U
where
    U: From<T>,
{
    fn from_request(request: Request<T>) -> Self {
        U::from(request.into_inner())
    }
}

pub fn format_validation_errors(errors: ValidationErrors) -> String {
    errors
        .field_errors()
        .iter()
        .map(|(field, errs)| {
            let messages: Vec<String> = errs
                .iter()
                .map(|e| {
                    if let Some(message) = &e.message {
                        message.to_string()
                    } else {
                        format!("{} is invalid", field)
                    }
                })
                .collect();
            format!("{}: {}", field, messages.join(", "))
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[macro_export]
macro_rules! impl_from_request {
    ($dto:ty, $req:ty, { $($field:ident),* $(,)? }) => {
        impl From<$req> for $dto {
            fn from(req: $req) -> Self {
                Self {
                    $(
                        $field: req.$field,
                    )*
                }
            }
        }

        impl ValidateFromRequest<$req> for $dto {}
    };
}
