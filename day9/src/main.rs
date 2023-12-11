use std::collections::HashSet;

use once_cell::sync::Lazy;
use processor::{process, read_next};

type AError = anyhow::Error;
type InitialState = Vec<Vec<i64>>;
type LoadedState = InitialState;
type ProcessedState = Vec<i64>;
type FinalResult = i64;

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        Vec::new(),
        parse_line,
        finalise_state,
        perform_processing_1,
        calc_result,
    );
    match result1 {
        Ok(res) => println!("Result 1: {:?}", res),
        Err(e) => println!("Error on 1: {}", e),
    }

    let result2 = process(
        file,
        Vec::new(),
        parse_line,
        finalise_state,
        perform_processing_2,
        calc_result,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ']));

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    let mut chars = line.chars();
    let mut nums: Vec<i64> = Vec::default();
    while let Ok((num, _)) = read_next::<i64>(&mut chars, &DELIMITERS) {
        nums.push(num);
    }
    state.push(nums);
    Ok(state)
}

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
    //println!("{state:?}");
    Ok(state)
}

fn calculate_seq_number<F1, F2>(
    nums: &Vec<i64>,
    get_num_in_sequence: F1,
    get_adjusted_number: &F2,
) -> i64
where
    F1: Fn(&Vec<i64>) -> i64,
    F2: Fn(i64, i64) -> i64,
{
    // println!("{nums:?}");
    let (all_zeros, diffs) = nums.windows(2).fold(
        (true, Vec::default()),
        |(all_zeros_so_far, mut diffs), ns| {
            let n1 = ns[0];
            let n2 = ns[1];
            let diff = n2 - n1;
            diffs.push(diff);
            (all_zeros_so_far && diff == 0, diffs)
        },
    );
    let seq_num = get_num_in_sequence(nums);
    if all_zeros {
        seq_num
    } else {
        get_adjusted_number(
            seq_num,
            calculate_seq_number(&diffs, get_num_in_sequence, get_adjusted_number),
        )
    }
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let next_nums = state
        .iter()
        .map(|nums| {
            calculate_seq_number(
                nums,
                |nums| *nums.last().unwrap(),
                &|num_in_seq, adjustment| num_in_seq + adjustment,
            )
        })
        .collect();
    Ok(next_nums)
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let next_nums = state
        .iter()
        .map(|nums| {
            calculate_seq_number(
                nums,
                |nums| *nums.first().unwrap(),
                &|num_in_seq, adjustment| num_in_seq - adjustment,
            )
        })
        .collect();
    Ok(next_nums)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    let result = state.iter().sum();
    Ok(result)
}
