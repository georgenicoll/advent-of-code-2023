use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

use processor::{process, Cells, CellsBuilder};

type AError = anyhow::Error;

#[derive(Debug, Clone, Copy, Default)]
enum Tile {
    #[default]
    Space,
    MirrorTopLeftBottomRight,
    MirrorBottomLeftTopRight,
    SplitterHorizontal,
    SplitterVertical,
}

impl Tile {
    fn char_rep(&self) -> char {
        match self {
            Tile::Space => '.',
            Tile::MirrorTopLeftBottomRight => '\\',
            Tile::MirrorBottomLeftTopRight => '/',
            Tile::SplitterHorizontal => '-',
            Tile::SplitterVertical => '|',
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char_rep())
    }
}

type InitialState = CellsBuilder<Tile>;
type LoadedState = Cells<Tile>;
type ProcessedState = usize;
type FinalResult = usize;

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        state.new_line();
        for c in line.chars() {
            let tile = match c {
                '.' => Tile::Space,
                '\\' => Tile::MirrorTopLeftBottomRight,
                '/' => Tile::MirrorBottomLeftTopRight,
                '-' => Tile::SplitterHorizontal,
                '|' => Tile::SplitterVertical,
                _ => panic!("Unrecognised tile: {c}"),
            };
            state.add_cell(tile)?;
        }
    }
    Ok(state)
}

fn output_cells(_cells: &Cells<Tile>) {
    // println!("Cells:");
    // println!("{cells}");
    // println!();
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    let cells = state.build_cells(Tile::Space)?;
    output_cells(&cells);
    Ok(cells)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum LightDirection {
    Up,
    Right,
    Down,
    Left,
}

type Coord = (usize, usize);
type ProcessingDirection = (Coord, LightDirection);

fn create_empty_light_directions(cells: &Cells<Tile>) -> Cells<HashSet<LightDirection>> {
    let mut directions = CellsBuilder::new_empty();
    for _y in 0..cells.side_lengths.1 {
        directions.new_line();
        for _x in 0..cells.side_lengths.0 {
            directions.add_cell(HashSet::default()).unwrap();
        }
    }

    directions.build_cells(HashSet::default()).unwrap()
}

fn get_next_direction(
    x: usize,
    y: usize,
    direction: LightDirection,
) -> ((isize, isize), LightDirection) {
    let x = x as isize;
    let y = y as isize;
    match direction {
        LightDirection::Up => ((x, y - 1), direction),
        LightDirection::Down => ((x, y + 1), direction),
        LightDirection::Left => ((x - 1, y), direction),
        LightDirection::Right => ((x + 1, y), direction),
    }
}

fn process_light_direction(
    tiles: &Cells<Tile>,
    directions: &mut Cells<HashSet<LightDirection>>,
    direction: &ProcessingDirection,
) -> Vec<ProcessingDirection> {
    let ((x, y), direction) = direction;
    let tile = tiles.get(*x, *y).unwrap();
    let next_directions: Vec<((isize, isize), LightDirection)> = match (tile, direction) {
        (Tile::MirrorTopLeftBottomRight, LightDirection::Up) => {
            vec![get_next_direction(*x, *y, LightDirection::Left)]
        }
        (Tile::MirrorTopLeftBottomRight, LightDirection::Down) => {
            vec![get_next_direction(*x, *y, LightDirection::Right)]
        }
        (Tile::MirrorTopLeftBottomRight, LightDirection::Left) => {
            vec![get_next_direction(*x, *y, LightDirection::Up)]
        }
        (Tile::MirrorTopLeftBottomRight, LightDirection::Right) => {
            vec![get_next_direction(*x, *y, LightDirection::Down)]
        }
        (Tile::MirrorBottomLeftTopRight, LightDirection::Up) => {
            vec![get_next_direction(*x, *y, LightDirection::Right)]
        }
        (Tile::MirrorBottomLeftTopRight, LightDirection::Down) => {
            vec![get_next_direction(*x, *y, LightDirection::Left)]
        }
        (Tile::MirrorBottomLeftTopRight, LightDirection::Left) => {
            vec![get_next_direction(*x, *y, LightDirection::Down)]
        }
        (Tile::MirrorBottomLeftTopRight, LightDirection::Right) => {
            vec![get_next_direction(*x, *y, LightDirection::Up)]
        }
        (Tile::SplitterHorizontal, LightDirection::Up)
        | (Tile::SplitterHorizontal, LightDirection::Down) => vec![
            get_next_direction(*x, *y, LightDirection::Left),
            get_next_direction(*x, *y, LightDirection::Right),
        ],
        (Tile::SplitterVertical, LightDirection::Left)
        | (Tile::SplitterVertical, LightDirection::Right) => vec![
            get_next_direction(*x, *y, LightDirection::Up),
            get_next_direction(*x, *y, LightDirection::Down),
        ],
        _ => vec![get_next_direction(*x, *y, *direction)],
    };
    //only keep directions that are in bounds and we didn't already process
    let next_directions: Vec<ProcessingDirection> = next_directions
        .into_iter()
        .filter_map(|candidate| {
            let ((x, y), direction) = candidate;
            if !directions.in_bounds(x, y) {
                return None; //off the cells
            };
            let x = x as usize;
            let y = y as usize;
            let dirs = directions.get(x, y).unwrap();
            if dirs.contains(&direction) {
                return None; //already processed
            };
            Some(((x, y), direction))
        })
        .collect();
    //mark the cells as visited...
    next_directions.iter().for_each(|dir| {
        let ((x, y), direction) = dir;
        directions.get_mut(*x, *y).unwrap().insert(*direction);
    });
    next_directions
}

fn number_of_energised_tiles(directions: &Cells<HashSet<LightDirection>>) -> usize {
    directions
        .iter()
        .map(|((_x, _y), directions)| if directions.is_empty() { 0 } else { 1 })
        .sum()
}

fn process_from(
    tiles: &Cells<Tile>,
    start_x: usize,
    start_y: usize,
    start_direction: LightDirection,
) -> usize {
    let mut directions = create_empty_light_directions(tiles);
    let mut current_processing_directions: VecDeque<ProcessingDirection> = VecDeque::default();
    //Prime - beam enters start x, y heading in the start direction
    current_processing_directions.push_back(((start_x, start_y), start_direction));
    directions
        .get_mut(start_x, start_y)
        .unwrap()
        .insert(start_direction);

    //process until we have no more beam locations to process
    while let Some(direction) = current_processing_directions.pop_front() {
        let mut new_directions = process_light_direction(tiles, &mut directions, &direction);
        new_directions
            .drain(..)
            .for_each(|dir| current_processing_directions.push_back(dir));
    }
    //calculate how many tiles
    number_of_energised_tiles(&directions)
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    Ok(process_from(&state, 0, 0, LightDirection::Right))
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let left = (0..state.side_lengths.1).map(|y| process_from(&state, 0, y, LightDirection::Right));
    let top = (0..state.side_lengths.0).map(|x| process_from(&state, x, 0, LightDirection::Down));
    let right = (0..state.side_lengths.1)
        .map(|y| process_from(&state, state.side_lengths.0 - 1, y, LightDirection::Left));
    let bottom = (0..state.side_lengths.0)
        .map(|x| process_from(&state, x, state.side_lengths.1 - 1, LightDirection::Up));
    let result = left.chain(top).chain(right).chain(bottom).max();
    Ok(result.unwrap())
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
        CellsBuilder::default(),
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
        CellsBuilder::default(),
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
