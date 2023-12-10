use std::{
    collections::{BTreeMap, HashSet},
    fmt::Display,
};

use num::Integer;
use once_cell::sync::Lazy;
use processor::{process, read_word};

#[derive(Debug)]
enum Step {
    Left,
    Right,
}

impl Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Self::Left => 'L',
            Self::Right => 'R',
        };
        write!(f, "{c}")
    }
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Node {
    name: String,
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug)]
struct Path {
    node: Node,
    left: Node,
    right: Node,
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.left, self.right)
    }
}

#[derive(Debug, Default)]
struct State {
    steps: Vec<Step>,
    nodes: BTreeMap<Node, Path>,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "steps: ")?;
        self.steps
            .iter()
            .fold(Ok(()), |_, step| write!(f, "{step}"))?;
        writeln!(f)?;
        writeln!(f)?;
        self.nodes
            .iter()
            .fold(Ok(()), |_, (node, path)| writeln!(f, "{} = {}", node, path))
    }
}

enum LoadingState {
    Steps,
    Nodes,
}

type AError = anyhow::Error;
type InitialState = (LoadingState, State);
type LoadedState = State;
type ProcessedState = u64;
type FinalResult = u64;

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    //let file = "test-input3.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        (LoadingState::Steps, State::default()),
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
        (LoadingState::Steps, State::default()),
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

fn map_step(c: char) -> Step {
    match c {
        'L' => Step::Left,
        'R' => Step::Right,
        _ => panic!("Unrecognised Step character: {c}"),
    }
}

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ', '=', '(', ',', ')']));

fn parse_node_line(line: String) -> Option<(Node, Path)> {
    //JKT = (KFV, CFQ)
    let mut chars = line.chars();
    match read_word(&mut chars, &DELIMITERS) {
        Some((name, _)) => {
            let (left, _) = read_word(&mut chars, &DELIMITERS).expect("Failed to read left node");
            let (right, _) = read_word(&mut chars, &DELIMITERS).expect("Failed to read right node");
            let node = Node { name: name.clone() };
            Some((
                node,
                Path {
                    node: Node { name: name.clone() },
                    left: Node { name: left },
                    right: Node { name: right },
                },
            ))
        }
        None => None,
    }
}

fn parse_line(istate: InitialState, line: String) -> Result<InitialState, AError> {
    let (loading_state, mut state) = istate;
    let next_state = match loading_state {
        LoadingState::Steps => {
            state.steps = line.chars().map(map_step).collect();
            LoadingState::Nodes
        }
        LoadingState::Nodes => {
            if let Some((node, path)) = parse_node_line(line) {
                state.nodes.insert(node, path);
            }
            LoadingState::Nodes
        }
    };
    Ok((next_state, state))
}

fn finalise_state(istate: InitialState) -> Result<LoadedState, AError> {
    let (_, state) = istate;
    // println!("{}", state);
    Ok(state)
}

fn calc_steps<T>(state: &LoadedState, start: &Node, finish_check: T) -> u64
where
    T: Fn(&Node) -> bool,
{
    let mut iter = state.steps.iter();
    let mut step = iter.next();
    let mut current_path = state.nodes.get(start).expect("Didn't find start node/path");
    let mut num_steps = 0;
    while step.is_some() {
        let next_node = match step.unwrap() {
            Step::Left => &current_path.left,
            Step::Right => &current_path.right,
        };
        num_steps += 1;
        current_path = state
            .nodes
            .get(next_node)
            .expect("Failed to find next node");

        if finish_check(&current_path.node) {
            break;
        }

        step = match iter.next() {
            None => {
                iter = state.steps.iter();
                iter.next()
            }
            some_step => some_step,
        }
    }
    num_steps
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let start: Node = Node { name: "AAA".into() };
    let end: Node = Node { name: "ZZZ".into() };

    let num_steps = calc_steps(&state, &start, |node| *node == end);

    Ok(num_steps)
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let current_paths: Vec<&Path> = state
        .nodes
        .values()
        .filter(|path| path.node.name.ends_with('A'))
        .collect();

    let repetitions: Vec<u64> = current_paths
        .iter()
        .map(|current_path| calc_steps(&state, &current_path.node, |node| node.name.ends_with('Z')))
        .collect();
    // println!("{:?}", repetitions);

    let mut iter = repetitions.iter();
    let a = iter.next().unwrap();
    let b = iter.next().unwrap();
    let lcm = iter.fold(a.lcm(b), |current_lcm, next| current_lcm.lcm(next));

    Ok(lcm)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}
