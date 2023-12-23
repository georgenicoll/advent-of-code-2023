use std::{fmt::Display, collections::{HashSet, HashMap, VecDeque}, mem::swap};

use anyhow::anyhow;
use processor::{process, CellsBuilder, Cells, adjacent_coords_cartesian};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    North,
    East,
    South,
    West,
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
            Tile::Slope { direction: Direction::North } => '^',
            Tile::Slope { direction: Direction::East } => '>',
            Tile::Slope { direction: Direction::South } => 'v',
            Tile::Slope { direction: Direction::West } => '<',
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
                '^' => Tile::Slope { direction: Direction::North },
                '>' => Tile::Slope { direction: Direction::East },
                'v' => Tile::Slope { direction: Direction::South },
                '<' => Tile::Slope { direction: Direction::West },
                _ => return Err(anyhow!(format!("Unrecognised tile: {c}")))
            };
            state.add_cell(tile)?;
        };
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
    visited_cells: HashSet<Coord>,
    current_position: Coord,
}

fn calculate_next_steps(cells: &Cells<Tile>, walk: &Walk, ending_point: &Coord, next_walks: &mut Vec<Walk>, finished_walks: &mut Vec<Walk>) {
    //if this is a slope we have to go in the direction of the slope
    let current_tile = cells.get(walk.current_position.0, walk.current_position.1).unwrap();
    let next_candidates = match current_tile {
        Tile::Path => adjacent_coords_cartesian(&walk.current_position, &cells.side_lengths),
        Tile::Slope { direction } => {
            let next_coord = match direction {
                Direction::North => (walk.current_position.0, walk.current_position.1 - 1),
                Direction::East => (walk.current_position.0 + 1, walk.current_position.1),
                Direction::South => (walk.current_position.0, walk.current_position.1 + 1),
                Direction::West => (walk.current_position.0 - 1, walk.current_position.1),
            };
            if cells.in_bounds(next_coord.0, next_coord.1) {
                vec![next_coord]
            } else {
                vec![]
            }
        }
        _ => vec![],
    };
    for next_candidate in next_candidates {
        if walk.visited_cells.contains(&next_candidate) {
            continue;  //Been there already
        };
        if next_candidate == *ending_point {
            //Done!
            let mut new_visited = walk.visited_cells.clone();
            new_visited.insert(next_candidate);
            finished_walks.push(Walk {
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
                    visited_cells: new_visited,
                    current_position: next_candidate,
                })
            }
        }
    }

}

fn do_walks(cells: &Cells<Tile>, starting_point: &Coord, ending_point: &Coord) -> Vec<Walk> {
    let mut current_walks: Vec<Walk> = Vec::default();
    let mut next_walks: Vec<Walk> = Vec::default();
    let mut finished_walks: Vec<Walk> = Vec::default();
    //Prime
    next_walks.push(Walk {
        visited_cells: HashSet::from([*starting_point]),
        current_position: *starting_point,
    });
    //Pump
    while !next_walks.is_empty() {
        swap(&mut current_walks, &mut next_walks);
        current_walks.iter().for_each(|walk| calculate_next_steps(
            cells,
            &walk,
            &ending_point,
            &mut next_walks,
            &mut finished_walks,
        ));
        current_walks.clear();
    }

    finished_walks
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let starting_point = (1, 0);
    let ending_point = (state.side_lengths.0 - 2, state.side_lengths.1 - 1);
    let walks = do_walks(&state, &starting_point, &ending_point);
    Ok(walks.iter()
        .map(|walk| {
            walk.visited_cells.iter()
                .filter(|coord| **coord != starting_point)
                .count()
        })
        .max()
        .unwrap()
    )
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

fn go_to_next(
    cells: &Cells<Tile>,
    visit: &Visit,
    visited: &mut HashMap<Visited, usize>, direction: Direction,
    to_visit: &mut VecDeque<Visit>
) {
    let next_coord = match direction {
        Direction::North => (visit.coord.0, visit.coord.1 - 1),
        Direction::East => (visit.coord.0 + 1, visit.coord.1),
        Direction::South => (visit.coord.0, visit.coord.1 + 1),
        Direction::West => (visit.coord.0 - 1, visit.coord.1),
    };
    if visit.visited.contains(&next_coord) {
        return;
    }
    if !cells.in_bounds(next_coord.0, next_coord.1) {
        return;
    }
    let tile = cells.get(next_coord.0, next_coord.1).unwrap();
    let next_and_new_visited = match tile {
        Tile::Forest => None,
        _ => {
            let new_visited = Visited {
                coord: next_coord,
                direction,
            };
            // //if we got here in the direction with more steps already.  don't bother in this direction again here
            // let current_visited = visited.get(&new_visited);
            // if current_visited.is_none() || *current_visited.unwrap() <= visit.steps + 1 {
                //less steps, try this direction
                Some((next_coord, new_visited))
            // } else {
            //     //Already got here in more steps going in this direction
            //     None
            // }
        }
    };
    if let Some((next, new_visited)) = next_and_new_visited {
        //follow this, push to the front
        let mut new_visit_visited = visit.visited.clone();
        new_visit_visited.insert(next);
        to_visit.push_front(Visit {
            coord: next,
            steps: visit.steps + 1,
            visited: new_visit_visited,
        });
        //record the steps here - if this is the furthest we got yet
        let current_visited = visited.get(&new_visited);
        if current_visited.is_none() || *current_visited.unwrap() < visit.steps + 1 {
            //more steps, record it
            visited.insert(new_visited, visit.steps + 1);
        }
    }
}

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
    while let Some(visit) = to_visit.pop_front() {
        go_to_next(&state, &visit, &mut visited, Direction::North, &mut to_visit);
        go_to_next(&state, &visit, &mut visited, Direction::East, &mut to_visit);
        go_to_next(&state, &visit, &mut visited, Direction::South, &mut to_visit);
        go_to_next(&state, &visit, &mut visited, Direction::West, &mut to_visit);
    }
    //get longest to end
    let steps = visited.get(&Visited {
        coord: ending_point,
        direction: Direction::South,
    }).expect("Didn't find end visit");
    Ok(*steps)
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
        CellsBuilder::new_empty(),
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
        CellsBuilder::new_empty(),
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
