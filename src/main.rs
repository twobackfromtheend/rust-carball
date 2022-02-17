#[macro_use]
extern crate log;

use carball::analysis::CarballAnalyzer;
use carball::outputs::RangeChecker;
use carball::outputs::{
    DataFrameOutputFormat, DataFramesOutput, MetadataOutput, ParseOutputWriter,
};
use carball::CarballParser;
use clap::Parser;
use simplelog::*;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, parse(from_os_str))]
    input: PathBuf,
    #[clap(short, parse(from_os_str))]
    output_dir: PathBuf,
    #[clap(long)]
    skip_data_frames: bool,
    #[clap(long)]
    skip_write_data_frames: bool,

    #[clap(arg_enum, required_unless_present_any(&["skip_data_frames", "skip_write_data_frames"]), ignore_case = true)]
    data_frame_output_format: Option<DataFrameOutputFormat>,

    #[clap(long)]
    skip_checks: bool,

    #[clap(long)]
    skip_analysis: bool,
}

fn main() {
    setup_logging();

    let args = Args::parse();
    // dbg!(&args);
    info!("{:?}", &args);

    let carball_parser = CarballParser::parse_file(args.input, true).expect("Failed to parse.");

    let metadata =
        MetadataOutput::generate_from(&carball_parser.replay, &carball_parser.frame_parser);
    let data_frames = if args.skip_data_frames {
        None
    } else {
        Some(
            DataFramesOutput::generate_from(&carball_parser.frame_parser)
                .expect("Failed to generate data frames."),
        )
    };

    if !args.skip_data_frames && !args.skip_checks {
        if let Some(_data_frames) = &data_frames {
            let range_checker = RangeChecker::new();
            range_checker
                .check_ranges(_data_frames)
                .expect("Failed to complete range checks.");
        }
    }

    let parse_output_writer =
        ParseOutputWriter::new(args.output_dir.clone(), args.data_frame_output_format);
    if args.skip_write_data_frames {
        parse_output_writer
            .write_outputs(Some(&metadata), None)
            .expect("Failed to write outputs.");
    } else {
        parse_output_writer
            .write_outputs(Some(&metadata), data_frames.as_ref())
            .expect("Failed to write outputs.");
    }

    if !args.skip_data_frames && !args.skip_analysis {
        let analyzer = CarballAnalyzer::analyze(&carball_parser, &metadata, &data_frames.unwrap())
            .expect("Failed to analyze.");
        analyzer
            .write(args.output_dir)
            .expect("Failed to write analysis.");
    }

    info!("fin");
}

fn setup_logging() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    // println!("Testing logging");
    // debug!("debug");
    // info!("info");
    // warn!("warn");
    // error!("error");
}
