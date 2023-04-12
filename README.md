# Carball

Rocket League replay analysis tool.

## Features

- Highly performant
- Well-typed

## Procedure

### Part 1: Raw data extraction

1. Binary replay file is parsed with `boxcars`, yielding well-typed objects to be processed.
2. Network data is parsed to extract useful information (e.g. ball, car, and boost etc.)
3. This data is processed to fix unexpected aspects from parsing the raw replay data. (Such as boost amounts not decreasing, and boost pickups not being accurate).

### Stat generation

## TODO

- Stats
  - Hit
    - Shot detection
    - Passes, dribbles, goals, etc.
  - Possession
    - Time-based
    - ~~Distance-based~~
  -

### Development notes

It tends to be faster to compile for release and parse as opposed to compiling for debug and parsing (as the duration increase for parsing in debug is more than the decrease in compile time).
`cargo run --release -- -i "assets\replays\ranked-3s.replay" -o "outputs" csv`

To read logs when running tests:
`cargo test -- --nocapture`

#### Building without RLUtilities (required to build on Linux or macOS)

If you want to build without RLUtilities, perhaps because you're running `rust-carball` on Linux or macOS, run `cargo build` with the `--no-default-features`. Unfortunately this will come at the cost of disabling ball prediction analysis.

This is necessary for Linux and macOS because the `ball_prediction` feature (enabled by default) makes use of `[RLUtilities](https://github.com/samuelpmish/RLUtilities)` via the `[chip](https://github.com/whatisaphone/chip)` crate. Unfortunately `chip` doesn't build `RLUtilities` directly, and the version of `RLUtilities` that `chip` depends on will only build on Windows.
