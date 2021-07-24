use carball::frame_parser::FrameParser;
use carball::read_file;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;

fn bench_full(path: PathBuf) {
    let replay = read_file(path).expect("failed to parse");
    let frame_parser = FrameParser::new(replay);
    frame_parser.process_replay();
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("full-parse");
    let file_path = PathBuf::from("assets\\replays\\ranked-3s.replay");
    // PathBuf::from("D:\\code_projects\\boxcars\\carball\\assets\\replays\\soccar-lan.replay");

    group.sample_size(10);
    group.bench_function("bench-full-parse", |b| {
        // b.iter(|| black_box(bench_full(file_path.clone())))
        b.iter(|| bench_full(black_box(file_path.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
