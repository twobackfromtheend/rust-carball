pub mod actor_handlers;
pub mod analysis;
pub mod cleaner;
pub mod frame_parser;
pub mod outputs;

use crate::frame_parser::{FrameParser, FrameParserError};
use boxcars::{CrcCheck, NetworkParse, ParseError, ParserBuilder, Replay};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum CarballError {
    ReadFileError(io::Error),
    BoxCarsParseError(ParseError),
    FrameParserError(FrameParserError),
}

impl From<ParseError> for CarballError {
    fn from(err: ParseError) -> CarballError {
        CarballError::BoxCarsParseError(err)
    }
}
impl From<FrameParserError> for CarballError {
    fn from(err: FrameParserError) -> CarballError {
        CarballError::FrameParserError(err)
    }
}

#[derive(Debug, Clone)]
pub struct CarballParser {
    pub file_path: PathBuf,
    pub replay: Replay,
    pub frame_parser: FrameParser,
}

impl CarballParser {
    pub fn parse_file(file_path: PathBuf, show_progress: bool) -> Result<Self, CarballError> {
        let replay = read_file(&file_path)?;

        let frame_parser = FrameParser::from_replay(&replay, show_progress)?;

        Ok(Self {
            file_path,
            replay,
            frame_parser,
        })
    }

    pub fn write_outputs(self) {}
}

pub fn read_file(file_path: &Path) -> Result<Replay, CarballError> {
    let data = fs::read(file_path).map_err(CarballError::ReadFileError)?;
    Ok(ParserBuilder::new(&data[..])
        .with_crc_check(CrcCheck::Always)
        .with_network_parse(NetworkParse::Always)
        .parse()?)
}
