use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

use once_cell::sync::Lazy;
use processor::{process, read_next, read_word, Cells};
use substring::Substring;

type AError = anyhow::Error;

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct DigInstruction {
    direction: Direction,
    steps: usize,
    hex_code: String,
}

type Coord = (usize, usize);
type SideLengths = (usize, usize);
type InitialState = (Coord, Vec<DigInstruction>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Tile {
    Space,
    Trench { hex_code: String },
}

impl Tile {
    fn char_rep(&self) -> char {
        match self {
            Tile::Space => '.',
            Tile::Trench {
                hex_code: _hex_code,
            } => '#',
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char_rep())
    }
}

struct LoadedState1 {
    inside_tile: Coord,
    dig_instructions: Vec<DigInstruction>,
    area: Cells<Tile>,
}

type ProcessedState = usize;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ', '(', ')']));

fn parse_line_1(state: InitialState, line: String) -> Result<InitialState, AError> {
    let (inside_tile, mut dig_instructions) = state;
    let mut chars = line.chars();
    if let Some(c) = chars.next() {
        let direction = match c {
            'U' => Direction::Up,
            'D' => Direction::Down,
            'L' => Direction::Left,
            'R' => Direction::Right,
            _ => panic!("Unrecognised direction in {line}"),
        };
        let (steps, _) = read_next::<usize>(&mut chars, &DELIMITERS)?;
        if let Some((hex_code, _)) = read_word(&mut chars, &DELIMITERS) {
            dig_instructions.push(DigInstruction {
                direction,
                steps,
                hex_code,
            });
        } else {
            panic!("Couldn't read the hex code");
        }
    };
    Ok((inside_tile, dig_instructions))
}

fn get_deltas(direction: &Direction) -> (isize, isize) {
    match direction {
        Direction::Up => (0isize, -1isize),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
    }
}

fn calculate_tile_area_bounds(dig_instructions: &Vec<DigInstruction>) -> (Coord, SideLengths) {
    let mut x = 0isize;
    let mut y = 0isize;
    let mut max_x = 0isize;
    let mut max_y = 0isize;
    let mut min_x = 0isize;
    let mut min_y = 0isize;
    for instruction in dig_instructions {
        let (delta_x, delta_y) = get_deltas(&instruction.direction);
        x += delta_x * instruction.steps as isize;
        y += delta_y * instruction.steps as isize;
        max_x = max_x.max(x);
        max_y = max_y.max(y);
        min_x = min_x.min(x);
        min_y = min_y.min(y);
    }
    println!(
        "calculated: min ({},{}) and max({}, {})",
        min_x, min_y, max_x, max_y
    );
    let side_lengths = ((max_x - min_x + 1) as usize, (max_y - min_y + 1) as usize);
    let start = (-min_x as usize, -min_y as usize);
    println!(
        "adjusted: start {:?} with side lengths {:?}",
        start, side_lengths
    );
    (start, side_lengths)
}

fn dig(
    area: &mut Cells<Tile>,
    instruction: &DigInstruction,
    current_x: usize,
    current_y: usize,
) -> (usize, usize) {
    let (mut x, mut y) = (current_x as isize, current_y as isize);
    let (delta_x, delta_y) = get_deltas(&instruction.direction);
    for _i in 0..instruction.steps {
        x += delta_x;
        y += delta_y;
        *area.get_mut(x as usize, y as usize).unwrap() = Tile::Trench {
            hex_code: instruction.hex_code.clone(),
        };
    }
    (x as usize, y as usize)
}

fn finalise_state_1(state: InitialState) -> Result<LoadedState1, AError> {
    let (inside_tile, dig_instructions) = state;
    //work out how big this needs to be and where we need to start and finish
    let (start, side_lenths) = calculate_tile_area_bounds(&dig_instructions);
    //Dig out the steps - just make a great big area
    let mut area = Cells::with_dimension(side_lenths.0, side_lenths.1, Tile::Space);
    //First Cell is a hole
    if let Some(instruction) = dig_instructions.first() {
        *area.get_mut(start.0, start.1).unwrap() = Tile::Trench {
            hex_code: instruction.hex_code.clone(),
        }
    }
    //Now dig the rest
    let (_current_x, _current_y) = dig_instructions
        .iter()
        .fold((start.0, start.1), |(current_x, current_y), instruction| {
            dig(&mut area, instruction, current_x, current_y)
        });
    // println!("Area:");
    // println!("{area}");
    Ok(LoadedState1 {
        inside_tile,
        dig_instructions,
        area,
    })
}

fn add_next(
    area: &Cells<Tile>,
    visited: &HashSet<Coord>,
    next: &mut VecDeque<Coord>,
    candidate: (usize, usize),
) {
    let tile = area.get(candidate.0, candidate.1).unwrap();
    if *tile == Tile::Space && !visited.contains(&candidate) {
        next.push_back(candidate);
    };
}

fn perform_processing_1(state: LoadedState1) -> Result<ProcessedState, AError> {
    //Calculate the area that is enclosed
    let mut next: VecDeque<Coord> = VecDeque::default();
    let mut visited: HashSet<Coord> = HashSet::default();
    //Prime
    next.push_back(state.inside_tile);
    //Process
    while let Some(tile_coord) = next.pop_front() {
        if !visited.insert(tile_coord) {
            continue;
        }
        let (tile_x, tile_y) = tile_coord;
        add_next(&state.area, &visited, &mut next, (tile_x, tile_y - 1)); //Up
        add_next(&state.area, &visited, &mut next, (tile_x, tile_y + 1)); //Down
        add_next(&state.area, &visited, &mut next, (tile_x - 1, tile_y)); //Left
        add_next(&state.area, &visited, &mut next, (tile_x + 1, tile_y)); //Right
    }
    //calculate area of the initial trench
    let trench_area: usize = state.dig_instructions.iter().map(|i| i.steps).sum();

    Ok(visited.len() + trench_area)
}

struct LoadedState2 {
    dig_instructions: Vec<DigInstruction>,
    points: Vec<(isize, isize)>,
}

fn parse_line_2(state: InitialState, line: String) -> Result<InitialState, AError> {
    let (inside_tile, mut dig_instructions) = state;
    let mut chars = line.chars();
    if let Some(_c) = chars.next() {
        //ignore first letter and number
        let (_ignore, _) = read_next::<usize>(&mut chars, &DELIMITERS)?;
        if let Some((encoded_instruction, _)) = read_word(&mut chars, &DELIMITERS) {
            let hex_steps = encoded_instruction.substring(1, 6);
            let steps = usize::from_str_radix(hex_steps, 16).map_err(AError::from)?;
            let direction = match encoded_instruction.substring(6, 7) {
                "0" => Direction::Right,
                "1" => Direction::Down,
                "2" => Direction::Left,
                "3" => Direction::Up,
                _ => panic!("Unrecognised direction in {}", encoded_instruction),
            };
            dig_instructions.push(DigInstruction {
                direction,
                steps,
                hex_code: encoded_instruction,
            });
        } else {
            panic!("Failed to read encoded instruction")
        }
    };
    Ok((inside_tile, dig_instructions))
}

fn finalise_state_2(state: InitialState) -> Result<LoadedState2, AError> {
    let (_inside_tile, dig_instructions) = state;
    let (_next, points) = dig_instructions.iter().fold(
        ((0, 0), Vec::from([(0, 0)])),
        |((last_x, last_y), mut points), instruction| {
            let (delta_x, delta_y) = get_deltas(&instruction.direction);
            let next = (
                last_x + (delta_x * instruction.steps as isize),
                last_y + (delta_y * instruction.steps as isize),
            );
            points.push(next);
            (next, points)
        },
    );
    Ok(LoadedState2 {
        dig_instructions,
        points,
    })
}

fn perform_processing_2(state: LoadedState2) -> Result<ProcessedState, AError> {
    //Using the shoelace formula: https://en.wikipedia.org/wiki/Shoelace_formula
    //adapted from C++ here: https://www.geeksforgeeks.org/area-of-a-polygon-with-given-n-ordered-vertices/
    let initial_j_coord = *state.points.last().unwrap();
    let (_, area) = state.points.iter().fold(
        (initial_j_coord, 0isize),
        |((j_x, j_y), area), (i_x, i_y)| ((*i_x, *i_y), area + (j_x + *i_x) * (j_y - *i_y)),
    );
    let enclosed_area = (area / 2).unsigned_abs();
    //Plus the trench.  Since we measured the area above from the centres of all of the outside trench, we can take half o the number of trench
    //tiles plus 1 to account for the unbalanced outside corners
    let trench_area = state
        .dig_instructions
        .iter()
        .map(|i| i.steps)
        .sum::<usize>()
        / 2
        + 1;
    Ok(enclosed_area + trench_area)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

fn main() {
    //let (inside_tile, file) = ((1,1), "test-input.txt");
    //let (inside_tile, file) = "test-input2.txt";
    let (inside_tile, file) = ((359, 1), "input.txt");

    let result1 = process(
        file,
        (inside_tile, Vec::default()),
        parse_line_1,
        finalise_state_1,
        perform_processing_1,
        calc_result,
    );
    match result1 {
        Ok(res) => println!("Result 1: {:?}", res),
        Err(e) => println!("Error on 1: {}", e),
    }

    let result2 = process(
        file,
        (inside_tile, Vec::default()),
        parse_line_2,
        finalise_state_2,
        perform_processing_2,
        calc_result,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}
