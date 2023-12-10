use std::{
    collections::{BTreeSet, HashSet, VecDeque},
    fmt::Display,
};

use once_cell::sync::Lazy;
use processor::{process, Cells, CellsBuilder};
use strum_macros::EnumIter;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Pipe {
    Vertical,
    Horizontal,
    NorthToEast,
    NorthToWest,
    SouthToWest,
    SouthToEast,
    Ground,
    Start,
}

impl Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = match self {
            Pipe::Vertical => '|',
            Pipe::Horizontal => '-',
            Pipe::NorthToEast => 'L',
            Pipe::NorthToWest => 'J',
            Pipe::SouthToWest => '7',
            Pipe::SouthToEast => 'F',
            Pipe::Ground => '.',
            Pipe::Start => 'S',
        };
        write!(f, "{c}")
    }
}

type Coord = (usize, usize);

#[derive(Debug)]
struct LoadingState {
    start: Option<Coord>,
    start_pipe: Pipe,
    pipes: CellsBuilder<Pipe>,
}

struct State {
    start: Coord,
    pipes: Cells<Pipe>,
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "start: {}, {}", self.start.0, self.start.1)?;
        let mut current_y = 0;
        for ((_, y), pipe) in self.pipes.iter() {
            if y != current_y {
                writeln!(f)?;
                current_y = y;
            }
            write!(f, "{pipe}")?;
        }
        writeln!(f)
    }
}

#[derive(Debug, Clone)]
enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Debug, Clone, EnumIter, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum PipeRunDirection {
    North,
    East,
    South,
    West,
    NorthEast,
    SouthEast,
    SouthWest,
    NorthWest,
}

#[derive(Debug, Clone)]
struct PipeRun {
    on_the_edge: bool,
    available_directions: BTreeSet<PipeRunDirection>,
    surrounding_ground: Vec<Coord>,
}

type AError = anyhow::Error;
type InitialState = LoadingState;
type LoadedState = State;
type ProcessedState = usize;
type FinalResult = usize;

fn add_next_pipe(c: char, pipes: &mut CellsBuilder<Pipe>) -> Result<bool, AError> {
    let (is_start, pipe) = match c {
        '|' => (false, Pipe::Vertical),
        '-' => (false, Pipe::Horizontal),
        'L' => (false, Pipe::NorthToEast),
        'J' => (false, Pipe::NorthToWest),
        '7' => (false, Pipe::SouthToWest),
        'F' => (false, Pipe::SouthToEast),
        '.' | 'O' | 'I' => (false, Pipe::Ground),
        'S' => (true, Pipe::Start),
        _ => panic!("Unrecognised pipe: {}", c),
    };
    pipes.add_cell(pipe)?;
    Ok(is_start)
}

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    state.pipes.new_line();
    state.start = line.chars().fold(state.start, |start, c| {
        let was_start = add_next_pipe(c, &mut state.pipes).ok()?;
        if was_start {
            let coords = state.pipes.current_cell().unwrap();
            Some(coords)
        } else {
            start
        }
    });
    Ok(state)
}

//Return bools for exists at north east south west
fn get_exits(pipe: &Pipe) -> (bool, bool, bool, bool) {
    match pipe {
        Pipe::Vertical => (true, false, true, false),
        Pipe::Horizontal => (false, true, false, true),
        Pipe::NorthToEast => (true, true, false, false),
        Pipe::NorthToWest => (true, false, false, true),
        Pipe::SouthToWest => (false, false, true, true),
        Pipe::SouthToEast => (false, true, true, false),
        Pipe::Ground => (false, false, false, false),
        Pipe::Start => (false, false, false, false),
    }
}

fn replace_start_pipe(start: &(usize, usize), state: &mut InitialState) {
    let start = state.pipes.get_mut(start.0, start.1).unwrap();
    *start = state.start_pipe.clone();
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    let start = state.start.ok_or(anyhow::anyhow!("No start found"))?;
    replace_start_pipe(&start, &mut state);
    Ok(LoadedState {
        start,
        pipes: state.pipes.build_cells(Pipe::Ground)?,
    })
}

