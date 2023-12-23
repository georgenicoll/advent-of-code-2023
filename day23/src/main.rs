use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Display,
    mem::swap,
    time,
};

use anyhow::anyhow;
use processor::{process, Cells, CellsBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tile {
    Path,
    Forest,
    Slope { direction: Direction },
}

impl Tile {
    fn char_rep(&self) -> char {
        match self {
            Tile::Path => '.',
            Tile::Forest => '#',
            Tile::Slope {
                direction: Direction::North,
            } => '^',
            Tile::Slope {
                direction: Direction::East,
            } => '>',
            Tile::Slope {
                direction: Direction::South,
            } => 'v',
            Tile::Slope {
                direction: Direction::West,
            } => '<',
        }
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.char_rep())
    }
}

type AError = anyhow::Error;

type InitialState = CellsBuilder<Tile>;
type LoadedState = Cells<Tile>;
type ProcessedState = usize;
type FinalResult = ProcessedState;

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        state.new_line();
        for c in line.chars() {
            let tile = match c {
                '.' => Tile::Path,
                '#' => Tile::Forest,
                '^' => Tile::Slope {
                    direction: Direction::North,
                },
                '>' => Tile::Slope {
                    direction: Direction::East,
                },
                'v' => Tile::Slope {
                    direction: Direction::South,
                },
                '<' => Tile::Slope {
                    direction: Direction::West,
                },
                _ => return Err(anyhow!(format!("Unrecognised tile: {c}"))),
            };
            state.add_cell(tile)?;
        }
    }
    Ok(state)
}

fn output_cells(cells: &Cells<Tile>) {
    println!("Cells:");
    println!("{cells}");
    println!();
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    let cells = state.build_cells(Tile::Forest)?;
    output_cells(&cells);
    Ok(cells)
}

type Coord = (usize, usize);

struct Walk {
    steps: usize,
    visited_cells: HashSet<Coord>,
    current_position: Coord,
}

fn calculate_next_steps<F>(
    cells: &Cells<Tile>,
    choose_candidates: &F,
    walk: &Walk,
    ending_point: &Coord,
    next_walks: &mut Vec<Walk>,
    finished_walks: &mut Vec<Walk>,
) where
    F: Fn(&Coord, &Tile) -> Vec<(Coord, usize)>,
{
    //if this is a slope we have to go in the direction of the slope
    let current_tile = cells
        .get(walk.current_position.0, walk.current_position.1)
        .unwrap();
    let next_candidates = choose_candidates(&walk.current_position, current_tile);
    for (next_candidate, steps) in next_candidates {
        if walk.visited_cells.contains(&next_candidate) {
            continue; //Been there already
        };
        if next_candidate == *ending_point {
            //Done!
            let mut new_visited = walk.visited_cells.clone();
            new_visited.insert(next_candidate);
            finished_walks.push(Walk {
                steps: walk.steps + steps,
                visited_cells: new_visited,
                current_position: next_candidate,
            });
            continue;
        };
        let next_tile = cells.get(next_candidate.0, next_candidate.1).unwrap();
        match next_tile {
            Tile::Forest => (), //can't go here
            _ => {
                let mut new_visited = walk.visited_cells.clone();
                new_visited.insert(next_candidate);
                next_walks.push(Walk {
                    steps: walk.steps + steps,
                    visited_cells: new_visited,
                    current_position: next_candidate,
                })
            }
        }
    }
}

fn do_walks<F>(
    cells: &Cells<Tile>,
    starting_point: &Coord,
    ending_point: &Coord,
    choose_candidates: &F,
) -> Vec<Walk>
where
    F: Fn(&Coord, &Tile) -> Vec<(Coord, usize)>,
{
    let mut current_walks: Vec<Walk> = Vec::default();
    let mut next_walks: Vec<Walk> = Vec::default();
    let mut finished_walks: Vec<Walk> = Vec::default();
    //Prime
    next_walks.push(Walk {
        steps: 0,
        visited_cells: HashSet::from([*starting_point]),
        current_position: *starting_point,
    });
    //Pump
    while !next_walks.is_empty() {
        swap(&mut current_walks, &mut next_walks);
        current_walks.iter().for_each(|walk| {
            calculate_next_steps(
                cells,
                choose_candidates,
                walk,
                ending_point,
                &mut next_walks,
                &mut finished_walks,
            )
        });
        current_walks.clear();
    }

    finished_walks
}

fn adjacent_coords_and_directions(tiles: &Cells<Tile>, coord: &Coord) -> Vec<(Coord, Direction)> {
    [
        (
            get_next_coord(tiles, coord, &Direction::North),
            Direction::North,
        ),
        (
            get_next_coord(tiles, coord, &Direction::East),
            Direction::East,
        ),
        (
            get_next_coord(tiles, coord, &Direction::South),
            Direction::South,
        ),
        (
            get_next_coord(tiles, coord, &Direction::West),
            Direction::West,
        ),
    ]
    .iter()
    .filter_map(|(coord, direction)| coord.map(|coord| (coord, *direction)))
    .collect()
}

