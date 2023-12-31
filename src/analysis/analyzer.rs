use crate::analysis::GameplayPeriod;
use crate::analysis::{Hit, HitDetectionError, Stats, StatsGenerationError};
use crate::outputs::{DataFramesOutput, MetadataOutput};
use crate::CarballParser;
use serde::Serialize;
use std::fs::File;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct CarballAnalyzer {
    pub hits: Vec<Hit>,
    pub stats: Stats,
    pub gameplay_periods: Vec<GameplayPeriod>,
}

impl CarballAnalyzer {
    pub fn analyze(
        carball_parser: &CarballParser,
        metadata: &MetadataOutput,
        data_frames: &DataFramesOutput,
    ) -> Result<Self, CarballAnalyzerError> {
        let hits = Hit::find_hits(&carball_parser.frame_parser, metadata)
            .map_err(CarballAnalyzerError::HitDetectionError)?;

        let gameplay_periods = GameplayPeriod::get_periods(metadata, data_frames);

        let stats = Stats::generate_from(metadata, data_frames, &gameplay_periods)
            .map_err(CarballAnalyzerError::StatsGenerationError)?;
        Ok(Self {
            hits,
            stats,
            gameplay_periods,
        })
    }

    pub fn write(&self, output_dir: PathBuf) -> Result<(), CarballAnalyzerWriteError> {
        let mut output_path = output_dir;
        output_path.push("analysis.json"); // TODO: Make this configurable, along with the other output files like metadata.json.

        serde_json::to_writer_pretty(
            &File::create(output_path).map_err(CarballAnalyzerWriteError::CreateFileError)?,
            &self,
        )
        .map_err(CarballAnalyzerWriteError::WriteJsonError)
    }
}

#[derive(Error, Debug)]
pub enum CarballAnalyzerError {
    #[error("Failed to calculate hits: {0}")]
    HitDetectionError(HitDetectionError),
    #[error("Failed to generate stats: {0}")]
    StatsGenerationError(StatsGenerationError),
}

#[derive(Debug, Error)]
pub enum CarballAnalyzerWriteError {
    #[error("Failed to create file: {0}")]
    CreateFileError(std::io::Error),
    #[error("Failed to write file to JSON: {0}")]
    WriteJsonError(serde_json::Error),
}
