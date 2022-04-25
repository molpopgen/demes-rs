#![macro_use]

macro_rules! impl_deserialize_for_try_from_f64 {
    ($type: ty) => {
        impl<'de> serde::Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = f64::deserialize(deserializer)?;

                match <$type>::try_from(value) {
                    Ok(x) => Ok(x),
                    Err(e) => Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Float(value),
                        &format!("{}", e).as_str(),
                    )),
                }
            }
        }
    };
}
