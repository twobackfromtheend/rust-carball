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
