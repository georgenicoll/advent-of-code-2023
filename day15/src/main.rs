use std::collections::HashSet;

use linked_hash_map::LinkedHashMap;
use once_cell::sync::Lazy;
use processor::{process, read_next, read_word};

type AError = anyhow::Error;

type InitialState = Vec<Vec<u8>>;
type LoadedState = InitialState;
type ProcessedState = Vec<usize>;
type FinalResult = usize;

const NUM_BOXES: usize = 256;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([',']));

fn parse_line_1(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        let mut chars = line.chars();
        //rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7
        while let Some((string, _)) = read_word(&mut chars, &DELIMITERS) {
            //Assumption - only ascii characters, nothing that needs more than 1 byte to encode
            let codes = string.as_bytes().to_vec();
            state.push(codes);
        }
    }
    Ok(state)
}

fn finalise_state_1(state: InitialState) -> Result<LoadedState, AError> {
    Ok(state)
}

fn calculate_hash(codes: &[u8]) -> usize {
    codes.iter().fold(0usize, |acc, code| {
        ((acc + *code as usize) * 17) % NUM_BOXES
    })
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let result = state.iter().map(|codes| calculate_hash(codes)).collect();
    Ok(result)
}

fn calc_result_1(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state.iter().sum())
}

#[derive(Debug)]
enum Operation {
    Remove,
    SetFocalLength { focal_length: usize },
}

#[derive(Debug)]
struct Step {
    label: String,
    hash: usize,
    operation: Operation,
}

type InitialState2 = Vec<Step>;
type LoadedState2 = InitialState2;
type ProcessedState2 = Vec<LinkedHashMap<String, usize>>; //boxes (label -> focal_length [in insertion order])
type FinalResult2 = usize;

static STEP_DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from(['=', '-']));

fn parse_line_2(mut state: InitialState2, line: String) -> Result<InitialState2, AError> {
    if !line.is_empty() {
        let mut chars = line.chars();
        //rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7
        while let Some((string, _)) = read_word(&mut chars, &DELIMITERS) {
            let mut step_chars = string.chars();
            let (label, step_delimiter) =
                read_word(&mut step_chars, &STEP_DELIMITERS).expect("Failed to read label");
            let hash = calculate_hash(label.as_bytes());
            let operation = match step_delimiter {
                Some('-') => Operation::Remove,
                Some('=') => {
                    if let Ok((focal_length, _)) = read_next(&mut step_chars, &DELIMITERS) {
                        Operation::SetFocalLength { focal_length }
                    } else {
                        panic!("Failed to read focal length in {string}")
                    }
                }
                _ => panic!("No or unrecognised step delimiter in {string}"),
            };
            state.push(Step {
                label,
                hash,
                operation,
            });
        }
    }
    Ok(state)
}

fn finalise_state_2(state: InitialState2) -> Result<LoadedState2, AError> {
    Ok(state)
}

fn perform_processing_2(state: LoadedState2) -> Result<ProcessedState2, AError> {
    let mut boxes = Vec::with_capacity(NUM_BOXES);
    while boxes.len() < NUM_BOXES {
        boxes.push(LinkedHashMap::default())
    }
    for step in state {
        // println!("Step: {:?}", step);
        let the_box = boxes.get_mut(step.hash).unwrap();
        match step.operation {
            Operation::Remove => {
                the_box.remove(&step.label);
            }
            Operation::SetFocalLength { focal_length } => {
                let entry = the_box.entry(step.label).or_default();
                *entry = focal_length;
            }
        };
        output_boxes(&boxes);
    }
    Ok(boxes)
}

fn output_boxes(_boxes: &[LinkedHashMap<String, usize>]) {
    // boxes.iter().enumerate().for_each(|(index, the_box)| {
    //     if !the_box.is_empty() {
    //         println!("Box {}: {:?}", index, the_box);
    //     }
    // });
    // println!();
}

fn calc_result_2(state: ProcessedState2) -> Result<FinalResult2, AError> {
    output_boxes(&state);
    let result = state
        .iter()
        .enumerate()
        .map(|(box_index, the_box)| {
            let box_number_multiplier = box_index + 1;
            let total_box_lenses_focusing_power: usize = the_box
                .iter()
                .enumerate()
                .map(|(lens_index, (_label, focal_length))| {
                    let lens_slot_number = lens_index + 1;
                    //lens_focal_power
                    box_number_multiplier * lens_slot_number * focal_length
                })
                .sum();
            total_box_lenses_focusing_power
        })
        .sum();
    Ok(result)
}

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        Vec::new(),
        parse_line_1,
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
        Vec::new(),
        parse_line_2,
        finalise_state_2,
        perform_processing_2,
        calc_result_2,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}
