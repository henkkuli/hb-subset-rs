use thiserror::Error;

/// An error returned when an allocation fails.
#[derive(Debug, Error)]
#[error("Failed to allocate object")]
pub struct AllocationError;

/// An error returned when font face could not be subset.
#[derive(Debug, Error)]
#[error("Failed to subset font face")]
pub struct SubsettingError;

/// An error returned when a font face could not be extracted from blob.
#[derive(Debug, Error)]
#[error("Failed to extract font face from blob")]
pub struct FontFaceExtractionError;
