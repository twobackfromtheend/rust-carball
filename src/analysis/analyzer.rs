use crate::analysis::{Hit, HitDetectionError};
use crate::outputs::MetadataOutput;
use crate::CarballParser;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct CarballAnalyzer {
    pub hits: Vec<Hit>,
}

impl CarballAnalyzer {
    pub fn analyze(
        carball_parser: &CarballParser,
        metadata: &MetadataOutput,
    ) -> Result<Self, CarballAnalyzerError> {
        let hits = Hit::find_hits(&carball_parser.frame_parser, metadata)
            .map_err(CarballAnalyzerError::HitDetectionError)?;
        // let hit_frame_numbers: Vec<usize> = hits.iter().map(|hit| hit.frame_number).collect();
        // println!("{:?}", hit_frame_numbers);
        Ok(Self { hits })
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum CarballAnalyzerError {
    #[error("error calculating hits: {0}")]
    HitDetectionError(HitDetectionError),
}
