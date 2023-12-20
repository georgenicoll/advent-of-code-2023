use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fmt::{Display, Write},
};

use anyhow::anyhow;
use itertools::Itertools;
use num::Integer;
use once_cell::sync::Lazy;
use processor::{process, read_word};
use substring::Substring;

type AError = anyhow::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pulse {
    High,
    Low,
    NotSeen,
}

#[derive(Debug)]
enum Module {
    FlipFlop {
        on: bool,
        inputs: HashMap<String, Pulse>,
        outputs: Vec<String>,
    }, //'%', ignores high, flips on low,
    Conjunction {
        inputs: HashMap<String, Pulse>,
        outputs: Vec<String>,
    }, //'&', starts low on all
    Broadcast {
        inputs: HashMap<String, Pulse>,
        outputs: Vec<String>,
    }, //Single one 'broadcaster'
}

impl Module {
    fn inputs_string(inputs: &HashMap<String, Pulse>) -> String {
        inputs
            .iter()
            .map(|(name, pulse)| format!("{}={:?}", name, pulse))
            .join(",")
    }

    fn outputs_string(outputs: &Vec<String>) -> String {
        outputs.iter().join(",")
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (prefix, inputs, outputs) = match self {
            Module::FlipFlop {
                on,
                inputs,
                outputs,
            } => {
                let on = if *on { "on" } else { "off" };
                (format!("FlipFlop {} ", on), inputs, outputs)
            }
            Module::Conjunction { inputs, outputs } => {
                ("Conjunction ".to_string(), inputs, outputs)
            }
            Module::Broadcast { inputs, outputs } => ("Broadcast ".to_string(), inputs, outputs),
        };
        write!(
            f,
            "{prefix} -> ({}) -> ({})",
            Module::inputs_string(inputs),
            Module::outputs_string(outputs)
        )
    }
}

type InitialState = (String, BTreeMap<String, Module>);

type LoadedState = (String, BTreeMap<String, Module>);
type ProcessedState = usize;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ', '-', '>', ',']));

fn parse_line(mut istate: InitialState, line: String) -> Result<InitialState, AError> {
    let (output, mut state) = istate;
    let mut chars = line.chars();
    if let Some((module_type_and_name, _)) = read_word(&mut chars, &DELIMITERS) {
        //read in the outputs
        let mut inputs: HashMap<String, Pulse> = HashMap::default();
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
                ("broadcaster", Module::Broadcast { inputs, outputs })
            }
            "%" => (
                possible_name,
                Module::FlipFlop {
                    on: false,
                    inputs,
                    outputs,
                },
            ),
            "&" => (possible_name, Module::Conjunction { inputs, outputs }),
            _ => {
                return Err(anyhow!(format!(
                    "indecipherable module type/name: {module_type_and_name}"
                )))
            }
        };
        state.insert(name.to_string(), module);
    }
    Ok((output, state))
}

fn get_outputs<'a>(module: &'a Module) -> &'a Vec<String> {
    match module {
        Module::Broadcast {
            inputs: _input,
            outputs,
        } => &outputs,
        Module::Conjunction {
            inputs: _input,
            outputs,
        } => &outputs,
        Module::FlipFlop {
            on: _on,
            inputs: _input,
            outputs,
        } => &outputs,
    }
}

fn finalise_state(mut istate: InitialState) -> Result<LoadedState, AError> {
    let (output, mut state) = istate;
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
            match module {
                Some(Module::FlipFlop {
                    on: _on,
                    inputs,
                    outputs: _outputs,
                }) => {
                    inputs.insert(source.clone(), Pulse::NotSeen);
                }
                Some(Module::Broadcast {
                    inputs,
                    outputs: _outputs,
                }) => {
                    inputs.insert(source.clone(), Pulse::NotSeen);
                }
                Some(Module::Conjunction {
                    inputs,
                    outputs: _outputs,
                }) => {
                    inputs.insert(source.clone(), Pulse::Low);
                }
                _ => (),
            }
        });
    Ok((output, state))
}

