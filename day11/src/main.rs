use std::collections::HashSet;

use processor::process;

type Int = u64;
type Coord = (Int, Int);

#[derive(Debug)]
struct Galaxy {
    id: Int,
    coord: Coord,
}

#[derive(Debug, Default)]
struct LoadingState {
    unexpanded_galaxies: Vec<Galaxy>,
    rows_with_galaxies: HashSet<Int>,
    columns_with_galaxies: HashSet<Int>,
    current_row: Int,
    current_galaxy: Int,
}

struct LoadedState {
    galaxies: Vec<Galaxy>,
}

type AError = anyhow::Error;
type InitialState = LoadingState;
type ProcessedState = Vec<Int>;
type FinalResult = Int;

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    for (index, char) in line.chars().enumerate() {
        if char == '#' {
            let coord = (index as Int, state.current_row as Int);
            let galaxy = Galaxy {
                id: state.current_galaxy,
                coord,
            };
            state.unexpanded_galaxies.push(galaxy);
            state.rows_with_galaxies.insert(state.current_row);
            state.columns_with_galaxies.insert(index as Int);
            state.current_galaxy += 1
        }
    }
    state.current_row += 1;
    Ok(state)
}

fn expand_galaxy(galaxy: &Galaxy, state: &LoadingState, increment: u64) -> Galaxy {
    let mut increment_x = 0u64;
    for i in 0..galaxy.coord.0 {
        if !state.columns_with_galaxies.contains(&i) {
            increment_x += increment
        }
    }
    let mut increment_y = 0u64;
    for i in 0..galaxy.coord.1 {
        if !state.rows_with_galaxies.contains(&i) {
            increment_y += increment
        }
    }
    let new_coord = (galaxy.coord.0 + increment_x, galaxy.coord.1 + increment_y);
    Galaxy {
        id: galaxy.id,
        coord: new_coord,
    }
}

fn expand_universe(state: &LoadingState, increment: u64) -> Vec<Galaxy> {
    state
        .unexpanded_galaxies
        .iter()
        .fold(Vec::default(), |mut acc, galaxy| {
            let expanded_galaxy = expand_galaxy(galaxy, state, increment);
            acc.push(expanded_galaxy);
            acc
        })
}

fn finalise_state_1(state: InitialState) -> Result<LoadedState, AError> {
    let expanded_galaxies = expand_universe(&state, 1);
    Ok(LoadedState {
        galaxies: expanded_galaxies,
    })
}

fn finalise_state_2(state: InitialState) -> Result<LoadedState, AError> {
    let expanded_galaxies = expand_universe(&state, 1000000 - 1);
    Ok(LoadedState {
        galaxies: expanded_galaxies,
    })
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let mut shortest_paths = Vec::default();
    for i in 0..state.galaxies.len() {
        for j in i + 1..state.galaxies.len() {
            let galaxy_a = state.galaxies.get(i).unwrap();
            let galaxy_b = state.galaxies.get(j).unwrap();
            let horizontal_distance = galaxy_b.coord.0 as i64 - galaxy_a.coord.0 as i64;
            let vertical_distance = galaxy_b.coord.1 as i64 - galaxy_a.coord.1 as i64;
            shortest_paths
                .push(horizontal_distance.unsigned_abs() + vertical_distance.unsigned_abs());
        }
    }
    Ok(shortest_paths)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state.iter().sum())
}

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        LoadingState::default(),
        parse_line,
        finalise_state_1,
        perform_processing,
        calc_result,
    );
    match result1 {
        Ok(res) => println!("Result 1: {:?}", res),
        Err(e) => println!("Error on 1: {}", e),
    }

    let result2 = process(
        file,
        LoadingState::default(),
        parse_line,
        finalise_state_2,
        perform_processing,
        calc_result,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}
