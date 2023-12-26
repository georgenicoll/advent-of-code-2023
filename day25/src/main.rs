use std::{time, collections::{HashSet, HashMap}, cmp::Ordering};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use processor::{process, read_word};

type AError = anyhow::Error;

#[derive(Debug, Clone)]
struct Component {
    name: String,
    connections: HashSet<String>,
    connection_weights: HashMap<String, usize>,
    // combined_nodes: HashSet<String>,
}

impl Component {
    fn new(name: &String) -> Component {
        Component {
            name: name.clone(),
            connections: HashSet::default(),
            connection_weights: HashMap::default(),
            // combined_nodes: HashSet::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Connection {
    from: String,
    to: String,
}

impl Connection {
    fn new(from: &String, to: &String) -> Connection {
        let (from, to) = match from.cmp(to) {
            Ordering::Less => (from, to),
            Ordering::Greater => (to, from),
            Ordering::Equal => panic!("Connection should not have the from and to the same: {from}"),
        };
        Connection {
            from: from.clone(),
            to: to.clone(),
        }
    }
}

#[derive(Default)]
struct State {
    components: HashMap<String, Component>,
    connections: HashSet<Connection>,
}

type InitialState = State;
type LoadedState = InitialState;
type ProcessedState = usize;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([':', ' ']));

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        let mut chars = line.chars();
        let (name, _) = read_word(&mut chars, &DELIMITERS).ok_or_else(|| anyhow!("Didn't find word: {line}"))?;
        //add a Component if we don't already have one
        let name = &name;
        state.components.entry(name.clone()).or_insert_with(|| Component::new(name));
        while let Some((other, _)) = read_word(&mut chars, &DELIMITERS) {
            let other = &other;
            //connect to this component
            state.components.entry(name.clone()).and_modify(|component| {
                component.connections.insert(other.clone());
            });
            //and connect this to the other as well
            state.components.entry(other.clone()).or_insert_with(|| Component::new(other))
                .connections.insert(name.clone());
            //and set up the connections we only keep one way
            let connection1 = Connection::new(name, other);
            if !state.connections.contains(&connection1) {
                state.connections.insert(connection1);
            }
        }
    }
    Ok(state)
}

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
    Ok(state)
}

#[derive(Debug, Clone)]
struct CutOfThePhase {
    s: String,
    t: String,
    cut_weight: usize,
}

fn maximum_adjacency_search(components: &HashMap<String, Component>) -> CutOfThePhase {
    let start = components.keys().next().unwrap();
    let mut found_set: Vec<&String> = Vec::from([start]);
    let mut cut_weights: Vec<usize> = Vec::default();
    let mut candidates: HashSet<&String> = components.keys().collect();
    candidates.remove(start);

    while !candidates.is_empty() {
        let mut max_next_vertex: Option<&String> = None;
        let mut max_weight = usize::MIN;
        for candidate in candidates.iter() {
            let mut weight_sum = 0;
            let component = components.get(*candidate).unwrap();
            for found in found_set.iter() {
                if component.connections.contains(*found) {
                    weight_sum += component.connection_weights.get(*found).unwrap_or(&1);
                }
            }
            if weight_sum > max_weight {
                max_next_vertex = Some(candidate);
                max_weight = weight_sum;
            }
        }

        if let Some(next) = max_next_vertex {
            candidates.remove(&next);
            found_set.push(next);
            cut_weights.push(max_weight);
        }
    }
    //Sanity
    if found_set.len() < 2 {
        panic!("Not enough in found set")
    }
    CutOfThePhase {
        s: found_set[found_set.len() - 2].clone(),
        t: found_set[found_set.len() - 1].clone(),
        cut_weight: cut_weights[cut_weights.len() - 1],
    }
}

