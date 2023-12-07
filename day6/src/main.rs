use std::collections::HashSet;

use once_cell::sync::Lazy;
use processor::{process, read_next, read_word};

type AError = anyhow::Error;

enum LoadingState {
    Times,
    Distances,
    Done,
}

#[derive(Debug)]
struct RaceStats {
    time: u64,
    record_distance: u64,
}

type InitialState = (LoadingState, (Vec<u64>, Vec<u64>));
type LoadedState1 = Vec<RaceStats>;
type ProcessedState1 = Vec<u64>;
type LoadedState2 = RaceStats;
type ProcessedState2 = u64;
type FinalResult = u64;

fn main() {
    let file = "test-input.txt";
    //let file = "test-input2.txt";
    //let file = "input.txt";

    let result1 = process(
        file,
        (LoadingState::Times, (Vec::new(), Vec::new())),
        parse_line,
        finalise_state_1,
        perform_processing_1,
        calc_result_1,
    );
    match result1 {
        Ok(res) => println!("Result 1: {:?}", res),
        Err(e) => println!("Error on 1: {}", e),
    }

    let result2 = process(
        file,
        (LoadingState::Times, (Vec::new(), Vec::new())),
        parse_line,
        finalise_state_2,
        perform_processing_2,
        calc_result_2,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([':', ' ']));

fn parse_line(istate: InitialState, line: String) -> Result<InitialState, AError> {
    let (loading_state, (mut times, mut distances)) = istate;

    fn load_values(line: String, storage: &mut Vec<u64>) {
        let mut chars = line.chars();
        if let Some(_name) = read_word(&mut chars, &DELIMITERS) {
            let mut keep_going = true;
            while keep_going {
                match read_next::<u64>(&mut chars, &DELIMITERS) {
                    Ok((value, _)) => storage.push(value),
                    Err(_) => keep_going = false,
                }
            }
        }
    }

    let next_state = match loading_state {
        LoadingState::Times => {
            load_values(line, &mut times);
            LoadingState::Distances
        }
        LoadingState::Distances => {
            load_values(line, &mut distances);
            LoadingState::Done
        }
        LoadingState::Done => return Err(AError::msg("Unexpectedly reached Done while loading")),
    };
    Ok((next_state, (times, distances)))
}

fn finalise_state_1(istate: InitialState) -> Result<LoadedState1, AError> {
    let (_, (times, distances)) = istate;
    let stats = times
        .iter()
        .zip(distances.iter())
        .fold(Vec::new(), |mut acc, (time, distance)| {
            let race_stats = RaceStats {
                time: *time,
                record_distance: *distance,
            };
            acc.push(race_stats);
            acc
        });
    //println!("{stats:?}");
    Ok(stats)
}

fn caclulate_distance_for_hold_time(hold_time: &u64, race_stats: &RaceStats) -> u64 {
    let speed = hold_time;
    (race_stats.time - hold_time) * speed
}

fn find_winning_combinations(race_stats: &RaceStats) -> u64 {
    let first_winning_time = (1..(race_stats.time - 1))
        .find(|hold_time| {
            let distance = caclulate_distance_for_hold_time(hold_time, race_stats);
            distance > race_stats.record_distance
        })
        .expect("Failed to find the first winning time");
    let last_winning_time = (1..(race_stats.time - 1))
        .rev()
        .find(|hold_time| {
            let distance = caclulate_distance_for_hold_time(hold_time, race_stats);
            distance > race_stats.record_distance
        })
        .expect("Failed to find the last winning time");
    last_winning_time - first_winning_time + 1
}

fn find_winning_combinations_quadratic(race_stats: &RaceStats) -> u64 {
    let a = -1f64;
    let b = race_stats.time as f64;
    let c = -(race_stats.record_distance as f64);

    let b_squared = b.powf(2f64);
    let _4_a_c = 4f64 * a * c;
    let b_squared_minus_4ac = b_squared - _4_a_c;
    let sqrt_b_squared_minus_4ac = b_squared_minus_4ac.sqrt();
    let _2_a = 2f64 * a;

    let positive_root = (-b + sqrt_b_squared_minus_4ac) / _2_a;
    let negative_root = (-b - sqrt_b_squared_minus_4ac) / _2_a;

    let (lower, upper) = if positive_root > negative_root {
        (negative_root, positive_root)
    } else {
        (positive_root, negative_root)
    };

    let low_time = {
        let lower = lower.floor() as u64;
        if caclulate_distance_for_hold_time(&lower, race_stats) > race_stats.record_distance {
            lower
        } else {
            lower + 1
        }
    };
    let high_time = {
        let upper = upper.ceil() as u64;
        if caclulate_distance_for_hold_time(&upper, race_stats) > race_stats.record_distance {
            upper
        } else {
            upper - 1
        }
    };
    high_time - low_time + 1
}

fn perform_processing_1(state: LoadedState1) -> Result<ProcessedState1, AError> {
    let numbers_of_winning_possibilities = state.iter().map(find_winning_combinations_quadratic).collect();
    Ok(numbers_of_winning_possibilities)
}

fn calc_result_1(state: ProcessedState1) -> Result<FinalResult, AError> {
    Ok(state.iter().product())
}

fn finalise_state_2(istate: InitialState) -> Result<LoadedState2, AError> {
    let (_, (times, distances)) = istate;
    let time_string = times
        .iter()
        .fold(string_builder::Builder::new(10), |mut acc, value| {
            acc.append(format!("{value}"));
            acc
        })
        .string()?;
    let distance_string = distances
        .iter()
        .fold(string_builder::Builder::new(10), |mut acc, value| {
            acc.append(format!("{value}"));
            acc
        })
        .string()?;
    let time = time_string.parse::<u64>()?;
    let distance = distance_string.parse::<u64>()?;
    Ok(RaceStats {
        time,
        record_distance: distance,
    })
}

fn perform_processing_2(state: LoadedState2) -> Result<ProcessedState2, AError> {
    let num = find_winning_combinations_quadratic(&state);
    Ok(num)
}

fn calc_result_2(state: ProcessedState2) -> Result<FinalResult, AError> {
    Ok(state)
}
