use std::collections::{HashMap, HashSet};

use once_cell::sync::Lazy;
use processor::{ok_identity, process, read_next, read_word};

type AError = anyhow::Error;
type InitialState = Vec<Game>;
type LoadedState = InitialState;
type ProcessedState = i64;

#[derive(Debug)]
struct Game {
    number: i64,
    picks: Vec<HashMap<String, i64>>,
}

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        Vec::new(),
        parse_line,
        ok_identity,
        perform_processing_1,
        ok_identity,
    );
    match result1 {
        Ok(res) => println!("Result 1: {:?}", res),
        Err(e) => println!("Error on 1: {}", e),
    }

    let result2 = process(
        file,
        Vec::new(),
        parse_line,
        ok_identity,
        perform_processing_2,
        ok_identity,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ', ':', ',', ';']));

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    let mut chars = line.chars();
    //Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
    if let Some(_game) = read_word(&mut chars, &DELIMITERS) {
        let (number, _delimiter) = read_next::<i64>(&mut chars, &DELIMITERS)?;
        let mut picks = Vec::new();
        let mut cubes: HashMap<String, i64> = HashMap::new();

        let mut num_cubes_and_delimiter = read_next::<i64>(&mut chars, &DELIMITERS);
        while num_cubes_and_delimiter.is_ok() {
            let (num_cubes, _) = num_cubes_and_delimiter.as_ref().ok().unwrap();
            let (colour, delimiter) = read_word(&mut chars, &DELIMITERS).ok_or_else(|| {
                AError::msg(format!(
                    "Expected a colour after {} in '{}'",
                    num_cubes, line
                ))
            })?;
            cubes.insert(colour, num_cubes.clone());
            let end_of_pick = delimiter.map(|c| c == ';').unwrap_or(true);
            if end_of_pick {
                picks.push(cubes);
                cubes = HashMap::new();
            }
            num_cubes_and_delimiter = read_next::<i64>(&mut chars, &DELIMITERS);
        }

        state.push(Game { number, picks });
    }
    Ok(state)
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let max_cubes: HashMap<String, i64> = HashMap::from([
        ("red".into(), 12.into()),
        ("green".into(), 13.into()),
        ("blue".into(), 14.into()),
    ]);

    let possible_games = state
        .iter()
        .filter(|game| {
            game.picks.iter().all(|pick| {
                max_cubes.iter().all(|(max_colour, max_number)| {
                    pick.get(max_colour)
                        .map(|number| number <= max_number)
                        .unwrap_or(true)
                })
            })
        })
        .map(|game| {
            //println!("Possible: {:?}", game);
            game.number
        })
        .fold(0, |acc, number| acc + number);

    Ok(possible_games)
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let powers = state
        .iter()
        .map(|game| {
            let mut max_cubes: HashMap<String, i64> = HashMap::from([
                ("red".into(), 0.into()),
                ("green".into(), 0.into()),
                ("blue".into(), 0.into()),
            ]);
            game.picks.iter().for_each(|pick| {
                pick.iter().for_each(|(colour, number)| {
                    let current_max = max_cubes.get_mut(colour).unwrap();
                    if *current_max < *number {
                        *current_max = *number;
                    }
                })
            });
            let power = max_cubes
                .iter()
                .fold(1, |acc, (_colour, number)| acc * number);
            power
        })
        .collect::<Vec<i64>>();
    let result = powers.iter().fold(0, |acc, power| acc + power);
    Ok(result)
}
