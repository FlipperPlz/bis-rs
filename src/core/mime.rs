/// This macro generates a new type enum that is statically convertible to and from a provided base
/// type. It also generates a TryFrom<T> implementation where T is the provided base type, this can
/// be useful for checked conversion from the base type to the enum.
///
/// # Usage
///
/// magic_enum! {
///     base_type, enum_name, error_type, error_variant {
///         variant = value,
///         ...
///     }
/// }
#[macro_export]
macro_rules! magic_enum {
    ($typ: ty, $name: ident, $error: ty, $error_variant: ident {
        $($variant: ident = $value:expr),* $(,)?
    }) => {
        #[repr($typ)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant = $value),*
        }

        impl std::convert::TryFrom<$typ> for $name {
            type Error = $error;

            fn try_from(value: $typ) -> Result<Self, Self::Error> {
                match value {
                    $($value => Ok($name::$variant),)*
                    _ => Err(Self::Error::$error_variant(value)),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $( $name::$variant => write!(f, stringify!($variant)), )*
                }
            }
        }
    }
}


