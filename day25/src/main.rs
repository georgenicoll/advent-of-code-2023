use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet, VecDeque},
    time,
};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use processor::{process, read_word};
use rand::seq::SliceRandom;

type AError = anyhow::Error;

type Id = usize;

#[derive(Debug, Clone, Default)]
struct Component {
    connections: HashSet<Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Connection {
    from: Id,
    to: Id,
}

impl Connection {
    fn new(from: &Id, to: &Id) -> Connection {
        let (from, to) = match from.cmp(to) {
            Ordering::Less => (from, to),
            Ordering::Greater => (to, from),
            Ordering::Equal => {
                panic!("Connection should not have the from and to the same: {from}")
            }
        };
        Connection {
            from: *from,
            to: *to,
        }
    }
}

#[derive(Default)]
struct State {
    names_to_ids: HashMap<String, Id>,
    ids_to_names: HashMap<Id, String>,
    components: HashMap<Id, Component>,
    connections: HashSet<Connection>,
}

type InitialState = State;
type LoadedState = InitialState;
type ProcessedState = usize;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([':', ' ']));

fn get_id_for_name(state: &mut InitialState, name: &String) -> Id {
    if state.names_to_ids.contains_key(name) {
        *state.names_to_ids.get(name).unwrap()
    } else {
        let new_id = state.ids_to_names.len();
        state.names_to_ids.insert(name.clone(), new_id);
        state.ids_to_names.insert(new_id, name.clone());
        new_id
    }
}

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        let mut chars = line.chars();
        let (name, _) = read_word(&mut chars, &DELIMITERS)
            .ok_or_else(|| anyhow!("Didn't find word: {line}"))?;
        let id = get_id_for_name(&mut state, &name);
        state.components.entry(id).or_default();
        while let Some((other, _)) = read_word(&mut chars, &DELIMITERS) {
            let other = &other;
            //connect to this component
            let other_id = get_id_for_name(&mut state, other);
            state.components.entry(id).and_modify(|component| {
                component.connections.insert(other_id);
            });
            //and connect this to the other as well
            state
                .components
                .entry(other_id)
                .or_default()
                .connections
                .insert(id);
            //and set up the connections we only keep one way
            let connection1 = Connection::new(&id, &other_id);
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

struct Subset {
    parent: usize,
    rank: usize,
}

impl Subset {
    fn new(parent: usize, rank: usize) -> Subset {
        Subset { parent, rank }
    }
}

fn find(subsets: &mut [Subset], id: Id) -> usize {
    if subsets[id].parent != id {
        subsets[id].parent = find(subsets, subsets[id].parent);
    }
    subsets[id].parent
}

fn union(subsets: &mut [Subset], x: Id, y: Id) {
    let x_root = find(subsets, x);
    let y_root = find(subsets, y);

    match subsets[x_root].rank.cmp(&subsets[y_root].rank) {
        Ordering::Less => subsets[x_root].parent = y_root,
        Ordering::Greater => subsets[y_root].parent = x_root,
        Ordering::Equal => {
            subsets[y_root].parent = x_root;
            subsets[x_root].rank += 1;
        }
    }
}

//Adapted from https://www.geeksforgeeks.org/introduction-and-implementation-of-kargers-algorithm-for-minimum-cut/
fn kargers_min_cut(state: &State) -> HashSet<Connection> {
    let mut subsets: Vec<Subset> = (0..state.components.len())
        .map(|i| Subset::new(i, 0))
        .collect();

    let mut connections = state.connections.iter().collect::<Vec<_>>();
    connections.shuffle(&mut rand::thread_rng());
    let mut connections_iter = connections.iter();

    let mut vertices = state.components.len();
    while vertices > 2 {
        let connection = if let Some(conn) = connections_iter.next() {
            conn
        } else {
            panic!("Ran out of connections :(");
        };

        // println!("{connection:?}");

        let subset1 = find(&mut subsets, connection.from);
        let subset2 = find(&mut subsets, connection.to);

        if subset1 == subset2 {
            continue;
        }

        // println!("{subset1} <- {subset2}");
        union(&mut subsets, subset1, subset2);
        vertices -= 1;
    }

    let mut cutedges: HashSet<Connection> = HashSet::default();
    for connection in connections {
        let subset1 = find(&mut subsets, connection.from);
        let subset2 = find(&mut subsets, connection.to);
        if subset1 != subset2 {
            cutedges.insert(connection.clone());
        }
    }
    cutedges
}

#[derive(Debug)]
struct Visit {
    current_group: Id,
    to_visit: Id,
}

impl Visit {
    fn new(current_group: &Id, to_visit: &Id) -> Visit {
        Visit {
            current_group: *current_group,
            to_visit: *to_visit,
        }
    }
}

/// find all of the groups, ignoring any connections in the disconnected_connections set
///
/// returns a map of component id to all connected component ids
fn get_groups(
    components: &HashMap<Id, Component>,
    disconnected_connections: &HashSet<Connection>,
) -> HashMap<Id, HashSet<Id>> {
    let mut component_ids = components.keys().cloned().collect::<HashSet<_>>();
    let mut result = HashMap::default();
    //Prime
    let first = component_ids.iter().next().unwrap();
    let mut to_visit: VecDeque<Visit> = VecDeque::from([Visit::new(first, first)]);
    //Pump
    while let Some(visit) = to_visit.pop_front() {
        if component_ids.contains(&visit.to_visit) {
            // Not Already visited
            component_ids.remove(&visit.to_visit); //now we have, add it to the group
            result
                .entry(visit.current_group)
                .or_insert_with(HashSet::default)
                .insert(visit.to_visit);
            //visit each of the connections (ignoring disconnected_connections)
            let component = components.get(&visit.to_visit).unwrap();
            for connection in component.connections.iter() {
                if component_ids.contains(connection) {
                    let connection1 = Connection::new(&visit.to_visit, connection);
                    if !disconnected_connections.contains(&connection1) {
                        //Not been disconnected - DFS
                        to_visit.push_front(Visit::new(&visit.current_group, connection));
                    }
                }
            }
        }
        //If the queue is empty, and there are more components, then visit the next one in the component_names
        if to_visit.is_empty() {
            if let Some(id) = component_ids.iter().next() {
                to_visit.push_front(Visit::new(id, id));
            }
        }
    }
    //Sanity
    if !component_ids.is_empty() {
        panic!("Still had some components!: {component_ids:?}")
    }
    result
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let mut cut_edges = HashSet::default();
    while cut_edges.len() != 3 {
        cut_edges = kargers_min_cut(&state);
    }
    //Now calculate the partition sizes.
    let partitions = get_groups(&state.components, &cut_edges);
    Ok(partitions
        .values()
        .map(|components| components.len())
        .product())
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
