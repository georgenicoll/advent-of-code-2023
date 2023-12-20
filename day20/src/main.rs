use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use processor::{process, read_word};
use substring::Substring;

type AError = anyhow::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pulse {
    High,
    Low,
}

#[derive(Debug)]
enum Module {
    FlipFlop {
        on: bool,
        outputs: Vec<String>,
    }, //'%', ignores high, flips on low,
    Conjunction {
        received: HashMap<String, Pulse>,
        outputs: Vec<String>,
    }, //'&', starts low on all
    Broadcast {
        outputs: Vec<String>,
    }, //Single one 'broadcaster'
}

type InitialState = HashMap<String, Module>;

type LoadedState = InitialState;
type ProcessedState = usize;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ', '-', '>', ',']));

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    let mut chars = line.chars();
    if let Some((module_type_and_name, _)) = read_word(&mut chars, &DELIMITERS) {
        //read in the outputs
        let mut outputs: Vec<String> = Vec::default();
        while let Some((output_name, _)) = read_word(&mut chars, &DELIMITERS) {
            outputs.push(output_name);
        }
        let possible_name = module_type_and_name.substring(1, module_type_and_name.len());
        let (name, module) = match module_type_and_name.substring(0, 1) {
            "b" => {
                if module_type_and_name != "broadcaster" {
                    return Err(anyhow!(format!(
                        "Unexpected module name following 'b': {module_type_and_name}"
                    )));
                };
                ("broadcaster", Module::Broadcast { outputs })
            }
            "%" => (possible_name, Module::FlipFlop { on: false, outputs }),
            "&" => (
                possible_name,
                Module::Conjunction {
                    received: HashMap::default(),
                    outputs,
                },
            ),
            _ => {
                return Err(anyhow!(format!(
                    "indecipherable module type/name: {module_type_and_name}"
                )))
            }
        };
        state.insert(name.to_string(), module);
    }
    Ok(state)
}

fn get_outputs<'a>(module: &'a Module) -> &'a Vec<String> {
    match module {
        Module::Broadcast { outputs } => &outputs,
        Module::Conjunction {
            received: _received,
            outputs,
        } => &outputs,
        Module::FlipFlop { on: _on, outputs } => &outputs,
    }
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    //Set up all of the Conjunction states - we need to prime them with the incoming conections (set them all to Pulse::Low)
    let source_destinations: Vec<(String, String)> = state
        .iter_mut()
        .flat_map(|(name, module)| {
            let outputs = get_outputs(module);
            outputs.iter().map(|output| (name.clone(), output.clone()))
        })
        .collect();
    source_destinations
        .iter()
        .for_each(|(source, destination)| {
            let module = state.get_mut(destination);
            if let Some(Module::Conjunction {
                received,
                outputs: _outputs,
            }) = module
            {
                received.insert(source.clone(), Pulse::Low);
            }
        });
    Ok(state)
}

fn push_button<F>(state: &mut HashMap<String, Module>, notify_pulse: &F) -> (usize, usize, bool)
where
    F: Fn(&String, &Pulse, &String) -> bool,
{
    let mut low_pulse_count = 0;
    let mut high_pulse_count = 0;
    let mut got_wanted_notif = false;

    //Queue of source, pulse_type and destination
    let mut pulse_queue: VecDeque<(String, Pulse, String)> = VecDeque::default();
    //First send a low pulse to 'broadcaster'
    let button = "button".to_string();
    let broadcaster = "broadcaster".to_string();
    pulse_queue.push_back((button, Pulse::Low, broadcaster));

    while let Some((source, pulse, destination)) = pulse_queue.pop_front() {
        // println!("{source} -{pulse:?}-> {destination}");
        match pulse {
            Pulse::Low => {
                low_pulse_count += 1;
            }
            Pulse::High => {
                high_pulse_count += 1;
            }
        }
        got_wanted_notif = got_wanted_notif || notify_pulse(&source, &pulse, &destination);
        if !state.contains_key(&destination) {
            // println!("No destination '{destination}'");
            continue;
        }
        state.entry(destination.clone()).and_modify(|module| {
            match module {
                Module::Broadcast { outputs } => {
                    //Same pulse to all outputs
                    outputs.iter().for_each(|output| {
                        pulse_queue.push_back((destination.clone(), pulse, output.clone()))
                    });
                }
                Module::FlipFlop { on, outputs } => {
                    //Ignore high pulses, flip on low pulse and send high if now on, or low if now off
                    if matches!(pulse, Pulse::Low) {
                        *on = !*on;
                        let next_pulse = if *on { Pulse::High } else { Pulse::Low };
                        outputs.iter().for_each(|output| {
                            pulse_queue.push_back((destination.clone(), next_pulse, output.clone()))
                        });
                    }
                }
                Module::Conjunction { received, outputs } => {
                    //Update memory for the input
                    received.insert(source.clone(), pulse);
                    //If all inputs the same...
                    let all_same = received
                            .values()
                            .fold(received.values().next(), |acc, this| {
                                if matches!(acc, Some(pulse) if pulse == this) {
                                    acc
                                } else {
                                    None
                                }
                            });
                    let pulse = match all_same {
                        Some(Pulse::High) => Pulse::Low, //If all were the same and high, send a low
                        _ => Pulse::High, //otherwise send a high
                    };
                    outputs.iter().for_each(|output| {
                        pulse_queue.push_back((
                            destination.clone(),
                            pulse,
                            output.clone(),
                        ))
                    });
                }
            }
        });
    }
    // println!("Done ({low_pulse_count}, {high_pulse_count})");
    // println!();
    (low_pulse_count, high_pulse_count, got_wanted_notif)
}

const NUM_ITERATIONS: usize = 1000;

fn perform_processing_1(mut state: LoadedState) -> Result<ProcessedState, AError> {
    //Button, //Single one called 'button' sends a low to the 'broadcaster'
    let mut low_pulse_count: usize = 0;
    let mut high_pulse_count: usize = 0;
    (0..NUM_ITERATIONS).for_each(|_iteration| {
        let (num_low, num_high, _) = push_button(&mut state, &|_, _, _| true);
        low_pulse_count += num_low;
        high_pulse_count += num_high;
    });
    Ok(low_pulse_count * high_pulse_count)
}

fn perform_processing_2(mut state: LoadedState) -> Result<ProcessedState, AError> {
    //Button, //Single one called 'button' sends a low to the 'broadcaster'
    let mut num_presses = 0;
    loop {
        num_presses += 1;
        let (_num_low, _num_high, got_it) = push_button(&mut state, &|_, pulse, destination| {
            *pulse == Pulse::Low && *destination == "rx"
        });
        if got_it {
            break;
        }
    };
    Ok(num_presses)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        HashMap::default(),
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
        HashMap::default(),
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
