#![macro_use]

macro_rules! impl_f64_newtypes {
    ($type: ty, $error: ident) => {
        impl std::convert::TryFrom<f64> for $type {
            type Error = $crate::DemesError;

            fn try_from(value: f64) -> Result<Self, Self::Error> {
                if value.is_nan() || value < 0.0 {
                    Err(Self::Error::$error(value))
                } else {
                    Ok(Self(value))
                }
            }
        }

        impl From<$type> for f64 {
            fn from(value: $type) -> Self {
                value.0
            }
        }

        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = f64::deserialize(deserializer)?;

                match <$type>::try_from(value) {
                    Ok(rv) => Ok(rv),
                    Err(_) => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Float(value),
                        &"value must be non-negative, non-NaN",
                    )),
                }
            }
        }
    };
}
