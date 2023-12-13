use std::{
    collections::{BTreeSet, HashSet},
    fmt::Display,
};

use once_cell::sync::Lazy;
use processor::{process, read_word, Cells, CellsBuilder};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Reflection {
    rows: BTreeSet<UpperIndexAndSize>,
    columns: BTreeSet<UpperIndexAndSize>,
}

impl Display for Reflection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {:?})", self.columns, self.rows)
    }
}

type AError = anyhow::Error;
type InitialState = LoadingState;
type ProcessedState = Vec<Reflection>;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(HashSet::default);

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    if state.patterns.is_empty() {
        state.patterns.push(CellsBuilder::default());
    }
    match read_word(&mut line.chars(), &DELIMITERS) {
        Some((line, _)) => {
            let current_builder = state.patterns.last_mut().unwrap();
            current_builder.new_line();
            line.chars().for_each(|c| match c {
                '.' => current_builder
                    .add_cell(Cell::Ash)
                    .expect("Failed to add ash cell"),
                '#' => current_builder
                    .add_cell(Cell::Rock)
                    .expect("Failed to add rock cell"),
                _ => panic!("unrecognised cell: {c}"),
            })
        }
        None => state.patterns.push(CellsBuilder::default()),
    };
    Ok(state)
}

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
    let mut patterns = Vec::default();
    for mut builder in state.patterns.into_iter() {
        patterns.push(builder.build_cells(Cell::Ash)?);
    }
    Ok(LoadedState { patterns })
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

fn output_lines(_index: usize, _name: &str, _lines: &[Vec<Cell>]) {
    // println!("{index} {name}");
    // lines.iter().for_each(|line| {
    //     println!("{}", line.iter().map(|c| c.character_rep()).collect::<String>());
    // });
    // println!();
}

/// Returns the upper_index and the size of the reflection
fn find_reflection_indices(
    index: usize,
    name: &str,
    lines: &Vec<Vec<Cell>>,
) -> BTreeSet<UpperIndexAndSize> {
    output_lines(index, name, lines);
    let mut reflections = BTreeSet::default();
    for i in 1..lines.len() {
        let lower_line = &lines[i - 1];
        let upper_line = &lines[i];
        if lower_line == upper_line {
            if let Some(span) = find_reflection_size(lines, i) {
                reflections.insert((i, span));
            }
        }
    }
    reflections
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
    Reflection {
        rows: found_rows,
        columns: found_columns,
    }
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let row_columns = state
        .patterns
        .iter()
        .enumerate()
        .map(|(index, cells)| get_mirrored_row_columns(index, cells))
        .collect();
    Ok(row_columns)
}

fn flip_cell(cells: &mut Cells<Cell>, x: usize, y: usize) {
    let cell = cells.get_mut(x, y).unwrap();
    let flipped = match cell {
        Cell::Ash => Cell::Rock,
        Cell::Rock => Cell::Ash,
    };
    *cell = flipped;
}

fn fix_smudge_and_get_mirrored_row_columns(index: usize, cells: &mut Cells<Cell>) -> Reflection {
    let original = get_mirrored_row_columns(index, cells);
    let mut smudge_reflections: HashSet<Reflection> = HashSet::default();
    for x in 0..cells.side_lengths.0 {
        for y in 0..cells.side_lengths.1 {
            //Flip it
            flip_cell(cells, x, y);
            let smudge_reflection = get_mirrored_row_columns(index, cells);
            //Remember to flip it back
            flip_cell(cells, x, y);

            // println!("{}, {} original: {}", x, y, original);
            // println!("{}, {} smudged: {}", x, y, original);

            let new_columns = smudge_reflection
                .columns
                .difference(&original.columns)
                .cloned()
                .collect::<BTreeSet<_>>();
            let new_rows = smudge_reflection
                .rows
                .difference(&original.rows)
                .cloned()
                .collect::<BTreeSet<_>>();

            if new_columns.is_empty() && new_rows.is_empty() {
                continue;
            }
            if !new_columns.is_empty() && !new_rows.is_empty() {
                panic!("Got both new columns and new rows");
            }
            let new_reflection = Reflection {
                rows: new_rows,
                columns: new_columns,
            };
            // println!("{}, {} inserting: {}", x, y, original);
            smudge_reflections.insert(new_reflection);
        }
    }
    if smudge_reflections.len() != 1 {
        panic!("Found: {} at {index}", smudge_reflections.len());
    }
    smudge_reflections.into_iter().next().unwrap()
}

fn perform_processing_2(mut state: LoadedState) -> Result<ProcessedState, AError> {
    let row_columns = state
        .patterns
        .iter_mut()
        .enumerate()
        .map(|(index, cells)| fix_smudge_and_get_mirrored_row_columns(index, cells))
        .collect();
    Ok(row_columns)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    let values = state.iter().enumerate().map(|(_index, reflection)| {
        let (col_upper_index, _col_span) = reflection.columns.first().unwrap_or(&(0, 0));
        let (row_upper_index, _row_span) = reflection.rows.first().unwrap_or(&(0, 0));
        row_upper_index * 100 + col_upper_index
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
        perform_processing_1,
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
        perform_processing_2,
        calc_result,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}
