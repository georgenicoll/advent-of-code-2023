use std::{time, fmt::Display, collections::{HashSet, HashMap}};

use num_rational::Rational64;
use once_cell::sync::Lazy;
use processor::{process, read_next};

type AError = anyhow::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ICoord3 {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl ICoord3 {
    pub fn new(x: isize, y: isize, z: isize) -> ICoord3 {
        ICoord3 { x, y, z }
    }
}

impl Display for ICoord3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy)]
struct HailStone {
    id: usize,
    position: ICoord3,
    velocity: ICoord3,
}

impl Display for HailStone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} @ {}", self.id, self.position, self.velocity)
    }
}

struct State {
    test_area: (isize, isize),
    hailstones: Vec<HailStone>,
}

type InitialState = State;
type LoadedState = InitialState;
type ProcessedState = usize;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ', ',', '@']));

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if !line.is_empty() {
        let mut chars = line.chars();
        let (x, _) = read_next::<isize>(&mut chars, &DELIMITERS)?;
        let (y, _) = read_next::<isize>(&mut chars, &DELIMITERS)?;
        let (z, _) = read_next::<isize>(&mut chars, &DELIMITERS)?;
        let (v_x, _) = read_next::<isize>(&mut chars, &DELIMITERS)?;
        let (v_y, _) = read_next::<isize>(&mut chars, &DELIMITERS)?;
        let (v_z, _) = read_next::<isize>(&mut chars, &DELIMITERS)?;
        let hailstone = HailStone {
            id: state.hailstones.len() + 1,
            position: ICoord3::new(x, y, z),
            velocity: ICoord3::new(v_x, v_y, v_z)
        };
        state.hailstones.push(hailstone);
    };
    Ok(state)
}

// fn output_hailstones(hailstones: &Vec<HailStone>) {
//     println!("HailStones:");
//     hailstones.iter().for_each(|hailstone| println!("{hailstone}"));
//     println!();
// }

fn output_state(_state: &State) {
    // println!("Bounds: {:?}", state.test_area);
    // output_hailstones(&state.hailstones);
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    state.hailstones.truncate(5);
    output_state(&state);
    Ok(state)
}

type Float = f64;

fn line_a_b_c_from_points(x1: isize, x2: isize, y1: isize, y2: isize) -> (Float, Float, Float) {
    let x1 = x1 as Float;
    let x2 = x2 as Float;
    let y1 = y1 as Float;
    let y2 = y2 as Float;

    let a = y2 - y1;
    let b = x1 - x2;
    let c = a * x1 + b * y1;

    (a, b, c)
}

fn line_a_b_c(stone: &HailStone) -> (Float, Float, Float) {
    line_a_b_c_from_points(
        stone.position.x,
        stone.position.x + stone.velocity.x,
        stone.position.y,
        stone.position.y + stone.velocity.y
    )
}

