use std::{
    collections::HashSet,
    error::Error,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    str::{Chars, FromStr},
};

use anyhow::Context;
use num::ToPrimitive;
use once_cell::sync::Lazy;

type AError = anyhow::Error;
type Delimiter = char;

pub static BLANK_DELIMITERS: Lazy<HashSet<Delimiter>> = Lazy::new(HashSet::default);

pub fn process<LoadState, State, ProcessedState, FinalResult>(
    file_name: &str,
    initial_state: LoadState,
    parse_line: fn(LoadState, String) -> Result<LoadState, AError>,
    finalise_state: fn(LoadState) -> Result<State, AError>,
    perform_processing: fn(State) -> Result<ProcessedState, AError>,
    calc_result: fn(ProcessedState) -> Result<FinalResult, AError>,
) -> Result<FinalResult, AError> {
    let loaded_state = {
        let file = File::open(file_name)?;
        BufReader::new(file)
            .lines()
            .map(|l| l.unwrap())
            .try_fold(initial_state, parse_line)?
    };
    let finalised_state = finalise_state(loaded_state)?;
    let processed_state = perform_processing(finalised_state)?;
    calc_result(processed_state)
}

pub fn ok_identity<T>(t: T) -> Result<T, AError> {
    Ok(t)
}

pub fn reverse(s: &str) -> String {
    //assume no graphemes - use unicode_segmentation if this is not the case
    s.chars().rev().collect()
}

/// Read a word for the current positions of chars, advancing to the next non-delimiter and reading to the end
/// or the next delimiter
pub fn read_word(
    chars: &mut Chars<'_>,
    delimiters: &HashSet<Delimiter>,
) -> Option<(String, Option<Delimiter>)> {
    let mut consumed: Vec<char> = Vec::new();
    let mut next: Option<char> = Some('/');
    while next.is_some() {
        next = chars.next();
        if next.is_none() {
            break;
        }
        let c = next.unwrap();
        if delimiters.contains(&c) {
            if consumed.is_empty() {
                continue;
            } else {
                break;
            }
        }
        consumed.push(c);
    }
    if consumed.is_empty() {
        None
    } else {
        Some((consumed.iter().collect(), next))
    }
}

/// Read the next word and parse it to a type implementing FromStr
pub fn read_next<T>(
    chars: &mut Chars<'_>,
    delimiters: &HashSet<Delimiter>,
) -> Result<(T, Option<Delimiter>), AError>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
{
    read_word(chars, delimiters)
        .ok_or_else(|| AError::msg("No word found to convert to integer"))
        .and_then(|word_and_delimiter| {
            let (word, delimiter) = word_and_delimiter;
            word.parse::<T>()
                .map(|t| (t, delimiter))
                .with_context(|| format!("Failed parsing word: '{}'", word))
        })
}

/// Get coords adjacent to the given centre, including diagonals, excluding any coords that would be outside the side lengths.
/// This will only return actual coordinates (i.e. if the centre is at an edge coords over the edge will not be returned).
fn adjacent_coords(
    centre: &(usize, usize),
    side_lengths: &(usize, usize),
    deltas: &[(i8, i8)],
) -> Vec<(usize, usize)> {
    deltas
        .iter()
        .map(|(delta_x, delta_y)| {
            (
                centre.0 as isize + *delta_x as isize,
                centre.1 as isize + *delta_y as isize,
            )
        })
        .filter(|(x, y)| *x >= 0 && *y >= 0)
        .map(|(x, y)| (x as usize, y as usize))
        .filter(|(x, y)| *x < side_lengths.0 && *y < side_lengths.1)
        .collect()
}

