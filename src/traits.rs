use crate::error::DemesError;

pub(crate) trait Validate {
    fn validate<F: FnOnce(String) -> DemesError>(&self, err: F) -> Result<(), DemesError>;
}
