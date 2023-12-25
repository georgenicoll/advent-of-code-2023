use std::{time, collections::{HashSet, HashMap, BTreeSet, VecDeque, BTreeMap}, cmp::Ordering};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use processor::{process, read_word};

type AError = anyhow::Error;

struct Component {
    _name: String,
    connections: BTreeSet<String>,
}

impl Component {
    fn new(name: &String) -> Component {
        Component {
            _name: name.clone(),
            connections: BTreeSet::default(),
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
    components: BTreeMap<String, Component>,
    connections: BTreeSet<Connection>,
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

#[derive(Debug)]
struct Visit {
    current_group: String,
    to_visit: String,
}

impl Visit {
    fn new(current_group: &String, to_visit: &String) -> Visit {
        Visit {
            current_group: current_group.clone(),
            to_visit: to_visit.clone(),
        }
    }
}

/// find all of the groups, ignoring any connections in the disconnected_connections set
///
/// returns a map of component name to all connected component names
fn get_groups(
    components: &BTreeMap<String, Component>,
    disconnected_connections: &HashSet<Connection>,
    max_groups: usize,
)
-> Option<HashMap<String, HashSet<String>>> {
    let mut component_names = components.keys().cloned().collect::<HashSet<_>>();
    let mut result = HashMap::default();
    //Prime
    let first = component_names.iter().next().unwrap();
    let mut to_visit: VecDeque<Visit> = VecDeque::from([Visit::new(first, first)]);
    //Pump
    while let Some(visit) = to_visit.pop_front() {
        let component_name = &visit.to_visit;
        if component_names.contains(component_name) { // Not Already visited
            component_names.remove(component_name); //now we have
            //add it to the group
            let group_name = &visit.current_group;
            if !result.contains_key(group_name) {
                let component_names = HashSet::default();
                result.insert(group_name.clone(), component_names);
            }
            result.get_mut(group_name).unwrap().insert(component_name.clone());
            //visit each of the connections (ignoring disconnected_connections)
            let component = components.get(component_name).unwrap();
            for connection in component.connections.iter() {
                if component_names.contains(connection) {
                    let connection1 = Connection::new(&component_name, &connection);
                    if !disconnected_connections.contains(&connection1) {
                        //Not been disconnected - DFS
                        to_visit.push_front(Visit::new(&group_name, &connection));
                    }
                }
            }
        }
        //If we're empty, and there are more components.  Then visit the next one in the component_names
        //but only if itr  we didn't get over the max groups
        if to_visit.is_empty() {
            if let Some(name) = component_names.iter().next() {
                if result.len() >= max_groups {
                    //we can stop now - this would push us over the number of groups
                    return None;
                }
                to_visit.push_back(Visit::new(name, name));
            }
        }
    }
    //Sanity
    if !component_names.is_empty() {
        panic!("Still had some components!: {component_names:?}")
    }
    if result.len() == max_groups {
        Some(result)
    } else {
        None
    }
}

struct Visit2 {
    to_visit: String,
    depth: usize,
}

impl Visit2 {
    fn new(to_visit: &String, depth: usize) -> Visit2 {
        Visit2 {
            to_visit: to_visit.clone(),
            depth,
        }
    }
}

fn get_connected_within_distance(components: &BTreeMap<String, Component>, component_name: &String, distance: usize) -> HashSet<String> {
    //starting at component, try to get to to as many components as possible
    let mut visited: HashSet<String> = HashSet::default();
    let mut to_visit: VecDeque<Visit2> = VecDeque::from([Visit2::new(component_name, 0)]);
    //BFS
    while let Some(visit) = to_visit.pop_front() {
        if visited.contains(&visit.to_visit) {
            continue;
        }
        //got to the max depth?
        if visit.depth == distance {
            continue;
        }
        //otherwise - visit the connections
        let component = components.get(&visit.to_visit).unwrap();
        for connection in component.connections.iter() {
            to_visit.push_back(Visit2::new(connection, visit.depth + 1));
        }
        visited.insert(visit.to_visit);
    }
    visited
}

fn get_end_point_connections(components: &BTreeMap<String, Component>, connection: &Connection) -> BTreeSet<String> {
    let component1 = components.get(&connection.from).unwrap();
    let component2 = components.get(&connection.to).unwrap();
    component1.connections.union(&component2.connections).cloned().collect()
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    // state.components.iter()
    //     .map(|(name, _)| (
    //         name,
    //         get_connected_within_distance(&state.components, name, 2)
    //     ))
    //     .for_each(|(name, connected)|
    //         println!("{name}: {connected:?}")
    //     );
    let mut connections = state.connections
        .iter()
        .map(|connection| (
            connection.clone(),
            get_end_point_connections(&state.components, &connection)
        ))
        .collect::<Vec<_>>();
    //Biggest number of connections first
    connections.sort_by(|(_, end_point_connections1), (_, end_point_connections2)|
        end_point_connections1.len().cmp(&end_point_connections2.len()).reverse()
    );
    connections.iter().for_each(|(connection, connections)| {
        println!("{:?}: {:?}", connection, connections);
    });

    let mut found_groups: Option<HashMap<String, HashSet<String>>> = None;
    //remove 3 connections and then see if we get 2 distinct groups
    let started = time::Instant::now();
    'outer: for i in 0..connections.len() {
        let (connection1, _) = &connections[i];
        for j in (i + 1)..connections.len() {
            println!("{i} {j}: {}", started.elapsed().as_secs());
            let (connection2, _) = &connections[j];
            for k in (j+1)..connections.len() {
                let (connection3, _) = &connections[k];
                // println!("{i}, {j}, {k}: {:?}, {:?}, {:?}", connection1, connection2, connection3);
                let disconnected_connections = HashSet::from([connection1.clone(), connection2.clone(), connection3.clone()]);
                if let Some(groups) = get_groups(&state.components, &disconnected_connections, 2) {
                    if groups.len() == 2 {
                        found_groups = Some(groups);
                        break 'outer;
                    }
                }
            }
        }
    }
    let found_groups = found_groups.expect("Didn't find the groups");
    let result = found_groups.values().map(|group| group.len()).product();
    Ok(result)
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
