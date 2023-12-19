use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use processor::{process, read_next, read_word};

type AError = anyhow::Error;

#[derive(Debug)]
enum Check {
    LessThan { lt_amount: usize },
    GreaterThan { gt_amount: usize },
}

#[derive(Debug, Clone)]
enum Destination {
    Rejected,
    Accepted,
    Workflow { name: String },
}

impl Display for Destination {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Destination::Rejected => "Rejected".to_string(),
            Destination::Accepted => "Accepted".to_string(),
            Destination::Workflow { name } => format!("Worflow '{name}'"),
        };
        write!(f, "{output}")
    }
}

#[derive(Debug)]
struct Rule {
    attribute: char,
    check: Check,
    destination: Destination,
}

#[derive(Debug)]
struct Workflow {
    name: String,
    rules: Vec<Rule>,
    unmatched_destination: Destination,
}

#[derive(Debug, Clone)]
struct Part {
    _index: usize,
    attributes: HashMap<char, usize>,
}

enum LoadingState {
    Workflows,
    Parts,
}

#[derive(Debug, Default)]
struct State {
    workflows: HashMap<String, Workflow>,
    parts: Vec<Part>,
}

type InitialState = (LoadingState, State);
type LoadedState = State;
type ProcessedState = usize;
type FinalResult = usize;

static WORKFLOW_DELIMITERS: Lazy<HashSet<char>> =
    Lazy::new(|| HashSet::from(['{', '}', ':', ',', '<', '>']));

fn parse_check(delimiter: char, amount: usize) -> Check {
    match delimiter {
        '>' => Check::GreaterThan { gt_amount: amount },
        '<' => Check::LessThan { lt_amount: amount },
        _ => panic!("Unrecognised check delimiter: {delimiter}"),
    }
}

fn parse_destination(s: String) -> Destination {
    match s.as_str() {
        "A" => Destination::Accepted,
        "R" => Destination::Rejected,
        _ => Destination::Workflow { name: s },
    }
}

fn load_worflow(line: String) -> Workflow {
    let mut chars = line.chars();
    //px{a<2006:qkq,m>2090:A,rfg}
    let (name, _) = read_word(&mut chars, &WORKFLOW_DELIMITERS).expect("No name");
    let mut rules = Vec::default();
    let mut unmatched_destination = None;
    while let Some((attribute_or_destination, delimiter)) =
        read_word(&mut chars, &WORKFLOW_DELIMITERS)
    {
        if matches!(delimiter, Some('>') | Some('<')) {
            let attribute = attribute_or_destination
                .chars()
                .next()
                .expect("Was empty attribute");
            let (amount, _) = read_next::<usize>(&mut chars, &WORKFLOW_DELIMITERS).unwrap();
            let check = parse_check(delimiter.unwrap(), amount);
            let (destination, _) = read_word(&mut chars, &WORKFLOW_DELIMITERS).unwrap();
            let destination = parse_destination(destination);
            rules.push(Rule {
                attribute,
                check,
                destination,
            })
        } else {
            unmatched_destination = Some(parse_destination(attribute_or_destination));
            continue;
        }
    }
    Workflow {
        name,
        rules,
        unmatched_destination: unmatched_destination.expect("Didn't get the unmatched destination"),
    }
}

static PART_DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from(['{', '}', '=', ',']));

fn load_part(part_index: usize, line: String) -> Part {
    let mut chars = line.chars();
    let mut attributes = HashMap::default();
    while let Some((attribute, _)) = read_word(&mut chars, &PART_DELIMITERS) {
        let (attribute_value, _) =
            read_next::<usize>(&mut chars, &PART_DELIMITERS).expect("Reading part value");
        attributes.insert(attribute.chars().next().unwrap(), attribute_value);
    }
    Part {
        _index: part_index,
        attributes,
    }
}

fn parse_line(istate: InitialState, line: String) -> Result<InitialState, AError> {
    let (loading_state, mut state) = istate;
    if line.is_empty() {
        return Ok((LoadingState::Parts, state));
    };
    match loading_state {
        LoadingState::Workflows => {
            let wf = load_worflow(line);
            state.workflows.insert(wf.name.clone(), wf);
        }
        LoadingState::Parts => {
            let part = load_part(state.parts.len(), line);
            state.parts.push(part);
        }
    }
    Ok((loading_state, state))
}

fn finalise_state(istate: InitialState) -> Result<LoadedState, AError> {
    let (_, state) = istate;
    Ok(state)
}

const INITIAL_WORKFLOW: &str = "in";

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let mut accepted_parts: Vec<Part> = Vec::default();
    // let mut rejected_parts: Vec<Part> = Vec::default();
    for part in state.parts.iter() {
        let mut current_wf = Some(INITIAL_WORKFLOW.to_string());
        while let Some(workflow_name) = current_wf {
            let workflow = state
                .workflows
                .get(workflow_name.as_str())
                .ok_or_else(|| anyhow!(format!("No workflow found with name '{workflow_name}'")))?;
            let mut destination: Option<Destination> = None;
            for rule in workflow.rules.iter() {
                let part_value = *part.attributes.get(&rule.attribute).ok_or_else(|| {
                    anyhow!(format!(
                        "Rule had attribute '{}' but was not found in {part:?}",
                        rule.attribute
                    ))
                })?;
                match rule.check {
                    Check::GreaterThan { gt_amount } => {
                        if part_value > gt_amount {
                            destination = Some(rule.destination.clone());
                            break;
                        }
                    }
                    Check::LessThan { lt_amount } => {
                        if part_value < lt_amount {
                            destination = Some(rule.destination.clone());
                            break;
                        }
                    }
                }
            }

            let destination = destination.unwrap_or(workflow.unmatched_destination.clone());
            match destination {
                Destination::Accepted => {
                    accepted_parts.push(part.clone());
                    current_wf = None;
                }
                Destination::Rejected => {
                    // rejected_parts.push(part.clone());
                    current_wf = None;
                }
                Destination::Workflow { name } => {
                    current_wf = Some(name);
                }
            }
        }
    }

    let result = accepted_parts
        .iter()
        .map(|part| part.attributes.values().sum::<usize>())
        .sum();
    Ok(result)
}