static ADJACENT_DELTAS_DIAGONAL: Lazy<Vec<(i8, i8)>> = Lazy::new(|| {
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

pub fn adjacent_coords_diagonal(
    centre: &(usize, usize),
    side_lengths: &(usize, usize),
) -> Vec<(usize, usize)> {
    adjacent_coords(centre, side_lengths, &ADJACENT_DELTAS_DIAGONAL)
}

static ADJACENT_DELTAS_CARTESION: Lazy<Vec<(i8, i8)>> =
    Lazy::new(|| Vec::from([(0, -1), (-1, 0), (1, 0), (0, 1)]));

pub fn adjacent_coords_cartesian(
    centre: &(usize, usize),
    side_lengths: &(usize, usize),
) -> Vec<(usize, usize)> {
    adjacent_coords(centre, side_lengths, &ADJACENT_DELTAS_CARTESION)
}

/// Represents an n * m block of data
#[derive(Debug, Clone)]
pub struct Cells<T> {
    contents: Vec<T>,
    pub side_lengths: (usize, usize),
}

impl<T> Cells<T> {
    /// Checks whether the input can be represented as coordinates (i.e. can be converted to usize)
    /// and that the values are within the range of the cells' sides
    pub fn in_bounds<N>(&self, x: N, y: N) -> bool
    where
        N: ToPrimitive,
    {
        match (x.to_usize(), y.to_usize()) {
            (Some(x), Some(y)) => x < self.side_lengths.0 && y < self.side_lengths.1,
            _ => false,
        }
    }

    #[inline]
    fn calculate_index(&self, x: usize, y: usize) -> usize {
        y * self.side_lengths.0 + x
    }

    pub fn get(&self, x: usize, y: usize) -> Result<&T, AError> {
        if !self.in_bounds(x, y) {
            return Err(AError::msg(format!("({}, {}) is not in bounds", x, y)));
        }
        let index = self.calculate_index(x, y);
        let cell = self
            .contents
            .get(index)
            .ok_or_else(|| AError::msg(format!("No cell value found at ({x}, {y})")))?;
        Ok(cell)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Result<&mut T, AError> {
        if !self.in_bounds(x, y) {
            return Err(AError::msg(format!("({}, {}) is not in bounds", x, y)));
        }
        let index = self.calculate_index(x, y);
        let cell = self
            .contents
            .get_mut(index)
            .ok_or_else(|| AError::msg(format!("No cell value found at ({x}, {y})")))?;
        Ok(cell)
    }

    pub fn iter(&self) -> CellsIter<T> {
        CellsIter {
            x: 0,
            y: 0,
            cells: self,
        }
    }

    pub fn swap(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) -> Result<(), AError> {
        if !self.in_bounds(x1, y1) {
            return Err(AError::msg(format!(
                "First ({}, {}) is not in bounds",
                x1, y1
            )));
        }
        if !self.in_bounds(x2, y2) {
            return Err(AError::msg(format!(
                "Second ({}, {}) is not in bounds",
                x2, y2
            )));
        }
        let index1 = self.calculate_index(x1, y1);
        let index2 = self.calculate_index(x2, y2);
        self.contents.swap(index1, index2);
        Ok(())
    }
}

impl <T: Clone> Cells<T> {
    pub fn with_dimension(width: usize, height: usize, initial_value: T) -> Cells<T> {
        let mut contents = Vec::with_capacity(width * height);
        contents.resize_with(width * height, || initial_value.clone());
        Cells {
            contents,
            side_lengths: (width, height),
        }
    }
}

pub struct CellsIter<'a, T> {
    x: usize,
    y: usize,
    cells: &'a Cells<T>,
}

impl<'a, T> Iterator for CellsIter<'a, T> {
    type Item = ((usize, usize), &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.y * self.cells.side_lengths.0 + self.x;
        if index >= self.cells.contents.len() {
            return None;
        }
        let coord = (self.x, self.y);
        if self.x == self.cells.side_lengths.0 - 1 {
            self.y += 1;
            self.x = 0;
        } else {
            self.x += 1;
        }
        let cell = self.cells.contents.get(index);
        cell.map(|c| (coord, c))
    }
}

impl<T: Display> Display for Cells<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.side_lengths.1 {
            for x in 0..self.side_lengths.0 {
                let cell = self.get(x, y).unwrap();
                write!(f, "{cell}")?
            }
            writeln!(f)?
        }
        write!(f, "")
    }
}

/// Represents a builder for a block/table of data
#[derive(Debug, Default)]
pub struct CellsBuilder<T> {
    lines: Vec<Vec<T>>,
    max_width: usize,
}

impl<T> CellsBuilder<T> {
    pub fn new_empty() -> Self {
        CellsBuilder {
            lines: Vec::new(),
            max_width: 0,
        }
    }

    pub fn new_line(&mut self) {
        self.lines.push(Vec::new());
    }

    pub fn add_cell(&mut self, cell: T) -> Result<(), AError> {
        let current_line = self
            .lines
            .last_mut()
            .ok_or_else(|| AError::msg("Cannot add a cell when no line has been added"))?;
        current_line.push(cell);
        self.max_width = self.max_width.max(current_line.len());
        Ok(())
    }

