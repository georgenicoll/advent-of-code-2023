use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashSet, VecDeque},
    fmt::Display,
};

use once_cell::sync::Lazy;
use processor::{process, read_next, Coord3};

#[derive(Debug, Clone)]
struct Brick {
    id: usize,
    corner1: Coord3,
    corner2: Coord3,
    supported_by_ids: HashSet<usize>, //ids of bricks that are supporting this
    supporting_ids: HashSet<usize>, //ids of bricks that this is supporting
}

impl Brick {
    fn min_x(&self) -> usize {
        self.corner1.x.min(self.corner2.x)
    }

    fn max_x(&self) -> usize {
        self.corner1.x.max(self.corner2.x)
    }

    fn min_y(&self) -> usize {
        self.corner1.y.min(self.corner2.y)
    }

    fn max_y(&self) -> usize {
        self.corner1.y.max(self.corner2.y)
    }

    fn min_z(&self) -> usize {
        self.corner1.z.min(self.corner2.z)
    }

    fn max_z(&self) -> usize {
        self.corner1.z.max(self.corner2.z)
    }

    fn overlaps_x_y(&self, other: &Brick) -> bool {
        self.min_x() <= other.max_x()
            && self.max_x() >= other.min_x()
            && self.min_y() <= other.max_y()
            && self.max_y() >= other.min_y()
    }
}

impl Display for Brick {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}~{}", self.corner1, self.corner2)
    }
}

type AError = anyhow::Error;

type InitialState = Vec<Brick>;

type LoadedState = InitialState;
type ProcessedState = BTreeMap<usize, Brick>;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([',', '~']));

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        let mut chars = line.chars();
        let (x1, _) = read_next::<usize>(&mut chars, &DELIMITERS)?;
        let (y1, _) = read_next::<usize>(&mut chars, &DELIMITERS)?;
        let (z1, _) = read_next::<usize>(&mut chars, &DELIMITERS)?;
        let (x2, _) = read_next::<usize>(&mut chars, &DELIMITERS)?;
        let (y2, _) = read_next::<usize>(&mut chars, &DELIMITERS)?;
        let (z2, _) = read_next::<usize>(&mut chars, &DELIMITERS)?;
        state.push(Brick {
            id: state.len(),
            corner1: Coord3::new(x1, y1, z1),
            corner2: Coord3::new(x2, y2, z2),
            supporting_ids: HashSet::default(),
            supported_by_ids: HashSet::default(),
        })
    }
    Ok(state)
}

fn sortby_z_y_x(a: &Brick, b: &Brick) -> Ordering {
    Ordering::Equal
        .then(a.min_z().cmp(&b.min_z()))
        .then(a.min_y().cmp(&b.min_y()))
        .then(a.min_x().cmp(&b.min_x()))
}

fn output_bricks(_bricks: &[Brick]) {
    // println!("Bricks:");
    // bricks.iter().for_each(|b| println!("{b}"));
    // println!();
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    output_bricks(&state);
    //Sort by the lowest z then the lowest y then the lowest x
    state.sort_by(sortby_z_y_x);
    output_bricks(&state);
    Ok(state)
}

fn place_brick(brick: &Brick, stacked: &mut BTreeMap<usize, Brick>) {
    //previous bricks will be stacked 'lowest' to highest. See if we overlap any other bricks
    //if we overlap then we have to put our brick above the other brick (brick.max_z + 1)... otherwise
    //we can put the brick at the bottom (z=1)
    let (max_z, supporting_bricks) = stacked
        .values()
        .filter(|other| brick.overlaps_x_y(other))
        .fold(
            (0, HashSet::default()),
            |(max_z_so_far, mut supporting), other| {
                match max_z_so_far.cmp(&other.max_z()) {
                    Ordering::Equal => {
                        //at the same level, this and others are supporting -> add to the supporting bricks
                        supporting.insert(other.id);
                        (max_z_so_far, supporting)
                    }
                    Ordering::Less => {
                        //new one is higher -> this will be supporting instead of the other ones
                        supporting.clear();
                        supporting.insert(other.id);
                        (other.max_z(), supporting)
                    }
                    Ordering::Greater => {
                        //overlapping but another higher is supporting -> this one can be discounted
                        (max_z_so_far, supporting)
                    }
                }
            },
        );
    //update the supporting_ids on the bricks that are supporting this one
    supporting_bricks.iter().for_each(|id| {
        let other = stacked.get_mut(id).unwrap();
        other.supporting_ids.insert(brick.id);
    });
    //and add the new stacked brick at its new level
    let z_adjustment = brick.min_z() - max_z - 1;
    let stacked_brick = Brick {
        id: brick.id,
        corner1: Coord3::new(
            brick.corner1.x,
            brick.corner1.y,
            brick.corner1.z - z_adjustment,
        ),
        corner2: Coord3::new(
            brick.corner2.x,
            brick.corner2.y,
            brick.corner2.z - z_adjustment,
        ),
        supported_by_ids: supporting_bricks,
        supporting_ids: HashSet::default(),
    };
    stacked.insert(stacked_brick.id, stacked_brick);
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    //take each brick (assuming that we are dealing with the lowest first)
    //and try to place them as close to the bottom as possible according to the floor (z > 0)
    //and any other bricks
    let mut stacked: BTreeMap<usize, Brick> = BTreeMap::default();
    for brick in state {
        place_brick(&brick, &mut stacked);
    }
    Ok(stacked)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    let mut num_can_be_disintegrated = 0usize;
    'outer: for brick in state.values() {
        //check that each of the bricks supported by this has at least another support
        for id in brick.supporting_ids.iter() {
            let other = state.get(id).unwrap();
            if other.supported_by_ids.len() <= 1 {
                //No other supports...
                continue 'outer;
            }
        }
        //all had other supports
        num_can_be_disintegrated += 1;
    }
    Ok(num_can_be_disintegrated)
}

fn calc_result_2(state: ProcessedState) -> Result<FinalResult, AError> {
    let mut total_number = 0usize;
    for brick in state.values() {
        //disintegrate this, how many will fall
        let mut brick_ids: HashSet<usize> = HashSet::default();
        let mut ids_to_process: VecDeque<usize> = VecDeque::default();
        ids_to_process.push_back(brick.id);
        while let Some(id) = ids_to_process.pop_front() {
            if brick_ids.contains(&id) {
                continue;
            }
            brick_ids.insert(id);

            let supported_ids = {
                let brick = state.get(&id).unwrap();
                brick.supporting_ids.clone()
            };
            //are we removing all of the bricks that support the supported ids?
            for supported_id in supported_ids.iter() {
                let supported = state.get(supported_id).unwrap();
                //if the supported by ids are a subset of bricks, then supported now has no support -
                //so, remove it as well... add to the list to process
                if supported.supported_by_ids.is_subset(&brick_ids) {
                    ids_to_process.push_back(*supported_id);
                }
            }
        }
        //only interested in others so we -1 to remove this id which will be in the set
        total_number += brick_ids.len() - 1;
    }
    Ok(total_number)
}

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        Vec::new(),
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
        Vec::new(),
        parse_line,
        finalise_state,
        perform_processing,
        calc_result_2,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}
