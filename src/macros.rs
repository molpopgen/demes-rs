#![macro_use]

macro_rules! impl_newtype_traits {
    ($type: ty) => {
        impl From<$type> for f64 {
            fn from(value: $type) -> f64 {
                value.0
            }
        }

        impl PartialOrd<$type> for $type {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(&other))
            }
        }

        impl Ord for $type {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                match self.0.partial_cmp(&other.0) {
                    Some(ordering) => ordering,
                    None => panic!("invalid f64 snuck in somehow"),
                }
            }
        }

        impl PartialEq for $type {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }

        impl Eq for $type {}

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}({})", stringify!($type), self.0)
            }
        }
    };
}
