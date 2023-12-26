use std::{collections::HashSet, fmt::Display, time};

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
            velocity: ICoord3::new(v_x, v_y, v_z),
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

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
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
        stone.position.y + stone.velocity.y,
    )
}

//https://www.topcoder.com/thrive/articles/Geometry%20Concepts%20part%202:%20%20Line%20Intersection%20and%20its%20Applications
fn paths_intersect_x_y(
    min: Float,
    max: Float,
    a: HailStone,
    b: HailStone,
) -> Option<(Float, Float)> {
    let (a1, b1, c1) = line_a_b_c(&a);
    let (a2, b2, c2) = line_a_b_c(&b);

    let det = a1 * b2 - a2 * b1;
    if det == 0.0 {
        return None; //parallel
    }
    let intersection_x = (b2 * c1 - b1 * c2) / det;
    let intersection_y = (a1 * c2 - a2 * c1) / det;

    //Is the intersection within the bounds?
    if intersection_x < min || intersection_x > max || intersection_y < min || intersection_y > max
    {
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
            if i == j {
                //shouldn't be necessary but just in case
                continue;
            }
            if let Some((_time_1, _time_2)) =
                paths_intersect_x_y(min, max, state.hailstones[i], state.hailstones[j])
            {
                // println!("{} {}", time_1, time_2);
                collisions += 1
            }
        }
    }
    Ok(collisions)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

type ProcessedState2 = Rational64;
type FinalResult2 = Rational64;

#[inline]
fn as_rational(i: isize) -> Rational64 {
    Rational64::from_integer(i.try_into().unwrap())
}

/// See https://math.stackexchange.com/a/3176648
///
/// Returns ((intersection_x, intersection_y), time_of_intersect) if one can be found
fn get_intersect_pos_time(
    stone_a: &HailStone,
    stone_b: &HailStone,
    delta_x: isize,
    delta_y: isize,
) -> Option<((Rational64, Rational64), Rational64)> {
    let zero = as_rational(0);
    let minus1 = as_rational(-1);

    let pos_ax = as_rational(stone_a.position.x);
    let pos_ay = as_rational(stone_a.position.y);
    let pos_bx = as_rational(stone_b.position.x);
    let pos_by = as_rational(stone_b.position.y);
    let vel_ax = as_rational(stone_a.velocity.x + delta_x);
    let vel_ay = as_rational(stone_a.velocity.y + delta_y);
    let vel_bx = as_rational(stone_b.velocity.x + delta_x);
    let vel_by = as_rational(stone_b.velocity.y + delta_y);

    let det = (vel_ax * minus1 * vel_by) - (vel_ay * minus1 * vel_bx);

    if det == zero {
        return None;
    }

    let qx = (minus1 * vel_by * (pos_bx - pos_ax)) - (minus1 * vel_bx * (pos_by - pos_ay));
    let qy = (vel_ax * (pos_by - pos_ay)) - (vel_ay * (pos_bx - pos_ax));

    let t = qx / det;
    let _s = qy / det;

    let px = pos_ax + t * vel_ax;
    let py = pos_ay + t * vel_ay;

    Some(((px, py), t))
}

const RANGE: isize = 337; //Smallest that we can still find it

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState2, AError> {
    let stone_0 = state.hailstones[0];
    let stone_1 = state.hailstones[1];
    let stone_2 = state.hailstones[2];
    let stone_3 = state.hailstones[3];

    let mut found_pos: Option<(Rational64, Rational64, Rational64)> = None;
    'outer: for x in -RANGE..RANGE + 1 {
        // if x % 1000 == 0 {
        //     println!("{x}");
        // }
        for y in -RANGE..RANGE + 1 {
            //find the intersection of the hailstones when modifying the velocities by x, y
            //(i.e. we are looking to calculate where the rock came from if it had velocity (-x, -y) for each stone with stone_0)
            let intersect1 = get_intersect_pos_time(&stone_1, &stone_0, x, y);
            let intersect2 = get_intersect_pos_time(&stone_2, &stone_0, x, y);
            let intersect3 = get_intersect_pos_time(&stone_3, &stone_0, x, y);

            let (coord, time1, time2, time3) = match (intersect1, intersect2, intersect3) {
                (Some((coord1, time1)), Some((coord2, time2)), Some((coord3, time3)))
                    if coord1 == coord2 && coord1 == coord3 =>
                {
                    (coord1, time1, time2, time3)
                }
                _ => continue,
            };

            //Now get the z velocity
            for z in -RANGE..RANGE + 1 {
                //Check z positions intersect at the time from our x,y calculation
                let z_intersect1 =
                    as_rational(stone_1.position.z) + time1 * as_rational(stone_1.velocity.z + z);
                let z_intersect2 =
                    as_rational(stone_2.position.z) + time2 * as_rational(stone_2.velocity.z + z);
                let z_intersect3 =
                    as_rational(stone_3.position.z) + time3 * as_rational(stone_3.velocity.z + z);

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
