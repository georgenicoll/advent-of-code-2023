use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
    mem::swap,
};

use anyhow::anyhow;
use processor::{adjacent_coords_cartesian, process, Cells, CellsBuilder};

type AError = anyhow::Error;

#[derive(Debug, Clone, Copy)]
enum Tile {
    Plot,
    Rock,
}

impl Tile {
    fn char_rep(&self) -> char {
        match self {
            Self::Plot => '.',
            Self::Rock => '#',
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char_rep())
    }
}

type Coord = (usize, usize);

struct LoadingState {
    total_steps: usize,
    total_to_calculate: usize,
    start: Option<Coord>,
    tiles: CellsBuilder<Tile>,
}

type InitialState = LoadingState;

struct LoadedState {
    total_steps: usize,
    total_to_calculate: usize,
    start: Coord,
    tiles: Cells<Tile>,
}

type ProcessedState = usize;
type FinalResult = usize;

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        state.tiles.new_line();
        for c in line.chars() {
            let (tile, was_start) = match c {
                '.' => (Tile::Plot, false),
                '#' => (Tile::Rock, false),
                'S' => (Tile::Plot, true),
                _ => {
                    return Err(anyhow!(format!("Unrecognised tile: {c}")));
                }
            };
            state.tiles.add_cell(tile)?;
            if was_start {
                state.start = state.tiles.current_cell();
            }
        }
    }
    Ok(state)
}

fn output_state(_state: &LoadedState) {
    // println!("=== State ===:");
    // println!("Start: {:?}", state.start);
    // println!("{}", state.tiles);
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    let loaded = LoadedState {
        total_steps: state.total_steps,
        total_to_calculate: state.total_to_calculate,
        start: state.start.ok_or_else(|| anyhow!("No start found"))?,
        tiles: state.tiles.build_cells(Tile::Plot)?,
    };
    output_state(&loaded);
    Ok(loaded)
}

fn make_step(tiles: &Cells<Tile>, current_position: &Coord, next_positions: &mut HashSet<Coord>) {
    adjacent_coords_cartesian(current_position, &tiles.side_lengths)
        .iter()
        .for_each(|(candidate_x, candidate_y)| {
            let tile = tiles.get(*candidate_x, *candidate_y).unwrap();
            if matches!(tile, Tile::Plot) {
                next_positions.insert((*candidate_x, *candidate_y));
            }
        })
}

