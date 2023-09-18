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

        impl PartialOrd<$type> for f64 {
            fn partial_cmp(&self, other: &$type) -> Option<std::cmp::Ordering> {
                self.partial_cmp(&other.0)
            }
        }

        impl PartialOrd<f64> for $type {
            fn partial_cmp(&self, other: &f64) -> Option<std::cmp::Ordering> {
                self.0.partial_cmp(other)
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

        impl PartialEq<$type> for f64 {
            fn eq(&self, other: &$type) -> bool {
                self.eq(&other.0)
            }
        }

        impl PartialEq<f64> for $type {
            fn eq(&self, other: &f64) -> bool {
                self.0.eq(other)
            }
        }

        impl Eq for $type {}

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

macro_rules! impl_input_newtype_arithmetic {
    ($type: ty, $op: ident, $fn: ident) => {
        impl std::ops::$op for $type {
            type Output = Self;
            fn $fn(self, value: Self) -> Self::Output {
                self.0.$fn(value.0).into()
            }
        }

        impl std::ops::$op<f64> for $type {
            type Output = Self;
            fn $fn(self, value: f64) -> Self::Output {
                self.0.$fn(value).into()
            }
        }

        impl std::ops::$op<$type> for f64 {
            type Output = Self;
            fn $fn(self, value: $type) -> Self::Output {
                self.$fn(value.0).into()
            }
        }
    };
}

macro_rules! impl_input_newtype_traits {
    ($type: ty) => {
        impl From<f64> for $type {
            fn from(value: f64) -> Self {
                Self(value)
            }
        }

        impl From<$type> for f64 {
            fn from(value: $type) -> Self {
                value.0
            }
        }

        impl PartialEq<$type> for f64 {
            fn eq(&self, other: &$type) -> bool {
                self.eq(&other.0)
            }
        }

        impl PartialEq<f64> for $type {
            fn eq(&self, other: &f64) -> bool {
                self.0.eq(other)
            }
        }

        impl_input_newtype_arithmetic!($type, Add, add);
        impl_input_newtype_arithmetic!($type, Sub, sub);
        impl_input_newtype_arithmetic!($type, Mul, mul);
        impl_input_newtype_arithmetic!($type, Div, div);
    };
}