type MinMax = (usize, usize);

#[derive(Debug)]
struct PartPossibilities {
    attributes: HashMap<char, MinMax>,
}

struct ToProcess {
    possibilities: PartPossibilities,
    workflow: String,
}

fn process_to_direction(
    accepted: &mut Vec<PartPossibilities>,
    to_process: &mut VecDeque<ToProcess>,
    original_possibilities: &PartPossibilities,
    attribute: char,
    min_max: MinMax,
    destination: &Destination,
) {
    match destination {
        Destination::Accepted => {
            let mut new_attributes = original_possibilities.attributes.clone();
            new_attributes.insert(attribute, min_max);
            accepted.push(PartPossibilities {
                attributes: new_attributes,
            });
        }
        Destination::Workflow { name } => {
            let mut new_attributes = original_possibilities.attributes.clone();
            new_attributes.insert(attribute, min_max);
            to_process.push_back(ToProcess {
                possibilities: PartPossibilities {
                    attributes: new_attributes,
                },
                workflow: name.clone(),
            });
        }
        Destination::Rejected => (), //drop it
    }
}

fn process_next(
    workflows: &HashMap<String, Workflow>,
    accepted: &mut Vec<PartPossibilities>,
    to_process: &mut VecDeque<ToProcess>,
    this_one: ToProcess,
) {
    // println!("Processing at {}: {:?}", this_one.workflow, this_one.possibilities);
    let workflow = workflows.get(&this_one.workflow).unwrap();
    let mut part_possibilities = Some(this_one.possibilities);
    for rule in workflow.rules.iter() {
        if let Some(possibilites) = part_possibilities {
            let (min, max) = possibilites.attributes.get(&rule.attribute).unwrap();

            let (matched, unmatched) = match rule.check {
                Check::GreaterThan { gt_amount } => {
                    if gt_amount < *min {
                        //all match
                        (Some((*min, *max)), None)
                    } else if gt_amount >= *max {
                        //none match
                        (None, Some((*min, *max)))
                    } else {
                        //some match
                        (Some((gt_amount + 1, *max)), Some((*min, gt_amount)))
                    }
                }
                Check::LessThan { lt_amount } => {
                    if lt_amount > *max {
                        //all match
                        (Some((*min, *max)), None)
                    } else if lt_amount <= *min {
                        //none match
                        (None, Some((*min, *max)))
                    } else {
                        //some match
                        (Some((*min, lt_amount - 1)), Some((lt_amount, *max)))
                    }
                }
            };
            if let Some(matched) = matched {
                process_to_direction(
                    accepted,
                    to_process,
                    &possibilites,
                    rule.attribute,
                    matched,
                    &rule.destination,
                );
            }
            part_possibilities = unmatched.map(|unmatched| {
                let mut new_attributes = possibilites.attributes.clone();
                new_attributes.insert(rule.attribute, unmatched);
                PartPossibilities {
                    attributes: new_attributes,
                }
            })
        }
    }
    //default?
    if let Some(possibilities) = part_possibilities {
        match &workflow.unmatched_destination {
            Destination::Accepted => accepted.push(possibilities),
            Destination::Workflow { name } => to_process.push_back(ToProcess {
                possibilities,
                workflow: name.clone(),
            }),
            Destination::Rejected => (), //drop it
        }
    }
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let mut accepted_possibilities: Vec<PartPossibilities> = Vec::default();
    //Push through the possibilities splitting them as required until they reach a final state (A or R)
    let mut to_process: VecDeque<ToProcess> = VecDeque::default();
    //prime
    to_process.push_back(ToProcess {
        possibilities: PartPossibilities {
            attributes: HashMap::from([
                ('x', (1, 4000)),
                ('m', (1, 4000)),
                ('a', (1, 4000)),
                ('s', (1, 4000)),
            ]),
        },
        workflow: INITIAL_WORKFLOW.to_string(),
    });
    //Pump
    while let Some(next_to_process) = to_process.pop_front() {
        process_next(
            &state.workflows,
            &mut accepted_possibilities,
            &mut to_process,
            next_to_process,
        );
    }
    //Calculate the final combinations and sum
    let result = accepted_possibilities
        .iter()
        .map(|possibility| {
            possibility
                .attributes
                .values()
                .fold(1usize, |acc, (min, max)| acc * (*max - *min + 1))
        })
        .sum();
    Ok(result)
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
        (LoadingState::Workflows, State::default()),
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
        (LoadingState::Workflows, State::default()),
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