fn merge_vertices_from_cut(components: &mut HashMap<String, Component>, cut_of_the_phase: &CutOfThePhase) {
    // println!();
    // println!("{cut_of_the_phase:?}");
    //merge t into s
    let mut t = components.remove(&cut_of_the_phase.t).unwrap();
    // println!("t: {t:?}");
    let mut s = components.remove(&cut_of_the_phase.s).unwrap();
    // println!("s: {s:?}");

    //Use s as the new component
    // t.combined_nodes.into_iter().for_each(|node| {
    //     s.combined_nodes.insert(node);
    // });
    // s.combined_nodes.insert(t.name.clone());

    t.connections.drain().filter(|edge| *edge != s.name).for_each(|edge| {
        // println!("opposite t get: {edge}");
        let opposite = components.get_mut(&edge).unwrap();
        // println!("opposite t before: {opposite:?}");
        if s.connections.contains(&edge) {
            // println!("In s");
            //edges in common should be weighted higher
            let connection_weight = match (s.connection_weights.get(&edge), t.connection_weights.get(&edge)) {
                (None, None) => 2,
                (Some(weight), None) |
                (None, Some(weight)) => weight + 1,
                (Some(weight_s), Some(weight_t)) => weight_s + weight_t,
            };
            s.connection_weights.insert(edge.clone(), connection_weight);
            opposite.connection_weights.insert(s.name.clone(), connection_weight);
        } else {
            // println!("Not in s");
            //not in s, move it across
            s.connections.insert(edge.clone());
            opposite.connections.insert(s.name.clone());
            if let Some(weight) = opposite.connection_weights.get(&t.name) {
                s.connection_weights.insert(edge.clone(), *weight);
                opposite.connection_weights.insert(s.name.clone(), *weight);
            }
        }
        opposite.connections.remove(&t.name);
        opposite.connection_weights.remove(&t.name);
        // println!("opposite t: {opposite:?}");
    });
    //Final adjustments to s
    s.connections.remove(&t.name);
    s.connection_weights.remove(&t.name);
    //add the new node back in
    // println!("new s: {s:?}");
    components.insert(s.name.clone(), s);
}

fn find_min_cut(state: &State) ->(HashSet<String>, CutOfThePhase) {
    let mut components = state.components.clone();
    let mut current_partition: HashSet<String> = HashSet::default();
    let mut current_best_partition: Option<HashSet<String>> = None;
    let mut current_best_cut: Option<CutOfThePhase> = None;

    let started_at = time::Instant::now();
    while components.len() > 1 {
        println!("components.len: {} ({})", components.len(), started_at.elapsed().as_secs_f32());
        let cut_of_the_phase = maximum_adjacency_search(&components);
        let current_best_cut_weight = current_best_cut.iter().map(|cut| cut.cut_weight).next().unwrap_or(usize::MAX);

        current_partition.insert(cut_of_the_phase.t.clone());

        //Ugly but hey ho
        let mut first: HashSet<&String> = HashSet::default();
        let mut second: HashSet<&String> = HashSet::default();
        let mut cutting_connections: Vec<&Connection> = Vec::default();

        for component_name in state.components.keys() {
            if current_partition.contains(component_name) {
                first.insert(component_name);
            } else {
                second.insert(component_name);
            }
        }
        for connection in state.connections.iter() {
            if !(first.contains(&connection.from) && first.contains(&connection.to))
               && !(second.contains(&connection.from) && second.contains(&connection.to)) {
                cutting_connections.push(connection);
            }
        }
        if cutting_connections.len() == 3 {
            return (current_partition, cut_of_the_phase);
        }

        if cut_of_the_phase.cut_weight < current_best_cut_weight {
            current_best_partition = Some(current_partition.clone());
            current_best_cut = Some(cut_of_the_phase.clone());
        }
        merge_vertices_from_cut(&mut components, &cut_of_the_phase);
    }

    (current_best_partition.expect("No current best partition"), current_best_cut.expect("No current best cut"))
}

//Adapted from https://blog.thomasjungblut.com/graph/mincut/mincut/
fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let (partition, _best_cut) = find_min_cut(&state);

    let mut first: HashSet<String> = HashSet::default();
    let mut second: HashSet<String> = HashSet::default();
    let mut cutting_connections: Vec<Connection> = Vec::default();

    for component_name in state.components.keys() {
        if partition.contains(component_name) {
            first.insert(component_name.clone());
        } else {
            second.insert(component_name.clone());
        }
    }
    for connnection in state.connections {
        if !(first.contains(&connnection.from) && first.contains(&connnection.to))
           && !(second.contains(&connnection.from) && second.contains(&connnection.to)) {
            cutting_connections.push(connnection);
        }
    }
    println!("Cutting Connections: {cutting_connections:?}");

    Ok(first.len() * second.len())
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let started1_at = time::Instant::now();
    let result1 = process(
        file,
        State::default(),
        parse_line,
        finalise_state,
        perform_processing,
        calc_result,
    );
    match result1 {
        Ok(res) => println!(
            "Result 1: {:?} (took: {}s)",
            res,
            started1_at.elapsed().as_secs_f32()
        ),
        Err(e) => println!("Error on 1: {}", e),
    }

    let started2_at = time::Instant::now();
    let result2 = process(
        file,
        State::default(),
        parse_line,
        finalise_state,
        perform_processing,
        calc_result,
    );
    match result2 {
        Ok(res) => println!(
            "Result 2: {:?} (took: {}s)",
            res,
            started2_at.elapsed().as_secs_f32()
        ),
        Err(e) => println!("Error on 2: {}", e),
    }
}