fn get_next_x_y_and_direction(
    x: usize,
    y: usize,
    pipe: &Pipe,
    direction: &Direction,
) -> (usize, usize, Direction) {
    match (pipe, direction) {
        (Pipe::Vertical, Direction::North) => (x, y - 1, Direction::North),
        (Pipe::Vertical, Direction::South) => (x, y + 1, Direction::South),
        (Pipe::Horizontal, Direction::East) => (x + 1, y, Direction::East),
        (Pipe::Horizontal, Direction::West) => (x - 1, y, Direction::West),
        (Pipe::NorthToEast, Direction::South) => (x + 1, y, Direction::East),
        (Pipe::NorthToEast, Direction::West) => (x, y - 1, Direction::North),
        (Pipe::NorthToWest, Direction::South) => (x - 1, y, Direction::West),
        (Pipe::NorthToWest, Direction::East) => (x, y - 1, Direction::North),
        (Pipe::SouthToWest, Direction::North) => (x - 1, y, Direction::West),
        (Pipe::SouthToWest, Direction::East) => (x, y + 1, Direction::South),
        (Pipe::SouthToEast, Direction::North) => (x + 1, y, Direction::East),
        (Pipe::SouthToEast, Direction::West) => (x, y + 1, Direction::South),
        _ => panic!("Unrecognised pipe/direction: {}, {:?}", pipe, direction),
    }
}

fn get_loop_tiles(state: &LoadedState) -> Result<HashSet<(usize, usize)>, AError> {
    //decide which direction to go initially
    let (start_x, start_y) = state.start;
    let pipe = state.pipes.get(start_x, start_y)?;
    let (n, e, s, w) = get_exits(pipe);
    let (mut x, mut y, mut direction) = match (n, e, s, w) {
        (true, _, _, _) => (start_x, start_y - 1, Direction::North),
        (_, _, true, _) => (start_x, start_y + 1, Direction::South),
        (_, true, _, _) => (start_x + 1, start_y, Direction::East),
        (_, _, _, true) => (start_x - 1, start_y, Direction::West),
        _ => panic!("Can't get current direction"),
    };
    let mut loop_tiles = HashSet::from([(start_x, start_y)]);
    while !loop_tiles.contains(&(x, y)) {
        loop_tiles.insert((x, y));
        let pipe = state.pipes.get(x, y)?;
        (x, y, direction) = get_next_x_y_and_direction(x, y, pipe, &direction);
    }
    Ok(loop_tiles)
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let loop_tiles = get_loop_tiles(&state)?;
    Ok(loop_tiles.len() / 2)
}

fn get_pipe_at<'a>(
    pipe_cells: &'a Cells<Pipe>,
    x: usize,
    y: usize,
    delta_x: i64,
    delta_y: i64,
    loop_tiles: &HashSet<Coord>,
) -> Option<(&'a Pipe, Coord)> {
    let (adjusted_x, adjusted_y) = (x as i64 + delta_x, y as i64 + delta_y);
    if adjusted_x < 0 || adjusted_y < 0 {
        return None;
    }
    let (adjusted_x, adjusted_y) = (adjusted_x as usize, adjusted_y as usize);
    let (side_x, side_y) = pipe_cells.side_lengths;
    if adjusted_x >= side_x || adjusted_y >= side_y {
        return None;
    }
    let adjusted = (adjusted_x, adjusted_y);
    if loop_tiles.contains(&adjusted) {
        Some((pipe_cells.get(adjusted_x, adjusted_y).unwrap(), adjusted))
    } else {
        //not a loop tile, treat it as though it is a ground tile - we should search as though it it
        Some((&Pipe::Ground, adjusted))
    }
}

//Return bools for allowed directions (north east south west)
fn get_exits_opt(pipe: Option<&Pipe>) -> (bool, bool, bool, bool) {
    match pipe {
        Some(pipe) => get_exits(pipe),
        None => (false, false, false, false),
    }
}

static NORTH_WEST_PIPES: Lazy<HashSet<Pipe>> =
    Lazy::new(|| HashSet::from([Pipe::NorthToEast, Pipe::SouthToWest]));
static NORTH_EAST_PIPES: Lazy<HashSet<Pipe>> =
    Lazy::new(|| HashSet::from([Pipe::NorthToWest, Pipe::SouthToEast]));
static SOUTH_WEST_PIPES: Lazy<HashSet<Pipe>> =
    Lazy::new(|| HashSet::from([Pipe::NorthToWest, Pipe::SouthToEast]));
static SOUTH_EAST_PIPES: Lazy<HashSet<Pipe>> =
    Lazy::new(|| HashSet::from([Pipe::NorthToEast, Pipe::SouthToWest]));

