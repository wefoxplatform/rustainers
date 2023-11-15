/// An image name errors
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ImageNameError {
    /// Empty name
    #[error("Image name cannot be empty")]
    EmptyName,
}