fn perform_walk(state: &LoadedState) -> usize {
    let mut current_positions: HashSet<Coord> = HashSet::default();
    let mut next_positions: HashSet<Coord> = HashSet::default();
    //start at start
    current_positions.insert(state.start);
    //make the steps
    for _i in 0..state.total_steps {
        current_positions
            .iter()
            .for_each(|position| make_step(&state.tiles, position, &mut next_positions));
        swap(&mut current_positions, &mut next_positions);
        next_positions.clear();
    }
    current_positions.len()
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    Ok(perform_walk(&state))
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

type Coord2 = (isize, isize);

fn get_position_in_bounds(side_lengths: &Coord, candidate_x: isize, candidate_y: isize) -> Coord {
    let x = candidate_x % side_lengths.0 as isize;
    let x = if x < 0 {
        side_lengths.0 as isize + x
    } else {
        x
    };
    let y = candidate_y % side_lengths.1 as isize;
    let y = if y < 0 {
        side_lengths.1 as isize + y
    } else {
        y
    };
    (x as usize, y as usize)
}

fn try_make_step(
    tiles: &Cells<Tile>,
    next_positions: &mut HashSet<Coord2>,
    candidate_x: isize,
    candidate_y: isize,
) {
    //get the cell within the bounds of the tiles
    let (x, y) = get_position_in_bounds(&tiles.side_lengths, candidate_x, candidate_y);
    let tile = tiles.get(x, y).unwrap();
    if matches!(tile, Tile::Plot) {
        next_positions.insert((candidate_x, candidate_y));
    }
}

fn make_step_2(
    tiles: &Cells<Tile>,
    current_position: &Coord2,
    next_positions: &mut HashSet<Coord2>,
) {
    let (current_x, current_y) = *current_position;
    //North
    try_make_step(tiles, next_positions, current_x, current_y - 1);
    //East
    try_make_step(tiles, next_positions, current_x + 1, current_y);
    //South
    try_make_step(tiles, next_positions, current_x, current_y + 1);
    //West
    try_make_step(tiles, next_positions, current_x - 1, current_y);
}

fn perform_walk_2(state: &LoadedState) -> Vec<isize> {
    let mut lengths = Vec::with_capacity(state.total_steps);
    let mut current_positions: HashSet<Coord2> = HashSet::default();
    let mut next_positions: HashSet<Coord2> = HashSet::default();
    //start at start
    let start = (state.start.0 as isize, state.start.1 as isize);
    current_positions.insert(start);
    //make the steps
    for i in 0..state.total_steps {
        current_positions
            .iter()
            .for_each(|position| make_step_2(&state.tiles, position, &mut next_positions));
        swap(&mut current_positions, &mut next_positions);
        next_positions.clear();
        lengths.push(current_positions.len() as isize);
        if (i + 1) % 50 == 0 {
            println!("Calculated to {} ({})", i + 1, current_positions.len())
        }
    }
    lengths
}

type ProcessedState2 = (usize, Vec<isize>);

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState2, AError> {
    let first_n = perform_walk_2(&state);
    Ok((state.total_to_calculate, first_n))
}

//Used to investigate the repeat size finding code
// fn load_values(file_name: &str) -> Vec<isize> {
//     let file = File::open(file_name).unwrap();
//     BufReader::new(file)
//         .lines()
//         .map(|l| l.unwrap())
//         .map(|l| l.parse::<isize>().unwrap())
//         .collect()
// }

fn get_differences(nums: &[isize]) -> Vec<isize> {
    nums.windows(2)
        .map(|window| {
            let first = window[0];
            let second = window[1];
            second - first
        })
        .collect()
}

const NUM_REPEAT_CHECKS: usize = 1;

fn calculate_repeat_size(second_order_differences: &Vec<isize>) -> (usize, Vec<isize>) {
    let end_repeat_index = second_order_differences.len() - 1;
    let mut candidate_repeat_size = 0usize;
    'outer: loop {
        candidate_repeat_size += 1;
        // println!("Trying repeat size of {}", candidate_repeat_size);
        //we are looking for consistent 'differences' between repeat size elements
        let mut first_repeat_differences: VecDeque<isize> =
            VecDeque::with_capacity(candidate_repeat_size);
        for i in 0..candidate_repeat_size {
            let later_index = end_repeat_index - i;
            let previous_index = end_repeat_index - candidate_repeat_size - i;
            let diff =
                second_order_differences[later_index] - second_order_differences[previous_index];
            first_repeat_differences.push_front(diff);
        }
        //now check that if we do comparisons of the previous differences, these match
        for check_num in 1..(NUM_REPEAT_CHECKS + 1) {
            let mut check_differences: VecDeque<isize> =
                VecDeque::with_capacity(candidate_repeat_size);
            for i in 0..candidate_repeat_size {
                let later_index = end_repeat_index - (check_num * candidate_repeat_size) - i;
                let previous_index =
                    end_repeat_index - ((check_num + 1) * candidate_repeat_size) - i;
                let diff = second_order_differences[later_index]
                    - second_order_differences[previous_index];
                check_differences.push_front(diff);
            }
            if check_differences != first_repeat_differences {
                // println!("Failed");
                continue 'outer;
            }
        }
        //found it...
        println!(
            "Found a repeat of size {}: {:?}",
            candidate_repeat_size, first_repeat_differences
        );
        return (
            candidate_repeat_size,
            first_repeat_differences.into_iter().collect(),
        );
    }
}

