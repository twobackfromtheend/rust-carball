use carball::analysis::CarballAnalyzer;
use carball::outputs::DataFramesOutput;
use carball::outputs::MetadataOutput;
use carball::CarballParser;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

pub fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("Parse");
    let file_path = PathBuf::from("assets\\replays\\ranked-3s.replay");
    // PathBuf::from("D:\\code_projects\\boxcars\\carball\\assets\\replays\\soccar-lan.replay");

    group.sample_size(20);
    group.bench_function("bench-parse", |b| {
        b.iter(|| {
            CarballParser::parse_file(black_box(file_path.clone()), false).expect("failed to parse")
        })
    });
}

pub fn bench_generate_metadata_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate Output");
    let file_path = PathBuf::from("assets\\replays\\ranked-3s.replay");

    let carball_parser = CarballParser::parse_file(file_path, false).expect("failed to parse");
    group.sample_size(100);
    group.bench_function("bench-generate-metadata", |b| {
        b.iter(|| {
            MetadataOutput::generate_from(
                black_box(&carball_parser.replay),
                black_box(&carball_parser.frame_parser),
            )
        })
    });
}

pub fn bench_generate_data_frame_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("Generate Output");
    let file_path = PathBuf::from("assets\\replays\\ranked-3s.replay");

    let carball_parser = CarballParser::parse_file(file_path, false).expect("failed to parse");
    group.sample_size(100);
    group.bench_function("bench-generate-data-frame", |b| {
        b.iter(|| DataFramesOutput::generate_from(black_box(&carball_parser.frame_parser)))
    });
}

pub fn bench_analyze(c: &mut Criterion) {
    let mut group = c.benchmark_group("Analyze");
    let file_path = PathBuf::from("assets\\replays\\ranked-3s.replay");
    let carball_parser = CarballParser::parse_file(file_path, false).expect("failed to parse");
    let metadata =
        MetadataOutput::generate_from(&carball_parser.replay, &carball_parser.frame_parser);
    group.sample_size(50);
    group.bench_function("bench-analyze", |b| {
        b.iter(|| CarballAnalyzer::analyze(black_box(&carball_parser), black_box(&metadata)))
    });
}

criterion_group!(
    benches,
    bench_parse,
    bench_generate_metadata_output,
    bench_generate_data_frame_output
);

criterion_main!(benches);