fn push_button(state: &mut BTreeMap<String, Module>) -> (usize, usize) {
    let mut low_pulse_count = 0;
    let mut high_pulse_count = 0;

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
            _ => (),
        }
        if !state.contains_key(&destination) {
            // println!("No destination '{destination}'");
            continue;
        }
        state.entry(destination.clone()).and_modify(|module| {
            match module {
                Module::Broadcast { inputs, outputs } => {
                    inputs.insert(source.clone(), pulse);
                    //Same pulse to all outputs
                    outputs.iter().for_each(|output| {
                        pulse_queue.push_back((destination.clone(), pulse, output.clone()))
                    });
                }
                Module::FlipFlop {
                    on,
                    inputs,
                    outputs,
                } => {
                    inputs.insert(source.clone(), pulse);
                    //Ignore high pulses, flip on low pulse and send high if now on, or low if now off
                    if matches!(pulse, Pulse::Low) {
                        *on = !*on;
                        let next_pulse = if *on { Pulse::High } else { Pulse::Low };
                        outputs.iter().for_each(|output| {
                            pulse_queue.push_back((destination.clone(), next_pulse, output.clone()))
                        });
                    }
                }
                Module::Conjunction { inputs, outputs } => {
                    //Update memory for the input
                    inputs.insert(source.clone(), pulse);
                    //If all inputs the same...
                    let all_same = inputs.values().fold(inputs.values().next(), |acc, this| {
                        if matches!(acc, Some(pulse) if pulse == this) {
                            acc
                        } else {
                            None
                        }
                    });
                    let pulse = match all_same {
                        Some(Pulse::High) => Pulse::Low, //If all were the same and high, send a low
                        _ => Pulse::High,                //otherwise send a high
                    };
                    outputs.iter().for_each(|output| {
                        pulse_queue.push_back((destination.clone(), pulse, output.clone()))
                    });
                }
            }
        });
    }
    // println!("Done ({low_pulse_count}, {high_pulse_count})");
    // println!();
    (low_pulse_count, high_pulse_count)
}

const NUM_ITERATIONS: usize = 1000;

fn perform_processing_1(mut lstate: LoadedState) -> Result<ProcessedState, AError> {
    let (_, mut state) = lstate;
    //Button, //Single one called 'button' sends a low to the 'broadcaster'
    let mut low_pulse_count: usize = 0;
    let mut high_pulse_count: usize = 0;
    (0..NUM_ITERATIONS).for_each(|_iteration| {
        let (num_low, num_high) = push_button(&mut state);
        low_pulse_count += num_low;
        high_pulse_count += num_high;
    });
    Ok(low_pulse_count * high_pulse_count)
}

fn perform_processing_2(mut lstate: LoadedState) -> Result<ProcessedState, AError> {
    //Very specific to this input :(...
    //Watch the 4 inputs to rx and see what their cadences for outputting a High are.
    //&dr -> rx
    //&mp -> dr
    //&qt -> dr
    //&qb -> dr
    //&ng -> dr
    let (_output, mut state) = lstate;
    //Button, //Single one called 'button' sends a low to the 'broadcaster'
    let mut mp_num: Option<usize> = None;
    let mut qt_num: Option<usize> = None;
    let mut qb_num: Option<usize> = None;
    let mut ng_num: Option<usize> = None;
    let mut num_presses = 0;
    loop {
        num_presses += 1;
        let (_num_low, _num_high) = push_button(&mut state);
        // state.iter().for_each(|(name, module)| {
        //     println!("{name} : {module}");
        // });
        if let Some(Module::Conjunction {
            inputs,
            outputs: _outputs,
        }) = state.get("dr")
        {
            if mp_num.is_none() {
                if *inputs.get("mp").unwrap() == Pulse::High {
                    mp_num = Some(num_presses);
                    println!("mp = {num_presses}")
                }
            }
            if qt_num.is_none() {
                if *inputs.get("qt").unwrap() == Pulse::High {
                    qt_num = Some(num_presses);
                    println!("qt = {num_presses}")
                }
            }
            if qb_num.is_none() {
                if *inputs.get("qb").unwrap() == Pulse::High {
                    qb_num = Some(num_presses);
                    println!("qb = {num_presses}")
                }
            }
            if ng_num.is_none() {
                if *inputs.get("ng").unwrap() == Pulse::High {
                    ng_num = Some(num_presses);
                    println!("ng = {num_presses}")
                }
            }
        }
        if mp_num.is_some() && qt_num.is_some() && qb_num.is_some() && ng_num.is_some() {
            break;
        }
    }
    let mp_num = mp_num.unwrap();
    let qt_num = qt_num.unwrap();
    let qb_num = qb_num.unwrap();
    let ng_num = ng_num.unwrap();
    let result = mp_num.lcm(&qt_num).lcm(&qb_num).lcm(&ng_num);
    Ok(result)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

fn main() {
    //let (output, file) = ("a", test-input.txt");
    //let (output, file) = ("outputxx", "test-input2.txt");
    let (output, file) = ("rx", "input.txt");

    let result1 = process(
        file,
        (output.to_string(), BTreeMap::default()),
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
        (output.to_string(), BTreeMap::default()),
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
