use std::collections::HashSet;

use processor::{adjacent_coords, process, Cells, CellsBuilder};

type AError = anyhow::Error;
type InitialState = CellsBuilder<Cell>;
type LoadedState = Cells<PartCell>;
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
        CellsBuilder::new_empty(),
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
        CellsBuilder::new_empty(),
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
    state.new_line();
    for c in line.chars() {
        let cell = c
            .to_digit(10)
            .map(|d: u32| Cell::Number(d.into()))
            .unwrap_or_else(|| match c {
                '.' => Cell::Dot,
                _ => Cell::Symbol(c),
            });
        state.add_cell(cell)?;
    }
    Ok(state)
}

fn calculate_part_cell_number(current_number: &[&u64], current_id: &mut u32) -> PartCell {
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
    *current_id += 1;
    part_number
}

fn write_part_numbers(
    current_number: &mut Vec<&u64>,
    current_id: &mut u32,
    builder: &mut CellsBuilder<PartCell>,
    x: usize,
    y: usize,
) {
    let part_cell_number = calculate_part_cell_number(current_number, current_id);
    for cell_index in (x - current_number.len())..x {
        let current = builder.get_mut(cell_index, y).unwrap();
        *current = part_cell_number;
    }
    current_number.clear();
}

fn finalise_state(mut state: InitialState) -> Result<LoadedState, AError> {
    let mut current_id: u32 = 0;
    let cells = state.build_cells(Cell::Dot)?;

    let mut builder = CellsBuilder::new_empty();
    for y in 0..cells.side_lengths.1 {
        builder.new_line();

        let mut current_number = Vec::new();

        for x in 0..cells.side_lengths.0 {
            let cell = cells.get(x, y)?;
            let write_part_number = match cell {
                Cell::Number(n) => {
                    current_number.push(n);
                    builder.add_cell(PartCell::Dot)?;
                    false
                }
                Cell::Symbol(s) => {
                    builder.add_cell(PartCell::Symbol(*s))?;
                    true
                }
                Cell::Dot => {
                    builder.add_cell(PartCell::Dot)?;
                    true
                }
            };
            if write_part_number && !current_number.is_empty() {
                write_part_numbers(&mut current_number, &mut current_id, &mut builder, x, y)
            }
        }

        if !current_number.is_empty() {
            write_part_numbers(
                &mut current_number,
                &mut current_id,
                &mut builder,
                cells.side_lengths.0,
                y,
            )
        }
    }
    builder.build_cells(PartCell::Dot)
}

fn is_symbol_cell(cell: &PartCell) -> bool {
    matches!(cell, PartCell::Symbol(_))
}

fn is_adjacent_to_symbol(x: usize, y: usize, state: &LoadedState) -> bool {
    let centre = (x, y);
    let adjacent_coords = adjacent_coords(&centre, &state.side_lengths);
    adjacent_coords.iter().any(|(x, y)| {
        let cell = state.get(*x, *y).unwrap();
        is_symbol_cell(cell)
    })
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState1, AError> {
    let mut counted_part_ids: HashSet<u32> = HashSet::new();
    let mut adjacent_parts: Vec<PartCell> = Vec::new();

    for ((x, y), cell) in state.iter() {
        if let PartCell::PartNumber {
            id,
            number: _number,
        } = cell
        {
            if !counted_part_ids.contains(id) && is_adjacent_to_symbol(x, y, &state) {
                counted_part_ids.insert(*id);
                adjacent_parts.push(*cell);
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
            PartCell::PartNumber { id: _id, number } => *number,
            _ => 0u64,
        })
        .sum())
}

fn get_part(x: usize, y: usize, state: &LoadedState) -> Option<PartCell> {
    let cell = state.get(x, y).unwrap();

    match cell {
        PartCell::PartNumber { id, number } => Some(PartCell::PartNumber {
            id: *id,
            number: *number,
        }),
        _ => None,
    }
}

fn find_adjacent_parts(x: usize, y: usize, state: &LoadedState) -> HashSet<PartCell> {
    let mut parts = HashSet::new();
    let centre = (x, y);
    let coords = adjacent_coords(&centre, &state.side_lengths);
    coords.iter().for_each(|(x, y)| {
        if let Some(part) = get_part(*x, *y, state) {
            parts.insert(part);
        }
    });
    parts
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState2, AError> {
    let mut adjacent_parts: Vec<(PartCell, PartCell)> = Vec::new();

    for ((x, y), cell) in state.iter() {
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
