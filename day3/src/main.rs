use std::collections::HashSet;

use once_cell::sync::Lazy;
use processor::process;

type AError = anyhow::Error;
type InitialState = Vec<Vec<Cell>>;
type LoadedState = Vec<Vec<PartCell>>;
type ProcessedState1 = Vec<PartCell>;
type ProcessedState2 = Vec<(PartCell, PartCell)>;
type FinalResult = u64;

#[derive(Debug, Clone, Copy)]
enum Cell {
    Number(u64),
    Dot,
    Symbol(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum PartCell {
    PartNumber { id: u32, number: u64 },
    Dot,
    Symbol(char),
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
        perform_processing_1,
        calc_result_1,
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
        perform_processing_2,
        calc_result_2,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    let mut cells = Vec::new();
    for c in line.chars() {
        let cell = c
            .to_digit(10)
            .map(|d: u32| Cell::Number(d.into()))
            .unwrap_or_else(|| match c {
                '.' => Cell::Dot,
                _ => Cell::Symbol(c),
            });
        cells.push(cell);
    }
    state.push(cells);
    Ok(state)
}

fn calculate_part_cell_number(current_number: &Vec<&u64>, current_id: &mut u32) -> PartCell {
    let part_number = current_number
        .iter()
        .rev()
        .enumerate()
        .fold(0u64, |acc, (index, number)| {
            acc + 10u64.pow(index as u32) * **number
        });
    let part_number = PartCell::PartNumber {
        id: *current_id,
        number: part_number,
    };
    *current_id = *current_id + 1;
    part_number
}

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
    let mut current_id: u32 = 0;

    let final_state: LoadedState = state
        .iter()
        .map(|cells| {
            let mut current_number = Vec::new();
            let mut part_cells: Vec<PartCell> = Vec::with_capacity(cells.len());

            fn write_part_numbers(
                current_number: &mut Vec<&u64>,
                current_id: &mut u32,
                index: &usize,
                part_cells: &mut Vec<PartCell>,
            ) {
                let part_cell_number = calculate_part_cell_number(current_number, current_id);
                for cell_index in (*index - current_number.len())..*index {
                    part_cells[cell_index] = part_cell_number.clone();
                }
                current_number.clear();
            }

            for (i, cell) in cells.iter().enumerate() {
                let write_part_number = match cell {
                    Cell::Number(n) => {
                        current_number.push(n);
                        part_cells.push(PartCell::Dot);
                        false
                    }
                    Cell::Symbol(s) => {
                        part_cells.push(PartCell::Symbol(*s));
                        true
                    }
                    Cell::Dot => {
                        part_cells.push(PartCell::Dot);
                        true
                    }
                };
                if write_part_number && !current_number.is_empty() {
                    write_part_numbers(&mut current_number, &mut current_id, &i, &mut part_cells)
                }
            }
            if !current_number.is_empty() {
                write_part_numbers(
                    &mut current_number,
                    &mut current_id,
                    &cells.len(),
                    &mut part_cells,
                )
            }
            part_cells
        })
        .collect();
    // for cells in &final_state {
    //     println!("{:?}", cells);
    // }
    Ok(final_state)
}

static ADJACENT_DELTAS: Lazy<Vec<(i8, i8)>> = Lazy::new(|| {
    Vec::from([
        (-1, -1),
        (0, -1),
        (1, -1), //line above
        (-1, 0),
        (1, 0), //this line
        (-1, 1),
        (0, 1),
        (1, 1), //line below
    ])
});

fn is_symbol_cell(cell: &PartCell) -> bool {
    match cell {
        PartCell::Symbol(_) => true,
        _ => false,
    }
}

fn is_adjacent_to_symbol_in_line(
    x: usize,
    cells: &Vec<PartCell>,
    ignore_current_pos: bool,
) -> bool {
    //before
    if x > 0 {
        let part_cell = cells.get(x - 1).unwrap();
        if is_symbol_cell(part_cell) {
            return true;
        };
    }
    //this
    if !ignore_current_pos {
        let part_cell = cells.get(x).unwrap();
        if is_symbol_cell(part_cell) {
            return true;
        };
    }
    //after
    if x < cells.len() - 1 {
        let part_cell = cells.get(x + 1).unwrap();
        if is_symbol_cell(part_cell) {
            return true;
        };
    }
    false
}

fn is_adjacent_to_symbol(x: usize, y: usize, state: &LoadedState) -> bool {
    //line above
    if y > 0 {
        let cells = state.get(y - 1).unwrap();
        if is_adjacent_to_symbol_in_line(x, cells, false) {
            return true;
        }
    }
    //this line
    let cells = state.get(y).unwrap();
    if is_adjacent_to_symbol_in_line(x, cells, true) {
        return true;
    }
    //line below
    if y < state.len() - 1 {
        let cells = state.get(y + 1).unwrap();
        if is_adjacent_to_symbol_in_line(x, cells, false) {
            return true;
        }
    }
    return false;
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState1, AError> {
    let mut counted_part_ids: HashSet<u32> = HashSet::new();
    let mut adjacent_parts: Vec<PartCell> = Vec::new();

    for (y, cells) in state.iter().enumerate() {
        for (x, cell) in cells.iter().enumerate() {
            match cell {
                PartCell::PartNumber { id, number: _number } => {
                    if !counted_part_ids.contains(id) {
                        if is_adjacent_to_symbol(x, y, &state) {
                            counted_part_ids.insert(*id);
                            adjacent_parts.push(cell.clone());
                        }
                    }
                }
                _ => (),
            }
        }
    }
    Ok(adjacent_parts)
}

fn calc_result_1(state: ProcessedState1) -> Result<FinalResult, AError> {
    // println!("{:?}", state);
    Ok(state
        .iter()
        .map(|p| match p {
            PartCell::PartNumber { id: _id, number } => number.clone(),
            _ => 0u64,
        })
        .sum())
}

fn get_part(x: isize, y: isize, state: &LoadedState) -> Option<PartCell> {
    if y < 0 || x < 0 {
        return None;
    }
    let y = y as usize;
    if y >= state.len() {
        return None;
    }
    let cells = state.get(y).unwrap();

    let x = x as usize;
    if x >= cells.len() {
        return None;
    }
    let cell = cells.get(x).unwrap();

    match cell {
        PartCell::PartNumber { id, number } => Some(PartCell::PartNumber {
            id: *id,
            number: *number,
        }),
        _ => None,
    }
}

fn find_adjacent_parts(centre_x: usize, centre_y: usize, state: &LoadedState) -> HashSet<PartCell> {
    ADJACENT_DELTAS
        .iter()
        .fold(HashSet::new(), |mut parts, (dx, dy)| {
            let x = centre_x as isize + *dx as isize;
            let y = centre_y as isize + *dy as isize;
            if let Some(part) = get_part(x, y, state) {
                parts.insert(part);
            }
            parts
        })
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState2, AError> {
    let mut adjacent_parts: Vec<(PartCell, PartCell)> = Vec::new();

    for (y, cells) in state.iter().enumerate() {
        for (x, cell) in cells.iter().enumerate() {
            if let Some(parts) = match cell {
                PartCell::Symbol('*') => {
                    let parts = find_adjacent_parts(x, y, &state);
                    if parts.len() == 2 {
                        let mut parts = parts.into_iter();
                        Some((parts.next().unwrap(), parts.next().unwrap()))
                    } else {
                        None
                    }
                }
                _ => None,
            } {
                adjacent_parts.push(parts);
            }
        }
    }
    Ok(adjacent_parts)
}

fn calc_result_2(state: ProcessedState2) -> Result<FinalResult, AError> {
    // println!("{:?}", state);
    Ok(state
        .iter()
        .map(|(p1, p2)| match (p1, p2) {
            (
                PartCell::PartNumber { id: _, number: n1 },
                PartCell::PartNumber { id: _, number: n2 },
            ) => n1 * n2,
            _ => 0u64,
        })
        .sum())
}