    pub fn get(&self, x: usize, y: usize) -> Result<&T, AError> {
        let line = self
            .lines
            .get(y)
            .ok_or_else(|| AError::msg(format!("No line for y={y} created yet")))?;
        let cell = line
            .get(x)
            .ok_or_else(|| AError::msg(format!("No cell at ({x}, {y}) yet")))?;
        Ok(cell)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Result<&mut T, AError> {
        let line = self
            .lines
            .get_mut(y)
            .ok_or_else(|| AError::msg(format!("No line for y={y} created yet")))?;
        let cell = line
            .get_mut(x)
            .ok_or_else(|| AError::msg(format!("No cell at ({x}, {y}) yet")))?;
        Ok(cell)
    }

    pub fn build_cells(&mut self, default_value: T) -> Result<Cells<T>, AError>
    where
        T: Clone,
    {
        if self.lines.is_empty() {
            return Err(AError::msg(
                "No point in building cells when there are no lines",
            ));
        }
        if self.max_width == 0 {
            return Err(AError::msg(
                "No point in building cells when the width is 0",
            ));
        }

        let lines = std::mem::take(&mut self.lines);
        let height = lines.len();

        let contents = lines.into_iter().enumerate().fold(
            Vec::with_capacity(height * self.max_width),
            |mut acc, (y, mut line)| {
                let start_index = y * self.max_width;
                for cell in line.drain(..) {
                    acc.push(cell);
                }
                let expected_number_after_this_line = start_index + self.max_width;
                while acc.len() < expected_number_after_this_line {
                    acc.push(default_value.clone());
                }
                acc
            },
        );
        Ok(Cells {
            contents,
            side_lengths: (self.max_width, height),
        })
    }

    pub fn current_cell(&self) -> Option<(usize, usize)> {
        if self.lines.is_empty() {
            return None;
        }
        let line = self.lines.last().unwrap();
        if line.is_empty() {
            return None;
        }
        Some((line.len() - 1, self.lines.len() - 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_in_bounds() {
        let mut builder: CellsBuilder<char> = CellsBuilder::new_empty();
        builder.new_line();
        builder.add_cell('.').unwrap();
        let cells = builder.build_cells('.').unwrap();
        assert!(cells.in_bounds(0, 0));
        assert!(!cells.in_bounds(1, 0));
        assert!(!cells.in_bounds(0, 1));
        assert!(!cells.in_bounds(-1, 0));
        assert!(!cells.in_bounds(0, -1));
        assert!(!cells.in_bounds(-1, -1));
    }

    #[test]
    fn load_file() {
        let initial_state: Vec<String> = Vec::new();
        let res = process(
            "test-input.txt",
            initial_state,
            |mut vec, line| {
                vec.push(line);
                Ok(vec)
            },
            ok_identity,
            |vec| Ok(vec.join("+")),
            ok_identity,
        );
        match res {
            Ok(message) => assert_eq!(message, "Some Input Here+It's Good".to_string()),
            Err(e) => panic!("{}", e),
        }
    }

    #[test]
    fn build_cells() {
        //Arrange
        let expected_values: Vec<[((usize, usize), char); 3]> = vec![
            [((0, 0), 'a'), ((1, 0), 'b'), ((2, 0), 'c')],
            [((0, 1), '1'), ((1, 1), '2'), ((2, 1), '?')],
            [((0, 2), '-'), ((1, 2), '.'), ((2, 2), '+')],
        ];
        //Act
        let mut builder: CellsBuilder<char> = CellsBuilder::new_empty();
        for line_vals in expected_values.iter() {
            builder.new_line();
            for ((_, _), value) in line_vals {
                if *value != '?' {
                    builder.add_cell(value.clone()).unwrap();
                }
            }
        }
        let cells = builder.build_cells('?').unwrap();
        //Assert
        assert_eq!((3, 3), cells.side_lengths);
        for line_vals in expected_values.iter() {
            for ((x, y), expected) in line_vals.iter() {
                assert_eq!(*cells.get(*x, *y).unwrap(), *expected);
            }
        }
    }

    #[test]
    fn edit_cells() {
        let mut builder: CellsBuilder<char> = CellsBuilder::new_empty();
        builder.new_line();
        builder.add_cell('a').unwrap();
        let mut cells = builder.build_cells('?').unwrap();
        {
            let value = cells.get_mut(0, 0).unwrap();
            assert_eq!(*value, 'a');
            *value = 'b';
        }
        assert_eq!(*cells.get(0, 0).unwrap(), 'b');
    }

    #[test]
    fn iter_works() {
        let mut builder: CellsBuilder<char> = CellsBuilder::new_empty();
        builder.new_line();
        builder.add_cell('a').unwrap();
        builder.add_cell('b').unwrap();
        builder.new_line();
        builder.add_cell('c').unwrap();
        builder.add_cell('d').unwrap();

        let cells = builder.build_cells('?').unwrap();

        let items = cells.iter().fold(
            Vec::new(),
            |mut acc: Vec<((usize, usize), &char)>, ((x, y), c)| {
                acc.push(((x, y), c));
                acc
            },
        );

        let expected: Vec<((usize, usize), char)> =
            vec![((0, 0), 'a'), ((1, 0), 'b'), ((0, 1), 'c'), ((1, 1), 'd')];

        assert_eq!(items.len(), expected.len());
        for (index, exp) in expected.iter().enumerate() {
            let actual = items.get(index).unwrap();
            assert_eq!(actual.0, exp.0);
            assert_eq!(*actual.1, exp.1);
        }
    }

    static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from(['@']));

    #[test]
    fn read_next_works() {
        let s = "57";
        assert_eq!(
            read_next::<u64>(&mut s.chars(), &DELIMITERS).unwrap(),
            (57u64, None)
        );
        assert_eq!(
            read_next::<i64>(&mut s.chars(), &DELIMITERS).unwrap(),
            (57i64, None)
        );
        assert_eq!(
            read_next::<u32>(&mut s.chars(), &DELIMITERS).unwrap(),
            (57u32, None)
        );
        assert_eq!(
            read_next::<usize>(&mut s.chars(), &DELIMITERS).unwrap(),
            (57usize, None)
        );
    }
}
