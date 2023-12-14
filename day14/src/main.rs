use std::fmt::Display;

use processor::{ok_identity, process, read_word, Cells, CellsBuilder, BLANK_DELIMITERS};

#[derive(Debug, Clone, Copy, Default)]
enum Cell {
    #[default]
    Space,
    RoundRock,
    CubeRock,
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Cell::Space => '.',
            Cell::RoundRock => 'O',
            Cell::CubeRock => '#',
        };
        write!(f, "{c}")
    }
}

enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Default)]
struct LoadingState {
    grid: CellsBuilder<Cell>,
}

struct LoadedState {
    grid: Cells<Cell>,
}

type AError = anyhow::Error;
type InitialState = LoadingState;
type ProcessedState = LoadedState;
type ProcessedState2 = usize;
type FinalResult = usize;

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    match read_word(&mut line.chars(), &BLANK_DELIMITERS) {
        Some((line, _)) => {
            state.grid.new_line();
            line.chars().for_each(|c| match c {
                '.' => state
                    .grid
                    .add_cell(Cell::Space)
                    .expect("Failed to add space cell"),
                'O' => state
                    .grid
                    .add_cell(Cell::RoundRock)
                    .expect("Failed to add cube rock cell"),
                '#' => state
                    .grid
                    .add_cell(Cell::CubeRock)
                    .expect("Failed to add cube rock cell"),
                _ => panic!("unrecognised cell: {c}"),
            })
        }
        None => panic!("Expect all lines to contain something"),
    };
    Ok(state)
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    let grid = state.grid.build_cells(Cell::Space)?;
    println!("Loaded:");
    println!("{grid}");
    Ok(LoadedState { grid })
}

/// Returns true if the cell could be moved
fn try_moving_cell(
    grid: &mut Cells<Cell>,
    x: usize,
    y: usize,
    delta_x: isize,
    delta_y: isize,
) -> bool {
    let next_x = x as isize + delta_x;
    let next_y = y as isize + delta_y;
    if !grid.in_bounds(next_x, next_y) {
        return false; //out of bounds
    }
    let next_x = next_x as usize;
    let next_y = next_y as usize;
    let next_cell = grid.get(next_x, next_y).unwrap();
    if matches!(next_cell, Cell::Space) {
        grid.swap(x, y, next_x, next_y).unwrap();
        true
    } else {
        false
    }
}

fn move_cell(grid: &mut Cells<Cell>, x: usize, y: usize, delta_x: isize, delta_y: isize) {
    let cell = grid.get(x, y).unwrap();
    //only round rocks move
    if matches!(cell, Cell::RoundRock) {
        let mut current_x = x;
        let mut current_y = y;
        while try_moving_cell(grid, current_x, current_y, delta_x, delta_y) {
            current_x = (current_x as isize + delta_x) as usize;
            current_y = (current_y as isize + delta_y) as usize;
        }
    }
}

fn tilt_grid_from_top_left(grid: &mut Cells<Cell>, delta_x: isize, delta_y: isize) {
    for y in 0..grid.side_lengths.1 {
        for x in 0..grid.side_lengths.0 {
            move_cell(grid, x, y, delta_x, delta_y);
        }
    }
}

fn tilt_grid_from_bottom_right(grid: &mut Cells<Cell>, delta_x: isize, delta_y: isize) {
    for y in (0..grid.side_lengths.1).rev() {
        for x in (0..grid.side_lengths.0).rev() {
            move_cell(grid, x, y, delta_x, delta_y);
        }
    }
}

fn tilt(grid: &mut Cells<Cell>, direction: Direction) {
    match direction {
        Direction::North => tilt_grid_from_top_left(grid, 0, -1),
        Direction::East => tilt_grid_from_bottom_right(grid, 1, 0),
        Direction::South => tilt_grid_from_bottom_right(grid, 0, 1),
        Direction::West => tilt_grid_from_top_left(grid, -1, 0),
    }
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let mut tilted_grid = state.grid.clone();
    tilt(&mut tilted_grid, Direction::North);
    println!("tilted:");
    println!("{tilted_grid}");
    Ok(ProcessedState { grid: tilted_grid })
}

static TARGET_CYCLES: usize = 1000000000;
static INVESTIGATION_CYCLES: usize = 10000;
static DISPLAY_LAST: usize = 100;
static NUM_CHECKS: usize = 10;

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState2, AError> {
    let mut grid = state.grid.clone();
    let mut cycle_loads = Vec::with_capacity(INVESTIGATION_CYCLES);
    for cycle in 0..INVESTIGATION_CYCLES {
        //N -> W -> S -> E
        tilt(&mut grid, Direction::North);
        tilt(&mut grid, Direction::West);
        tilt(&mut grid, Direction::South);
        tilt(&mut grid, Direction::East);
        let load = calculate_total_load(&grid, Direction::North);
        if cycle > INVESTIGATION_CYCLES - DISPLAY_LAST || cycle % 1000 == 0 {
            println!("cycle {cycle}: {load}");
        }
        cycle_loads.push(load);
    }
    let repetition_end = cycle_loads.len() - 1;
    let end_load = cycle_loads[repetition_end];
    println!(
        "Calculating the repetition length from cycle index {}, with load {}",
        repetition_end, end_load
    );
    let mut repetition_size: Option<usize> = None;
    let mut candidate_start = repetition_end;
    while candidate_start > 1000 {
        candidate_start -= 1;
        let candidate_load = cycle_loads[candidate_start];
        if candidate_load == end_load {
            let candidate_size = repetition_end - candidate_start;
            println!(
                "Found candidate at index {} with size {}, checking {} repetitions",
                candidate_start, candidate_size, NUM_CHECKS
            );
            if (0..NUM_CHECKS).all(|i| {
                let repetition_load = cycle_loads[repetition_end - candidate_size * i];
                repetition_load == end_load
            }) {
                println!("Found repetition of size {}", candidate_size);
                repetition_size = Some(candidate_size);
                break;
            }
            println!("Repetition not found, continuing...");
        }
    }
    let repetition_size = repetition_size.expect("Didn't find a repetition");
    let target_index = TARGET_CYCLES - 1;
    let index_difference = target_index - repetition_end;
    let modulus = index_difference % repetition_size;
    println!("{} % {} = {}", index_difference, repetition_size, modulus);
    let final_load = cycle_loads[repetition_end - (repetition_size - modulus)];
    Ok(final_load)
}

fn calculate_load(
    grid: &Cells<Cell>,
    cell: &Cell,
    _x: usize,
    y: usize,
    direction: &Direction,
) -> usize {
    // calculating the weight on the north support beam
    if matches!(cell, Cell::RoundRock) {
        match direction {
            Direction::North => grid.side_lengths.1 - y,
            _ => panic!("unhanded direction"),
        }
    } else {
        0
    }
}

fn calculate_total_load(grid: &Cells<Cell>, direction: Direction) -> usize {
    grid.iter().fold(0usize, |acc, ((x, y), cell)| {
        acc + calculate_load(grid, cell, x, y, &direction)
    })
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(calculate_total_load(&state.grid, Direction::North))
}

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        LoadingState::default(),
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
        LoadingState::default(),
        parse_line,
        finalise_state,
        perform_processing_2,
        ok_identity,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}
