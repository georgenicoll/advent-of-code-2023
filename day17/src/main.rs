use std::{
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use processor::{process, Cells, CellsBuilder};

type AError = anyhow::Error;

#[derive(Debug, Clone, Copy, Default)]
struct HeatLoss {
    amount: usize,
}

impl Display for HeatLoss {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.amount)
    }
}

type InitialState = CellsBuilder<HeatLoss>;
type LoadedState = Cells<HeatLoss>;
type ProcessedState = usize;
type FinalResult = usize;

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        state.new_line();
        line.chars().for_each(|c| {
            if let Some(heat_loss) = c.to_digit(10) {
                state
                    .add_cell(HeatLoss {
                        amount: heat_loss as usize,
                    })
                    .unwrap();
            } else {
                panic!("Non-number {} in line: {}", c, line);
            }
        })
    }
    Ok(state)
}

fn output_heat_loss_grid(_grid: &Cells<HeatLoss>) {
    // println!("Grid:");
    // println!("{grid}");
    // println!("")
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    let grid = state.build_cells(HeatLoss::default())?;
    output_heat_loss_grid(&grid);
    Ok(grid)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
struct Move {
    x: usize,
    y: usize,
    direction: Direction,
    cost: usize,
    turn_last_made: usize,
}

impl Move {
    fn new(x: usize, y: usize, direction: Direction, cost: usize, turn_last_made: usize) -> Move {
        Move {
            x,
            y,
            direction,
            cost,
            turn_last_made,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BestSoFarKey {
    direction: Direction,
    turn_last_made: usize,
}

impl BestSoFarKey {
    fn new(direction: Direction, turn_last_made: usize) -> BestSoFarKey {
        BestSoFarKey {
            direction,
            turn_last_made,
        }
    }
}

struct CrucibleParameters {
    min_in_straight_line: usize,
    max_in_straight_line: usize,
}

#[inline]
fn can_move_required_in_a_straight_line(
    x_y_direction: (isize, isize, Direction),
    turn_last_made: usize,
    heat_loss_grid: &Cells<HeatLoss>,
    crucible_parameters: &CrucibleParameters,
) -> bool {
    let (x, y, direction) = x_y_direction;
    if turn_last_made < crucible_parameters.min_in_straight_line {
        let (delta_x, delta_y) = match direction {
            Direction::Up => (0isize, -1isize),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };
        let still_to_go = (crucible_parameters.min_in_straight_line - turn_last_made) as isize;
        let (forced_x, forced_y) = (delta_x * still_to_go, delta_y * still_to_go);
        if !heat_loss_grid.in_bounds(x + forced_x, y + forced_y) {
            return false;
        }
    }
    true
}

#[inline]
fn construct_move(
    x_y_direction: (isize, isize, Direction),
    turn_last_made: usize,
    heat_loss_grid: &Cells<HeatLoss>,
    best_so_far: &mut Cells<HashMap<BestSoFarKey, usize>>,
    previous_move: &Move,
    crucible_parameters: &CrucibleParameters,
) -> Option<Move> {
    let (x, y, direction) = x_y_direction;
    // println!("x={}, y={}, direction={:?}, turn_last_made={}", x, y, direction, turn_last_made);

    //not in bounds?
    if !heat_loss_grid.in_bounds(x, y) {
        // println!("Not in bounds");
        return None;
    }
    //Need to move a minium in this direction, can we do it?
    if !can_move_required_in_a_straight_line(x_y_direction, turn_last_made, heat_loss_grid, crucible_parameters) {
        // println!("Unable to move all required in direction");
        return None;
    }

    let (x, y) = (x as usize, y as usize);

    let heat_loss = heat_loss_grid.get(x, y).unwrap().amount;
    let cost_to_get_here = previous_move.cost + heat_loss;
    //Did we already get to the position going in the same direction after the same number of moves after turning with a lower cost?
    let best_costs_so_far = best_so_far.get_mut(x, y).unwrap();
    let key = BestSoFarKey::new(direction, turn_last_made);
    if let Some(best_cost_so_far) = best_costs_so_far.get(&key) {
        if *best_cost_so_far <= cost_to_get_here {
            //already done it as good or better, no point continuing
            return None;
        }
    };
    //not got here with a better cost with direction etc. Accept the move...
    best_costs_so_far.insert(key, cost_to_get_here);
    //...but if we are at bottom right, no point in continuing from here
    if x == heat_loss_grid.side_lengths.0 - 1 && y == heat_loss_grid.side_lengths.1 - 1 {
        // print!("Not best");
        return None;
    }
    //better cost and not at final destination, we should process this move
    Some(Move::new(x, y, direction, cost_to_get_here, turn_last_made))
}

#[inline]
fn turn_allowed(this_move: &Move, crucible_parameters: &CrucibleParameters,) -> bool {
    //can't turn unless we've been going straight for our minimum number
    return this_move.turn_last_made >= crucible_parameters.min_in_straight_line;
}

#[inline]
fn turn_left(
    heat_loss_grid: &Cells<HeatLoss>,
    best_so_far: &mut Cells<HashMap<BestSoFarKey, usize>>,
    this_move: &Move,
    crucible_parameters: &CrucibleParameters,
) -> Option<Move> {
    if !turn_allowed(this_move, crucible_parameters) {
        return None;
    }

    let (x, y) = (this_move.x, this_move.y);
    let x_y_direction = match this_move.direction {
        Direction::Up => (x as isize - 1, y as isize, Direction::Left),
        Direction::Down => (x as isize + 1, y as isize, Direction::Right),
        Direction::Left => (x as isize, y as isize + 1, Direction::Down),
        Direction::Right => (x as isize, y as isize - 1, Direction::Up),
    };
    construct_move(
        x_y_direction,
        1,
        heat_loss_grid,
        best_so_far,
        this_move,
        crucible_parameters,
    )
}

#[inline]
fn turn_right(
    heat_loss_grid: &Cells<HeatLoss>,
    best_so_far: &mut Cells<HashMap<BestSoFarKey, usize>>,
    this_move: &Move,
    crucible_parameters: &CrucibleParameters,
) -> Option<Move> {
    if !turn_allowed(this_move, crucible_parameters) {
        return None;
    }

    let (x, y) = (this_move.x, this_move.y);
    let x_y_direction = match this_move.direction {
        Direction::Up => (x as isize + 1, y as isize, Direction::Right),
        Direction::Down => (x as isize - 1, y as isize, Direction::Left),
        Direction::Left => (x as isize, y as isize - 1, Direction::Up),
        Direction::Right => (x as isize, y as isize + 1, Direction::Down),
    };
    construct_move(
        x_y_direction,
        1,
        heat_loss_grid,
        best_so_far,
        this_move,
        crucible_parameters,
    )
}

#[inline]
fn continue_straight_on_allowed(this_move: &Move, crucible_parameters: &CrucibleParameters,) -> bool {
    this_move.turn_last_made + 1 <= crucible_parameters.max_in_straight_line
}

#[inline]
fn go_straight(
    heat_loss_grid: &Cells<HeatLoss>,
    best_so_far: &mut Cells<HashMap<BestSoFarKey, usize>>,
    this_move: &Move,
    crucible_parameters: &CrucibleParameters,
) -> Option<Move> {
    //Only allowed to go a max number in a straight line before we have to turn
    if !continue_straight_on_allowed(this_move, crucible_parameters) {
        return None;
    }

    let (x, y) = (this_move.x, this_move.y);
    let x_y_direction = match this_move.direction {
        Direction::Up => (x as isize, y as isize - 1, Direction::Up),
        Direction::Down => (x as isize, y as isize + 1, Direction::Down),
        Direction::Left => (x as isize - 1, y as isize, Direction::Left),
        Direction::Right => (x as isize + 1, y as isize, Direction::Right),
    };
    construct_move(
        x_y_direction,
        this_move.turn_last_made + 1,
        heat_loss_grid,
        best_so_far,
        this_move,
        crucible_parameters,
    )
}

fn make_next_moves(
    heat_loss_grid: &Cells<HeatLoss>,
    best_so_far: &mut Cells<HashMap<BestSoFarKey, usize>>,
    this_move: &Move,
    current_moves: &mut VecDeque<Move>,
    crucible_parameters: &CrucibleParameters,
) {
    //we can either turn 90 degrees left, turn 90 degrees right or go ahead (if we haven't been going straight for too long)
    if let Some(turn_left) =
        turn_left(heat_loss_grid, best_so_far, this_move, crucible_parameters)
    {
        current_moves.push_back(turn_left);
    };
    if let Some(turn_right) =
        turn_right(heat_loss_grid, best_so_far, this_move, crucible_parameters)
    {
        current_moves.push_back(turn_right);
    }
    if let Some(go_straight) =
        go_straight(heat_loss_grid, best_so_far, this_move, crucible_parameters)
    {
        current_moves.push_back(go_straight);
    };
}

fn perform(heat_loss_grid: &Cells<HeatLoss>, crucible_parameters: CrucibleParameters) -> usize {
    let mut best_so_far: Cells<HashMap<BestSoFarKey, usize>> = Cells::with_dimension(
        heat_loss_grid.side_lengths.0,
        heat_loss_grid.side_lengths.1,
        HashMap::default(),
    );
    let mut current_moves: VecDeque<Move> = VecDeque::default();
    //prime
    current_moves.push_back(Move::new(0, 0, Direction::Right, 0, 0));
    best_so_far
        .get_mut(0, 0)
        .unwrap()
        .insert(BestSoFarKey::new(Direction::Right, 1), 0);
    current_moves.push_back(Move::new(0, 0, Direction::Down, 0, 0));
    best_so_far
        .get_mut(0, 0)
        .unwrap()
        .insert(BestSoFarKey::new(Direction::Down, 1), 0);
    //Run
    while let Some(this_move) = current_moves.pop_front() {
        make_next_moves(
            heat_loss_grid,
            &mut best_so_far,
            &this_move,
            &mut current_moves,
            &crucible_parameters,
        );
    }
    //look at the last square to see what the best was
    let bottom_right = best_so_far
        .get(
            best_so_far.side_lengths.0 - 1,
            best_so_far.side_lengths.1 - 1,
        )
        .unwrap();
    *bottom_right
        .values()
        .min()
        .expect("Didn't find a bottom right best")
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    Ok(perform(
        &state,
        CrucibleParameters {
            min_in_straight_line: 0,
            max_in_straight_line: 3,
        },
    ))
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    Ok(perform(
        &state,
        CrucibleParameters {
            min_in_straight_line: 4,
            max_in_straight_line: 10,
        },
    ))
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
        InitialState::new_empty(),
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
        InitialState::new_empty(),
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
