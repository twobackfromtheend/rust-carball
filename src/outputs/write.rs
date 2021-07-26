use crate::outputs::{DataFramesOutput, MetadataOutput};
use clap::arg_enum;
use log::info;
use polars::error::PolarsError;
use polars::prelude::DataFrame;
use polars::prelude::{CsvWriter, ParquetWriter, SerWriter};
use std::fmt::Debug;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use thiserror::Error;

arg_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DataFrameOutputFormat {
        Csv,
        Parquet,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutputWriter {
    output_dir: PathBuf,
    data_frame_output_format: Option<DataFrameOutputFormat>,
}

impl ParseOutputWriter {
    pub fn new(
        output_dir: PathBuf,
        data_frame_output_format: Option<DataFrameOutputFormat>,
    ) -> Self {
        Self {
            output_dir,
            data_frame_output_format,
        }
    }

    pub fn write_outputs(
        &self,
        metadata_output: Option<&MetadataOutput>,
        data_frames_output: Option<&DataFramesOutput>,
    ) -> Result<(), ParseOutputWriterError> {
        if let Some(_metadata_output) = metadata_output {
            let mut metadata_output_path = self.output_dir.clone();
            metadata_output_path.push("metadata.json");
            serde_json::to_writer_pretty(
                &File::create(metadata_output_path)
                    .map_err(ParseOutputWriterError::CreateMetadataFileError)?,
                &_metadata_output,
            )
            .map_err(ParseOutputWriterError::WriteMetadataJsonError)?;
        }
        if let Some(_data_frames_output) = data_frames_output {
            let data_frame_output_format = self
                .data_frame_output_format
                .ok_or(ParseOutputWriterError::DataFrameFormatNotSet)?;

            let mut ball_data_frame_path = self.output_dir.clone();
            ball_data_frame_path.push("__ball");
            write_df(
                ball_data_frame_path,
                &_data_frames_output.ball,
                data_frame_output_format,
            )?;

            let mut game_data_frame_path = self.output_dir.clone();
            game_data_frame_path.push("__game");
            write_df(
                game_data_frame_path,
                &_data_frames_output.game,
                data_frame_output_format,
            )?;

            for (actor_id, player_df) in _data_frames_output.players.iter() {
                let mut player_data_frame_path = self.output_dir.clone();
                player_data_frame_path.push(format!("player_{}", actor_id));
                write_df(player_data_frame_path, &player_df, data_frame_output_format)?;
            }
        }
        Ok(())
    }
}

pub fn write_df(
    path: PathBuf,
    df: &DataFrame,
    data_frame_output_format: DataFrameOutputFormat,
) -> Result<(), ParseOutputWriterError> {
    match data_frame_output_format {
        DataFrameOutputFormat::Csv => write_df_to_csv(path.with_extension("csv"), df),
        DataFrameOutputFormat::Parquet => write_df_to_parquet(path.with_extension("parquet"), df),
    }
}

pub fn write_df_to_csv<P: AsRef<Path> + Debug>(
    path: P,
    df: &DataFrame,
) -> Result<(), ParseOutputWriterError> {
    let mut csv_file = File::create(&path).expect("Could not create CSV file.");
    CsvWriter::new(&mut csv_file)
        .has_headers(true)
        .with_delimiter(b',')
        .finish(df)
        .map_err(ParseOutputWriterError::WriteDataFrameError)?;
    info!("Wrote df to csv at {:?}", &path);
    Ok(())
}

pub fn write_df_to_parquet<P: AsRef<Path> + Debug>(
    path: P,
    df: &DataFrame,
) -> Result<(), ParseOutputWriterError> {
    let file = File::create(&path).expect("Could not create parquet file");
    ParquetWriter::new(file)
        .finish(df)
        .map_err(ParseOutputWriterError::WriteDataFrameError)?;
    info!("Wrote df to csv at {:?}", &path);
    Ok(())
}

#[derive(Debug, Error)]
pub enum ParseOutputWriterError {
    #[error("Failed to create metadata file: {0}")]
    CreateMetadataFileError(std::io::Error),
    #[error("Failed to write metadata file to JSON: {0}")]
    WriteMetadataJsonError(serde_json::Error),
    #[error("Failed to write DataFrame: {0}")]
    WriteDataFrameError(PolarsError),
    #[error("DataFrame output format not set")]
    DataFrameFormatNotSet,
}