struct RepeatInfo {
    start_diff: isize,
    diff: isize,
}

fn create_repeat_infos(second_order_differences: &[isize], diffs: &[isize]) -> Vec<RepeatInfo> {
    second_order_differences
        .iter()
        .rev()
        .zip(diffs.iter().rev())
        .map(|(start_diff, diff)| RepeatInfo {
            start_diff: *start_diff,
            diff: *diff,
        })
        .rev()
        .collect()
}

fn calc_result_2_internal(values: Vec<isize>, num_required: usize) -> Result<FinalResult, AError> {
    //Need to find when the second order differences start to repeat
    let first_order_differences = get_differences(&values);
    let second_order_differences = get_differences(&first_order_differences);
    //find the repeat in the second order
    let (repeat_size, diffs) = calculate_repeat_size(&second_order_differences);
    let repeat_infos = create_repeat_infos(&second_order_differences, &diffs);
    //run through using the repeat values...
    let mut current_total = *values.last().unwrap();
    let mut current_diff = *first_order_differences.last().unwrap();
    for i in 0..(num_required - values.len()) {
        let num_multiplier = (i / repeat_size) + 1;
        let repeat_info_index = i % repeat_size;
        let repeat_info = repeat_infos.get(repeat_info_index).unwrap();
        current_diff += repeat_info.start_diff + repeat_info.diff * num_multiplier as isize;
        current_total += current_diff;
    }
    Ok(current_total as usize)
}

fn calc_result_2(state: ProcessedState2) -> Result<FinalResult, AError> {
    calc_result_2_internal(state.1, state.0)
}

fn main() {
    //let (total_steps, total_steps_2, total_to_calculate_2, file) = (6, 100, 5000, "test-input.txt");
    //let (total_steps, file) = ( "test-input2.txt");
    let (total_steps, total_steps_2, total_to_calculate_2, file) = (64, 500, 26501365, "input.txt");

    fn initial_state(total_steps: usize, total_to_calculate: usize) -> LoadingState {
        LoadingState {
            total_steps,
            total_to_calculate,
            start: None,
            tiles: CellsBuilder::new_empty(),
        }
    }

    let result1 = process(
        file,
        initial_state(total_steps, total_steps),
        parse_line,
        finalise_state,
        perform_processing,
        calc_result,
    );
    match result1 {
        Ok(res) => println!("Result 1: {:?}", res),
        Err(e) => println!("Error on 1: {}", e),
    }

    let result2 = process(
        file,
        initial_state(total_steps_2, total_to_calculate_2),
        parse_line,
        finalise_state,
        perform_processing_2,
        calc_result_2,
    );
    // Used to investigate the repeat finding code
    // let (mut values, num_required) = (load_values("test-input-values.txt"), 5000);
    // values.truncate(values.len() - 11);
    // let (values, num_required) = (load_values("input-values.txt"), 26501365);
    // let result2 = calc_result_2_internal(values, num_required);
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position_in_bound() {
        let side_lengths = (3usize, 3usize);
        assert_eq!(get_position_in_bounds(&side_lengths, 0, 0), (0, 0));
        assert_eq!(get_position_in_bounds(&side_lengths, 2, 2), (2, 2));
        assert_eq!(get_position_in_bounds(&side_lengths, 3, 3), (0, 0));
        assert_eq!(get_position_in_bounds(&side_lengths, 4, 4), (1, 1));
        assert_eq!(get_position_in_bounds(&side_lengths, 6, 6), (0, 0));
        assert_eq!(get_position_in_bounds(&side_lengths, -1, -1), (2, 2));
        assert_eq!(get_position_in_bounds(&side_lengths, -2, -2), (1, 1));
        assert_eq!(get_position_in_bounds(&side_lengths, -3, -3), (0, 0));
        assert_eq!(get_position_in_bounds(&side_lengths, -4, -4), (2, 2));
    }
}