//https://www.topcoder.com/thrive/articles/Geometry%20Concepts%20part%202:%20%20Line%20Intersection%20and%20its%20Applications
fn paths_intersect_x_y(min: Float, max: Float, a: HailStone, b: HailStone) -> Option<(Float, Float)> {
    let (a1, b1, c1) = line_a_b_c(&a);
    let (a2, b2, c2) = line_a_b_c(&b);

    let det = a1 * b2 - a2 * b1;
    if det == 0.0 {
        return None; //parallel
    }
    let intersection_x = (b2 * c1 - b1 * c2) / det;
    let intersection_y = (a1 * c2 - a2 * c1) / det;

    //Is the intersection within the bounds?
    if intersection_x < min || intersection_x > max || intersection_y < min || intersection_y > max {
        return None; //out of bounds
    }

    //check time is positive for a
    let x_0 = a.position.x as Float;
    let v_x = a.velocity.x as Float;
    let time_a = (intersection_x - x_0) / v_x;
    if time_a < 0.0 {
        return None;
    }

    //check time is positive for b
    let x_0 = b.position.x as Float;
    let v_x = b.velocity.x as Float;
    let time_b = (intersection_x - x_0) / v_x;

    if time_b >= 0.0 {
        Some((time_a, time_b))
    } else {
        None
    }
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let min = state.test_area.0 as Float;
    let max = state.test_area.1 as Float;
    let mut collisions = 0usize;
    for i in 0..state.hailstones.len() {
        for j in (i + 1)..state.hailstones.len() {
            if i == j { //shouldn't be necessary but just in case
                continue;
            }
            if let Some((time_1, time_2)) = paths_intersect_x_y(min, max, state.hailstones[i], state.hailstones[j]) {
                println!("{} {}", time_1, time_2);
                collisions += 1
            }
        }
    }
    Ok(collisions)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

type ProcessedState2 = f64;
type FinalResult2 = f64;

const EPSILON: f64 = 0.0000000000001;

fn intersect_on_integer_nanos(a1: Float, b1: Float, c1: Float, stone: &HailStone) -> bool {
    let (a2, b2, c2) = line_a_b_c_from_points(
        stone.position.x,
        stone.position.x + stone.velocity.x * 10,
        0,
        10
    );

    let det = &a1 * &b2 - &a2 * &b1;
    if det == 0.0 {
        return false; //parallel
    }
    let _intersection_x = (b2 * c1 - b1 * c2) / det;
    let intersection_t = (a1 * c2 - a2 * c1) / det;

    //if intersection_t is not +ve , we need to try again
    if intersection_t < 0.0 {
        return false;
    }

    let floor = intersection_t.floor();
    let ceil = intersection_t.ceil();
    (intersection_t - floor) < EPSILON || (ceil - intersection_t) < EPSILON
}

fn project_line(
    hailstones: &Vec<HailStone>,
    stone_a: &HailStone,
    stone_b: &HailStone,
    first_nano: usize,
    second_nano: usize,
) -> bool {
    //find the line between the stones at the 2 nanos
    let (a1, b1, c1) = line_a_b_c_from_points(
        stone_a.position.x + stone_a.velocity.x * first_nano as isize,
        stone_b.position.x + stone_b.velocity.x * second_nano as isize,
        first_nano as isize,
        second_nano as isize,
    );
    //now check to see whether all of the hailstones can match this line
    hailstones.iter().all(|stone|
        intersect_on_integer_nanos(a1, b1, c1, stone)
    )
}

fn calculate_line_start(a: &HailStone, b: &HailStone, first_nano: usize, second_nano: usize) -> ICoord3 {
    let start_pos = ICoord3::new(
        a.position.x + a.velocity.x * first_nano as isize,
        a.position.y + a.velocity.y * first_nano as isize,
        a.position.z + a.velocity.z * first_nano as isize,
    );
    let end_pos = ICoord3::new(
        b.position.x + b.velocity.x * second_nano as isize,
        b.position.y + b.velocity.y * second_nano as isize,
        b.position.z + b.velocity.z * second_nano as isize,
    );
    let line_velocity = ICoord3::new(
        (end_pos.x - start_pos.x) / (second_nano - first_nano) as isize,
        (end_pos.y - start_pos.y) / (second_nano - first_nano) as isize,
        (end_pos.z - start_pos.z) / (second_nano - first_nano) as isize,
    );
    ICoord3::new(
        start_pos.x - line_velocity.x * first_nano as isize,
        start_pos.y - line_velocity.y * first_nano as isize,
        start_pos.z - line_velocity.z * first_nano as isize,
    )
}

const SEARCH_NANOS_START: usize = 1;
const SEARCH_NANOS_END: usize = 1_000_000;

// fn perform_processing_2(state: LoadedState) -> Result<ProcessedState2, AError> {
//     //take the first 2 hailstones, we're going to see if we can draw lines through these to get
//     //a line that matches all points...  Note Just matching x initially
//     let stone_a = state.hailstones[0];
//     let stone_b = state.hailstones[1];
//     let hailstones: Vec<HailStone> = state.hailstones.iter().skip(2).cloned().collect();

//     let mut found_position: Option<ICoord3> = None;
//     'outer: for first_nano in SEARCH_NANOS_START..SEARCH_NANOS_END {
//         if first_nano % 1000 == 0 {
//             println!("first_nano: {}", first_nano);
//         }
//         for second_nano in first_nano + 1..SEARCH_NANOS_END {
//             if project_line(&hailstones, &stone_a, &stone_b, first_nano, second_nano) {
//                 //Done! it's this one calculate where we would need to start
//                 found_position = Some(calculate_line_start(&stone_a, &stone_b, first_nano, second_nano));
//                 break 'outer;
//             }
//             //try the other way
//             if project_line(&hailstones, &stone_b, &stone_a, first_nano, second_nano) {
//                 //Done! it's this one calculate where we would need to start
//                 found_position = Some(calculate_line_start(&stone_b, &stone_a, first_nano, second_nano));
//                 break 'outer;
//             }
//         }
//     }
//     //search the second from nanos
//     let found_position = found_position.expect("Didn't find it");
//     Ok(found_position.x + found_position.y + found_position.z)
// }

fn get_intersect_pos_time(stone_a: &HailStone, stone_b: &HailStone, delta_x: isize, delta_y: isize) -> Option<((Rational64, Rational64), Rational64)> {
    let (a1, b1, c1) = line_a_b_c_from_points(
        stone_a.position.x,
        stone_a.position.x +(stone_a.velocity.x + delta_x),
        stone_a.position.y,
        stone_a.position.y +(stone_a.velocity.y + delta_y),
    );
    let (a2, b2, c2) = line_a_b_c_from_points(
        stone_b.position.x,
        stone_b.position.x +(stone_b.velocity.x + delta_x),
        stone_b.position.y,
        stone_b.position.y +(stone_b.velocity.y + delta_y),
    );

    let det = a1 * b2 - a2 * b1;
    if det == 0.0 {
        return None; //parallel
    }
    let intersection_x = (b2 * c1 - b1 * c2) / det;
    let intersection_y = (a1 * c2 - a2 * c1) / det;

    //check time is positive for a
    let x_0 = stone_a.position.x as Float;
    let v_x = (stone_a.velocity.x + delta_x) as Float;
    let time_a = (intersection_x - x_0) / v_x;
    if time_a < 0.0 {
        return None; //intersection is negative
    }

    //check time is positive for b
    let x_0 = stone_b.position.x as Float;
    let v_x = (stone_b.velocity.x + delta_x) as Float;
    let time_b = (intersection_x - x_0) / v_x;

    if time_b >= 0.0 {
        Some(((intersection_x, intersection_y), time_a))
    } else {
        None
    }

}

const RANGE:isize = 100_000;

//Copy
fn perform_processing_2(state: LoadedState) -> Result<ProcessedState2, AError> {
    let stone_0 = state.hailstones[0];
    let stone_1 = state.hailstones[1];
    let stone_2 = state.hailstones[2];
    let stone_3 = state.hailstones[3];

    let mut found_pos: Option<(f64, f64, f64)> = None;
    'outer: for x in -RANGE..RANGE {
        if x % 1000 == 0 {
            println!("{x}");
        }
        for y in -RANGE..RANGE {
            //find the intersection of the hailstones when modifying the velocities by x, y
            //(i.e. the opposite direction velocity of the rock path)
            let intersect1 = get_intersect_pos_time(&stone_1, &stone_0, x, y);
            let intersect2 = get_intersect_pos_time(&stone_2, &stone_0, x, y);
            let intersect3 = get_intersect_pos_time(&stone_3, &stone_0, x, y);

            let (coord, time1, time2, time3) = match (intersect1, intersect2, intersect3) {
                (
                    Some((coord1, time1)),
                    Some((coord2, time2)),
                    Some((coord3, time3)),
                )
                if coord1 == coord2 && coord1 == coord3 => (coord1, time1, time2, time3),
                _ => continue,
            };

            //Now get the z velocity
            for z in -RANGE..RANGE {
                //We know what time we would intersect...  so just check the z pos
                let z_intersect1 = stone_1.position.z as f64 + time1 * (stone_1.velocity.z + z) as f64;
                let z_intersect2 = stone_2.position.z as f64 + time2 * (stone_2.velocity.z + z) as f64;
                let z_intersect3 = stone_3.position.z as f64 + time3 * (stone_3.velocity.z + z) as f64;

                if z_intersect1 == z_intersect2 && z_intersect1 == z_intersect3 {
                    //Found it
                    found_pos = Some((coord.0, coord.1, z_intersect1));
                    break 'outer;
                }

            }
        }
    }
    let (x, y, z) = found_pos.expect("Didn't find it");
    Ok(x + y + z)
}

fn calc_result_2(state: ProcessedState2) -> Result<FinalResult2, AError> {
    Ok(state)
}

fn main() {
    //let (bounds, file) = ((7isize, 27isize), "test-input.txt");
    //let (bounds, file) = ((7isize, 27isize), "test-input2.txt");
    let (bounds, file) = ((200000000000000isize, 400000000000000isize), "input.txt");

    fn initial_state(bounds: (isize, isize)) -> State {
        State {
            test_area: bounds,
            hailstones: Vec::default(),
        }
    }

    let started1_at = time::Instant::now();
    let result1 = process(
        file,
        initial_state(bounds),
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
        initial_state(bounds),
        parse_line,
        finalise_state,
        perform_processing_2,
        calc_result_2,
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
