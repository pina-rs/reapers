//! Enums that are saved and sent by name.

/// Gives a fieldless enum the list of its values, its saved name, a lookup by
/// that name, its index, and serde as that name: the form saves and the wire
/// have always used.
macro_rules! named_enum {
    ($ty:ident { $($variant:ident => $name:literal),+ $(,)? }) => {
        impl $ty {
            /// Every value, in declaration order.
            pub const ALL: [Self; [$($name),+].len()] = [$(Self::$variant),+];

            /// Its name, as saves and the wire write it.
            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }

            /// The value called `name`, if there is one.
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    $($name => Some(Self::$variant),)+
                    _ => None,
                }
            }

            /// Its position in [`Self::ALL`].
            pub const fn index(self) -> usize {
                self as usize
            }
        }

        impl ::core::fmt::Display for $ty {
            fn fmt(&self, formatter: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                formatter.write_str(self.name())
            }
        }

        impl ::serde::Serialize for $ty {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.name())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $ty {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let name = <::std::string::String as ::serde::Deserialize>::deserialize(deserializer)?;

                Self::from_name(&name)
                    .ok_or_else(|| ::serde::de::Error::unknown_variant(&name, &[$($name),+]))
            }
        }
    };
}

pub(crate) use named_enum;
