#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct CumulocityError {
    #[from]
    pub source: reqwest::Error,
}