fn create_pipe_run(
    pipe_cells: &Cells<Pipe>,
    x: usize,
    y: usize,
    loop_tiles: &HashSet<Coord>,
) -> Result<PipeRun, AError> {
    //Need to look around the crossing point to see where we can go - remember we are 'in-between' the squares
    let pipe_nw = get_pipe_at(pipe_cells, x, y, -1, -1, loop_tiles);
    let pipe_ne = get_pipe_at(pipe_cells, x, y, 0, -1, loop_tiles);
    let pipe_sw = get_pipe_at(pipe_cells, x, y, -1, 0, loop_tiles);
    let pipe_se = get_pipe_at(pipe_cells, x, y, 0, 0, loop_tiles);

    let mut directions: HashSet<PipeRunDirection> = HashSet::default();

    let pipe_nw_exits = get_exits_opt(pipe_nw.map(|it| it.0));
    let pipe_ne_exits = get_exits_opt(pipe_ne.map(|it| it.0));
    let pipe_sw_exits = get_exits_opt(pipe_sw.map(|it| it.0));
    let pipe_se_exits = get_exits_opt(pipe_se.map(|it| it.0));

    //North
    if (pipe_nw.is_some() || pipe_ne.is_some())
        && (pipe_nw.is_none() || !pipe_nw_exits.1)
        && (pipe_ne.is_none() || !pipe_ne_exits.3)
    {
        directions.insert(PipeRunDirection::North);
    }
    //East
    if (pipe_ne.is_some() || pipe_se.is_some())
        && (pipe_ne.is_none() || !pipe_ne_exits.2)
        && (pipe_se.is_none() || !pipe_se_exits.0)
    {
        directions.insert(PipeRunDirection::East);
    }
    //South
    if (pipe_sw.is_some() || pipe_se.is_some())
        && (pipe_sw.is_none() || !pipe_sw_exits.1)
        && (pipe_se.is_none() || !pipe_se_exits.3)
    {
        directions.insert(PipeRunDirection::South);
    }
    //West
    if (pipe_nw.is_some() || pipe_sw.is_some())
        && (pipe_nw.is_none() || !pipe_nw_exits.2)
        && (pipe_sw.is_none() || !pipe_sw_exits.0)
    {
        directions.insert(PipeRunDirection::West);
    }
    //NorthEast
    if let Some((pipe, _)) = pipe_ne {
        if NORTH_EAST_PIPES.contains(pipe) {
            directions.insert(PipeRunDirection::NorthEast);
        }
    }
    //NorthWest
    if let Some((pipe, _)) = pipe_nw {
        if NORTH_WEST_PIPES.contains(pipe) {
            directions.insert(PipeRunDirection::NorthWest);
        }
    }
    //SouthWest
    if let Some((pipe, _)) = pipe_sw {
        if SOUTH_WEST_PIPES.contains(pipe) {
            directions.insert(PipeRunDirection::SouthWest);
        }
    }
    //SouthEast
    if let Some((pipe, _)) = pipe_se {
        if SOUTH_EAST_PIPES.contains(pipe) {
            directions.insert(PipeRunDirection::SouthEast);
        }
    }

    let surrounding_ground = [pipe_ne, pipe_nw, pipe_se, pipe_sw]
        .iter()
        .filter(|pipe_and_coords|
            matches!(pipe_and_coords, Some((_, coords)) if !loop_tiles.contains(coords))
        )
        .map(|it| it.unwrap().1)
        .collect::<Vec<Coord>>();

    Ok(PipeRun {
        on_the_edge: pipe_ne.is_none()
            || pipe_nw.is_none()
            || pipe_se.is_none()
            || pipe_sw.is_none(),
        available_directions: BTreeSet::from_iter(directions),
        surrounding_ground,
    })
}

fn create_pipe_runs(
    pipe_cells: &Cells<Pipe>,
    loop_tiles: &HashSet<Coord>,
) -> Result<Cells<PipeRun>, AError> {
    let mut builder = CellsBuilder::new_empty();
    let (side_x, side_y) = pipe_cells.side_lengths;
    let (new_side_x, new_side_y) = (side_x + 1, side_y + 1);
    for y in 0..new_side_y {
        builder.new_line();
        for x in 0..new_side_x {
            builder.add_cell(create_pipe_run(pipe_cells, x, y, loop_tiles)?)?;
        }
    }
    builder.build_cells(PipeRun {
        on_the_edge: false,
        available_directions: BTreeSet::default(),
        surrounding_ground: Vec::default(),
    })
}

