#[macro_use]
extern crate log;

use carball::analysis::CarballAnalyzer;
use carball::outputs::RangeChecker;
use carball::outputs::{
    DataFrameOutputFormat, DataFramesOutput, MetadataOutput, ParseOutputWriter,
};
use carball::CarballParser;
use simplelog::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, parse(from_os_str))]
    input: PathBuf,
    #[structopt(short, parse(from_os_str))]
    output_dir: PathBuf,
    #[structopt(long)]
    skip_data_frames: bool,
    #[structopt(long)]
    skip_write_data_frames: bool,

    #[structopt(required_unless_one(&["skip_data_frames", "skip_write_data_frames"]), possible_values = &DataFrameOutputFormat::variants(), case_insensitive = true)]
    data_frame_output_format: Option<DataFrameOutputFormat>,

    #[structopt(long)]
    skip_checks: bool,

    #[structopt(long)]
    skip_analysis: bool,
}

fn main() {
    setup_logging();

    let opt = Opt::from_args();
    // dbg!(&opt);
    info!("{:?}", &opt);

    let carball_parser = CarballParser::parse_file(opt.input, true).expect("Failed to parse.");

    let metadata =
        MetadataOutput::generate_from(&carball_parser.replay, &carball_parser.frame_parser);
    let data_frames = if opt.skip_data_frames {
        None
    } else {
        Some(
            DataFramesOutput::generate_from(&carball_parser.frame_parser)
                .expect("Failed to generate data frames."),
        )
    };

    if !opt.skip_data_frames && !opt.skip_checks {
        if let Some(_data_frames) = &data_frames {
            let range_checker = RangeChecker::new();
            range_checker
                .check_ranges(_data_frames)
                .expect("Failed to complete range checks.");
        }
    }

    let parse_output_writer =
        ParseOutputWriter::new(opt.output_dir.clone(), opt.data_frame_output_format);
    if opt.skip_write_data_frames {
        parse_output_writer
            .write_outputs(Some(&metadata), None)
            .expect("Failed to write outputs.");
    } else {
        parse_output_writer
            .write_outputs(Some(&metadata), data_frames.as_ref())
            .expect("Failed to write outputs.");
    }

    if !opt.skip_data_frames && !opt.skip_analysis {
        let analyzer = CarballAnalyzer::analyze(&carball_parser, &metadata, &data_frames.unwrap())
            .expect("Failed to analyze.");
        analyzer
            .write(opt.output_dir)
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
