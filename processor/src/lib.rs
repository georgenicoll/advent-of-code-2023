use std::{
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
    str::Chars,
};

use anyhow::Context;

type AError = anyhow::Error;
type Delimiter = char;

pub fn process<LoadState, State, ProcessedState, FinalResult>(
    file_name: &str,
    initial_state: LoadState,
    parse_line: fn(LoadState, String) -> Result<LoadState, AError>,
    finalise_state: fn(LoadState) -> Result<State, AError>,
    perform_processing: fn(State) -> Result<ProcessedState, AError>,
    calc_result: fn(ProcessedState) -> Result<FinalResult, AError>,
) -> Result<FinalResult, AError> {
    let file = File::open(file_name)?;
    let loaded_state = BufReader::new(file)
        .lines()
        .map(|l| l.unwrap())
        .try_fold(initial_state, parse_line)?;
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

// Read a word for the current positions of chars, advancing to the next non-delimiter and reading to the end
// or the next delimiter
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

pub fn read_int(
    chars: &mut Chars<'_>,
    delimiters: &HashSet<Delimiter>,
) -> Result<(i64, Option<Delimiter>), AError> {
    read_word(chars, delimiters)
        .ok_or_else(|| AError::msg("No word found to convert to integer"))
        .and_then(|word_and_delimiter| {
            let (word, delimiter) = word_and_delimiter;
            word.parse::<i64>()
                .map(|i| (i, delimiter))
                .context(format!("Failed to parse '{}' to integer", word))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
