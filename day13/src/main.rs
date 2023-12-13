use std::{collections::HashSet, fmt::Display};

use once_cell::sync::Lazy;
use processor::{process, CellsBuilder, Cells, read_word};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Cell {
    #[default]
    Ash,
    Rock,
}

impl Cell {
    fn character_rep(&self) -> char {
        match self {
            Cell::Ash => '.',
            Cell::Rock => '#',
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.character_rep())
    }
}

#[derive(Debug, Default)]
struct LoadingState {
    patterns: Vec<CellsBuilder<Cell>>,
}

struct LoadedState {
    patterns: Vec<Cells<Cell>>,
}

type UpperIndexAndSize = (usize, usize);

/// indexes are the index of the row below the mirror line
/// or the column to the right of the mirror line
#[derive(Debug)]
struct Reflection {
    rows: Vec<UpperIndexAndSize>,
    columns: Vec<UpperIndexAndSize>,
}

type AError = anyhow::Error;
type InitialState = LoadingState;
type ProcessedState = Vec<Reflection>;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::default());

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if state.patterns.is_empty() {
        state.patterns.push(CellsBuilder::default());
    }
    match read_word(&mut line.chars(), &DELIMITERS) {
        Some((line, _)) => {
            let current_builder = state.patterns.last_mut().unwrap();
            current_builder.new_line();
            line.chars().for_each(|c| {
                match c {
                    '.' => current_builder.add_cell(Cell::Ash).expect("Failed to add ash cell"),
                    '#' => current_builder.add_cell(Cell::Rock).expect("Failed to add rock cell"),
                    _ => panic!("unrecognised cell: {c}"),
                }
            })
        },
        None => state.patterns.push(CellsBuilder::default()),
    };
    Ok(state)
}

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
    let mut patterns = Vec::default();
    for mut builder in state.patterns.into_iter() {
        patterns.push(builder.build_cells(Cell::Ash)?);
    }
    Ok(LoadedState {
        patterns,
    })
}

/// If a possible reflection is found, checks that the reflection gets all the way to an edge
/// returns None if it doesn't otherwise returns Some and the number of cells reflected
/// i.e. if returns 0, then there was no reflection all the way to an edge
fn find_reflection_size(lines: &Vec<Vec<Cell>>, upper_index: usize) -> Option<usize> {
    let max_repeats_upper = lines.len() - upper_index - 1;
    let max_repeats_lower = upper_index - 1;
    let required_repeats = max_repeats_lower.min(max_repeats_upper);
    for i in 0..(required_repeats + 1) {
        let upper_line = &lines[upper_index + i];
        let lower_line = &lines[upper_index - (i + 1)];
        if upper_line != lower_line {
            return None;
        }
    }
    Some(required_repeats)
}

fn output_lines(index: usize, name: &str, lines: &Vec<Vec<Cell>>) {
    println!("{index} {name}");
    lines.iter().for_each(|line| {
        println!("{}", line.iter().map(|c| c.character_rep()).collect::<String>());
    });
    println!();
}


/// Returns the upper_index and the size of the reflection
fn find_reflection_indices(index: usize, name: &str, lines: &Vec<Vec<Cell>>) -> Vec<UpperIndexAndSize> {
    output_lines(index, name, lines);
    let mut reflections = Vec::default();
    for i in 1..lines.len() {
        let lower_line = &lines[i - 1];
        let upper_line = &lines[i];
        if lower_line == upper_line {
            if let Some(span) = find_reflection_size(lines, i) {
                reflections.push((i, span));
            }
        }
    }
    return reflections
}

fn get_mirrored_row_columns(index: usize, cells: &Cells<Cell>) -> Reflection {
    //rows
    let mut rows: Vec<Vec<Cell>> = Vec::default();
    for row in 0..cells.side_lengths.1 {
        let mut this_row = Vec::default();
        for column in 0..cells.side_lengths.0 {
            this_row.push(*cells.get(column, row).unwrap());
        }
        rows.push(this_row);
    }
    let found_rows = find_reflection_indices(index, "rows: ", &rows);
    //columns
    let mut cols: Vec<Vec<Cell>> = Vec::default();
    for column in 0..cells.side_lengths.0 {
        let mut this_column = Vec::default();
        for row in 0..cells.side_lengths.1 {
            this_column.push(*cells.get(column, row).unwrap());
        }
        cols.push(this_column);
    }
    let found_columns = find_reflection_indices(index, "columns: ", &cols);
    let num_found = found_rows.len() + found_columns.len();
    if num_found != 1 {
        panic!("Found {num_found}");
    }
    Reflection {
        rows: found_rows,
        columns: found_columns,
    }
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let row_columns = state.patterns
        .iter()
        .enumerate()
        .map(|(index, cells)| get_mirrored_row_columns(index, cells)).collect();
    Ok(row_columns)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    let values = state
        .iter()
        .enumerate()
        .map(|(_index, reflection)| {
            //Take the one with the biggest span, if we have it
            let (col_upper_index, _col_span) = reflection.columns.first().unwrap_or(&(0, 0));
            let (row_upper_index, _row_span) = reflection.rows.first().unwrap_or(&(0, 0));
            *row_upper_index * 100 + *col_upper_index
        });
    Ok(values.sum())
}

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        LoadingState::default(),
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
        LoadingState::default(),
        parse_line,
        finalise_state,
        perform_processing,
        calc_result,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}