fn walk_to_end_of_corridor<F>(
    tiles: &Cells<Tile>,
    coord: &Coord,
    direction: &Direction,
    is_corridor_tile: &F,
) -> Option<(Coord, Direction, usize)>
where
    F: Fn(&Tile) -> bool,
{
    let mut next_coord = *coord;
    let mut last_direction = *direction;
    let mut steps = 1;
    while is_corridor(tiles, &next_coord, is_corridor_tile) {
        let next_and_direction = get_next_in_corridor(tiles, &next_coord, &last_direction);
        next_and_direction?;
        (next_coord, last_direction) = next_and_direction.unwrap();
        steps += 1;
    }
    Some((next_coord, last_direction, steps))
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let starting_point = (1, 0);
    let ending_point = (state.side_lengths.0 - 2, state.side_lengths.1 - 1);
    let walks = do_walks(&state, &starting_point, &ending_point, &|coord, tile| {
        let next_coords = match tile {
            Tile::Path => adjacent_coords_and_directions(&state, coord),
            Tile::Slope { direction } => {
                let next_coord_and_direction = match direction {
                    Direction::North => ((coord.0, coord.1 - 1), Direction::North),
                    Direction::East => ((coord.0 + 1, coord.1), Direction::East),
                    Direction::South => ((coord.0, coord.1 + 1), Direction::South),
                    Direction::West => ((coord.0 - 1, coord.1), Direction::West),
                };
                let (next_coord, _) = next_coord_and_direction;
                if state.in_bounds(next_coord.0, next_coord.1) {
                    vec![next_coord_and_direction]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        };
        next_coords
            .iter()
            .filter_map(|(coord, direction)| {
                walk_to_end_of_corridor(&state, coord, direction, &|tile| {
                    matches!(tile, Tile::Path)
                })
            })
            .map(|(coord, _, steps)| (coord, steps))
            .collect()
    });
    Ok(walks.iter().map(|walk| walk.steps).max().unwrap())
}

struct Visit {
    coord: Coord,
    steps: usize,
    visited: HashSet<Coord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Visited {
    coord: Coord,
    direction: Direction,
}

fn get_next_coord(cells: &Cells<Tile>, coord: &Coord, direction: &Direction) -> Option<Coord> {
    let (next_x, next_y) = match direction {
        Direction::North => (coord.0 as isize, coord.1 as isize - 1),
        Direction::East => (coord.0 as isize + 1, coord.1 as isize),
        Direction::South => (coord.0 as isize, coord.1 as isize + 1),
        Direction::West => (coord.0 as isize - 1, coord.1 as isize),
    };
    if !cells.in_bounds(next_x, next_y) {
        return None;
    }
    Some((next_x as usize, next_y as usize))
}

fn is_forest_or_edge(cells: &Cells<Tile>, coord: &Coord, delta_x: isize, delta_y: isize) -> bool {
    let (x, y) = (coord.0 as isize + delta_x, coord.1 as isize + delta_y);
    if !cells.in_bounds(x, y) {
        return true;
    }
    let (x, y) = (x as usize, y as usize);
    let tile = cells.get(x, y).unwrap();
    matches!(tile, Tile::Forest)
}

fn is_corridor<F>(cells: &Cells<Tile>, coord: &Coord, is_corridor_tile: &F) -> bool
where
    F: Fn(&Tile) -> bool,
{
    let mut count_walls = 0usize;
    //Firstly, This needs to be a Path
    let tile = cells.get(coord.0, coord.1).unwrap();
    if !is_corridor_tile(tile) {
        return false;
    }
    if is_forest_or_edge(cells, coord, 0, -1) {
        //above
        count_walls += 1;
    }
    if is_forest_or_edge(cells, coord, 0, 1) {
        //below
        count_walls += 1;
    }
    if is_forest_or_edge(cells, coord, 1, 0) {
        //right
        count_walls += 1;
    }
    if is_forest_or_edge(cells, coord, -1, 0) {
        //left
        count_walls += 1;
    }
    count_walls == 2
}

fn get_next_in_corridor(
    cells: &Cells<Tile>,
    coord: &Coord,
    direction: &Direction,
) -> Option<(Coord, Direction)> {
    let mut possible_direction: Vec<Direction> = Vec::default();
    if !is_forest_or_edge(cells, coord, 0, -1) {
        //above
        possible_direction.push(Direction::North);
    }
    if !is_forest_or_edge(cells, coord, 0, 1) {
        //below
        possible_direction.push(Direction::South);
    }
    if !is_forest_or_edge(cells, coord, 1, 0) {
        //right
        possible_direction.push(Direction::East);
    }
    if !is_forest_or_edge(cells, coord, -1, 0) {
        //left
        possible_direction.push(Direction::West);
    }
    let new_direction = possible_direction
        .iter()
        .find(|d| **d != direction.opposite())
        .expect("Expecting a direction");
    get_next_coord(cells, coord, new_direction).map(|coord| (coord, *new_direction))
}

fn go_to_next(
    cells: &Cells<Tile>,
    end_coord: &Coord,
    visit: &Visit,
    visited: &mut HashMap<Visited, usize>,
    direction: Direction,
    to_visit: &mut VecDeque<Visit>,
) {
    let next_coord = get_next_coord(cells, &visit.coord, &direction);
    if next_coord.is_none() {
        return;
    }
    let mut next_coord = next_coord.unwrap();
    if visit.visited.contains(&next_coord) {
        return;
    }
    let next_tile = cells.get(next_coord.0, next_coord.1).unwrap();
    if matches!(next_tile, Tile::Forest) {
        return;
    }
    let mut latest_direction = direction;
    let mut steps = 1;
    while is_corridor(cells, &next_coord, &|tile| !matches!(tile, Tile::Forest)) {
        //new_visit_visited.insert(next_coord);
        let next_and_direction = get_next_in_corridor(cells, &next_coord, &latest_direction);
        if next_and_direction.is_none() {
            return;
        }
        steps += 1;
        if next_coord == *end_coord {
            break;
        }
        (next_coord, latest_direction) = next_and_direction.unwrap();
    }
    if visit.visited.contains(&next_coord) {
        return;
    }
    let mut new_visit_visited = visit.visited.clone();
    new_visit_visited.insert(next_coord);
    let new_visited = Visited {
        coord: next_coord,
        direction: latest_direction,
    };
    // let max_and_coords = visited.get(&new_visited);
    // match max_and_coords {
    //     None => (), //keep going
    //     Some((max_steps_so_far, coords_so_far)) => {
    //         //already got here with the same coords?
    //         if new_visit_visited.is_subset(coords_so_far) && *max_steps_so_far < visit.steps + steps {
    //             //stop searching now
    //             return;
    //         } else {
    //             //this is more... don't stop
    //         }
    //     }
    // }
    to_visit.push_front(Visit {
        coord: next_coord,
        steps: visit.steps + steps,
        visited: new_visit_visited,
    });
    //Update the visited to this new max
    let new_max = visited
        .get(&new_visited)
        .copied()
        .unwrap_or(0)
        .max(visit.steps + steps);
    visited.insert(new_visited, new_max);
}

/// Original 'breadth first' search.  It needs a *lot* of memory but does get there
/// eventually, if it's available (~12G needed)
// fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
//     let starting_point = (1, 0);
//     let ending_point = (state.side_lengths.0 - 2, state.side_lengths.1 - 1);
//     let walks = do_walks(&state, &starting_point, &ending_point, &|coord, tile| {
//         let next_coords = match tile {
//             Tile::Path => adjacent_coords_and_directions(&state, coord),
//             Tile::Slope { direction: _direction } => adjacent_coords_and_directions(&state, coord),
//             _ => vec![],
//         };
//         next_coords.iter()
//             .filter_map(|(coord, direction)|
//                 walk_to_end_of_corridor(&state, coord, direction, &|tile|
//                     !matches!(tile, Tile::Forest)
//                 )
//             )
//             .map(|(coord, _, steps)| (coord, steps))
//             .collect()
//     });
//     Ok(walks
//         .iter()
//         .map(|walk| walk.steps)
//         .max()
//         .unwrap())
// }

/// Alternative Depth first search - requires much less memory but similar time require (still super slow -
/// takes ~10 mins on mini-pc)
fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let starting_point = (1, 0);
    let ending_point = (state.side_lengths.0 - 2, state.side_lengths.1 - 1);
    //need to do a depth first search...  ?dropping any where we got to the point in more from the same direction already
    let mut visited: HashMap<Visited, usize> = HashMap::default();
    let mut to_visit: VecDeque<Visit> = VecDeque::default();
    //Prime
    to_visit.push_front(Visit {
        coord: starting_point,
        steps: 0,
        visited: HashSet::from([starting_point]),
    });
    //Pump
    let mut last_reported = 0;
    while let Some(visit) = to_visit.pop_front() {
        let the_len = to_visit.len();
        if the_len % 10 == 0 && last_reported != the_len {
            // println!("to_visit: {}", to_visit.len());
            last_reported = the_len;
        }
        go_to_next(
            &state,
            &ending_point,
            &visit,
            &mut visited,
            Direction::North,
            &mut to_visit,
        );
        go_to_next(
            &state,
            &ending_point,
            &visit,
            &mut visited,
            Direction::East,
            &mut to_visit,
        );
        go_to_next(
            &state,
            &ending_point,
            &visit,
            &mut visited,
            Direction::South,
            &mut to_visit,
        );
        go_to_next(
            &state,
            &ending_point,
            &visit,
            &mut visited,
            Direction::West,
            &mut to_visit,
        );
    }
    //get longest to end
    let steps = visited
        .get(&Visited {
            coord: ending_point,
            direction: Direction::South,
        })
        .expect("Didn't find end visit");
    Ok(*steps)
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
        CellsBuilder::new_empty(),
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
        CellsBuilder::new_empty(),
        parse_line,
        finalise_state,
        perform_processing_2,
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
