use crate::outputs::{OutputError, ParseOutput};
use clap::arg_enum;
use log::info;
use polars::prelude::DataFrame;
use polars::prelude::{CsvWriter, ParquetWriter, SerWriter};
use std::fmt::Debug;
use std::fs::File;
use std::path::Path;

arg_enum! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum DataFrameOutputFormat {
        Csv,
        Parquet,
    }
}

pub fn write_outputs(
    parse_output: &ParseOutput,
    data_frame_output_format: DataFrameOutputFormat,
) -> Result<(), OutputError> {
    serde_json::to_writer_pretty(
        &File::create("outputs/metadata.json").map_err(OutputError::CreateMetadataFileError)?,
        &parse_output.metadata,
    )
    .expect("Failed to write JSON.");
    write_df(
        format!("outputs/{}", "__ball"),
        &parse_output.data_frames.ball,
        data_frame_output_format,
    )?;
    write_df(
        format!("outputs/{}", "__game"),
        &parse_output.data_frames.game,
        data_frame_output_format,
    )?;
    for (actor_id, player_df) in parse_output.data_frames.players.iter() {
        write_df(
            format!("outputs/{}", actor_id),
            &player_df,
            data_frame_output_format,
        )?;
    }
    Ok(())
}

pub fn write_df(
    name: String,
    df: &DataFrame,
    data_frame_output_format: DataFrameOutputFormat,
) -> Result<(), OutputError> {
    match data_frame_output_format {
        DataFrameOutputFormat::Csv => write_df_to_csv(name + ".csv", df),
        DataFrameOutputFormat::Parquet => write_df_to_parquet(name + ".parquet", df),
    }
}

pub fn write_df_to_csv<P: AsRef<Path> + Debug>(path: P, df: &DataFrame) -> Result<(), OutputError> {
    let mut csv_file = File::create(&path).expect("Could not create CSV file.");
    CsvWriter::new(&mut csv_file)
        .has_headers(true)
        .with_delimiter(b',')
        .finish(df)
        .map_err(OutputError::WriteDataFrameError)?;
    info!("Wrote df to csv at {:?}", &path);
    Ok(())
}

pub fn write_df_to_parquet<P: AsRef<Path> + Debug>(
    path: P,
    df: &DataFrame,
) -> Result<(), OutputError> {
    let file = File::create(&path).expect("Could not create parquet file");
    ParquetWriter::new(file)
        .finish(df)
        .map_err(OutputError::WriteDataFrameError)?;
    info!("Wrote df to csv at {:?}", &path);
    Ok(())
}
