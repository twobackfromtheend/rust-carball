use crate::analysis::Hit;
use crate::frame_parser::FrameParser;
use crate::outputs::ParseOutput;

pub struct CarballAnalysis {}

pub fn analyze(frame_parser: &FrameParser, parse_output: &ParseOutput) {
    let hits = Hit::find_hits(frame_parser, parse_output).unwrap();
    let hit_frame_numbers: Vec<usize> = hits.iter().map(|hit| hit.frame_number).collect();
    println!("{:?}", hit_frame_numbers);
}
