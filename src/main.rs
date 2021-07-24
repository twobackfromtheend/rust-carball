#[macro_use]
extern crate log;

use carball::analysis::analyze;
use carball::outputs::{write_outputs, DataFrameOutputFormat, ParseOutput};
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
    #[structopt(required_unless("skip_data_frames"), possible_values = &DataFrameOutputFormat::variants(), case_insensitive = true)]
    data_frame_output_format: Option<DataFrameOutputFormat>,

    #[structopt(long)]
    skip_analysis: bool,
}

fn main() {
    setup_logging();

    println!("Hello, world!");

    let opt = Opt::from_args();
    dbg!(&opt);

    // let file_path = PathBuf::from("assets\\replays\\ranked-3s.replay");
    // let file_path = PathBuf::from("assets\\replays\\201E87CC11EBE9966E057CBD63E6D63F.replay");
    // let file_path = PathBuf::from("assets\\replays\\soccar-lan.replay");
    let carball_parser = CarballParser::parse_file(opt.input).expect("failed to parse");
    // let replay = read_file(&opt.input).expect("failed to parse");

    // let frame_parser = FrameParser::from_replay(replay).expect("failed to process");
    // frame_parser.process_replay().expect("failed to process");

    let parse_output =
        ParseOutput::generate_from(&carball_parser.replay, &carball_parser.frame_parser)
            .expect("Failed to generate outputs.");
    write_outputs(&parse_output, DataFrameOutputFormat::Csv).expect("Failed to write outputs.");

    // analyze(&frame_parser, &parse_output);

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

    println!("Testing logging");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");
}