fn get_next_coord(coord: &Coord, direction: &PipeRunDirection, extents: &Coord) -> Option<Coord> {
    let (adjusted_x, adjusted_y) = match direction {
        PipeRunDirection::North => (coord.0 as i64, coord.1 as i64 - 1),
        PipeRunDirection::East => (coord.0 as i64 + 1, coord.1 as i64),
        PipeRunDirection::South => (coord.0 as i64, coord.1 as i64 + 1),
        PipeRunDirection::West => (coord.0 as i64 - 1, coord.1 as i64),
        PipeRunDirection::NorthEast => (coord.0 as i64 + 1, coord.1 as i64 - 1),
        PipeRunDirection::SouthEast => (coord.0 as i64 + 1, coord.1 as i64 + 1),
        PipeRunDirection::SouthWest => (coord.0 as i64 - 1, coord.1 as i64 + 1),
        PipeRunDirection::NorthWest => (coord.0 as i64 - 1, coord.1 as i64 - 1),
    };
    if adjusted_x < 0 || adjusted_y < 0 {
        return None;
    }
    let (adjusted_x, adjusted_y) = (adjusted_x as usize, adjusted_y as usize);
    if adjusted_x >= extents.0 || adjusted_y >= extents.1 {
        return None;
    }
    Some((adjusted_x, adjusted_y))
}

fn find_all_connected_ground_tiles(
    starting_at: &Coord,
    pipe_runs: &Cells<PipeRun>,
) -> (bool, HashSet<Coord>) {
    let mut visited_coords: HashSet<Coord> = HashSet::default();
    let mut to_process: VecDeque<Coord> = VecDeque::from([*starting_at]);
    visited_coords.insert(*starting_at);
    let mut ground_tiles: HashSet<Coord> = HashSet::default();
    let mut got_outside = false;

    while let Some(coord) = to_process.pop_front() {
        let (x, y) = coord;
        if let Ok(pipe_run) = pipe_runs.get(x, y) {
            for direction in pipe_run.available_directions.iter() {
                if let Some(new_coord) = get_next_coord(&coord, direction, &pipe_runs.side_lengths)
                {
                    if !visited_coords.contains(&new_coord) {
                        to_process.push_back(new_coord);
                        visited_coords.insert(new_coord);
                    }
                }
            }

            for tile_coord in pipe_run.surrounding_ground.iter() {
                ground_tiles.insert(*tile_coord);
            }
            got_outside = got_outside || pipe_run.on_the_edge;
        }
    }

    (got_outside, ground_tiles)
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let loop_tiles = get_loop_tiles(&state)?;
    let pipe_runs = create_pipe_runs(&state.pipes, &loop_tiles)?;

    let mut visited_tiles: HashSet<(usize, usize)> = HashSet::default();
    let mut inside_tiles: BTreeSet<Coord> = BTreeSet::default();

    for (coord, pipe) in state.pipes.iter() {
        if loop_tiles.contains(&coord) || visited_tiles.contains(&coord) {
            continue;
        }
        match pipe {
            Pipe::Ground => {
                let (can_get_outside, mut tile_coords) =
                    find_all_connected_ground_tiles(&coord, &pipe_runs);
                // output_tiles(can_get_outside, &tile_coords);
                if !can_get_outside {
                    for tile in tile_coords.iter() {
                        inside_tiles.insert(*tile);
                    }
                }
                for visited in tile_coords.drain() {
                    visited_tiles.insert(visited);
                }
            }
            _ => {
                visited_tiles.insert(coord);
            }
        }
    }
    Ok(inside_tiles.len())
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

fn main() {
    //let input = (Pipe::SouthToEast, "test-input.txt");
    //let input = (Pipe::SouthToEast, "test-input2.txt");
    //let input = (Pipe::SouthToEast, "test-input3.txt");
    //let input = (Pipe::SouthToWest, "test-input4.txt");
    let input = (Pipe::Vertical, "input.txt");

    let result1 = process(
        input.1,
        LoadingState {
            start: None,
            start_pipe: input.0.clone(),
            pipes: CellsBuilder::new_empty(),
        },
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
        input.1,
        LoadingState {
            start: None,
            start_pipe: input.0,
            pipes: CellsBuilder::new_empty(),
        },
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
