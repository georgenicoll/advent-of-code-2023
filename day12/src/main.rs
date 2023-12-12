use std::{collections::{HashSet, HashMap}, fmt::Display};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use processor::{process, read_word, read_next};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Condition{
    Operational,
    Damaged,
    Unknown,
}

impl Condition {
    fn to_char(&self) -> char {
        match self {
            Condition::Operational => '.',
            Condition::Damaged => '#',
            Condition::Unknown => '?',
        }
    }
}

impl Display for Condition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Debug)]
struct Line {
    conditions: Vec<Condition>,
    group_lengths: Vec<usize>,
}

type AError = anyhow::Error;
type InitialState = Vec<Line>;
type LoadedState = InitialState;
type ProcessedState = Vec<usize>;
type FinalResult = usize;

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(||
    HashSet::from([' ', ','])
);

fn parse_condition_line(line: &str) -> Result<Line, AError> {
    let mut chars = line.chars();
    let (conditions, _) = read_word(&mut chars, &DELIMITERS).ok_or_else(|| anyhow!("No Conditions"))?;
    let conditions = conditions.chars().into_iter().map(|c| match c {
        '.' => Condition::Operational,
        '#' => Condition::Damaged,
        '?' => Condition::Unknown,
        _ => panic!("Unknown condition: {c}"),
    }).collect();
    let mut group_lengths = Vec::default();
    while let Ok((group_length, _)) = read_next::<usize>(&mut chars, &DELIMITERS) {
        group_lengths.push(group_length);
    }
    Ok(Line {
        conditions,
        group_lengths
    })
}

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    state.push(parse_condition_line(&line)?);
    Ok(state)
}

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
    Ok(state)
}

fn expand_line(line: &mut Line) -> Line {
    line.conditions.push(Condition::Unknown);
    let mut repeated = line.conditions.repeat(5);
    repeated.remove(repeated.len() - 1);

    let repeated_lengths = line.group_lengths.repeat(5);

    Line {
        conditions: repeated,
        group_lengths: repeated_lengths,
    }
}

fn finalise_state_2(mut state: InitialState) -> Result<LoadedState, AError> {
    Ok(state.iter_mut().map(expand_line).collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ResultKey {
    arrange_from: usize,
    group_from: usize,
}

fn calculate_arrangements_from(to_arrange: &Vec<Condition>,
                               arrange_from: usize,
                               group_lengths: &Vec<usize>,
                               group_from: usize,
                               memoized: &mut HashMap<ResultKey, usize>) -> (bool, usize) {
    let result_key = ResultKey {
        arrange_from,
        group_from
    };
    if let Some(result) = memoized.get(&result_key) {
        return (true, *result);
    }

    // println!("==> Remaining: {}, {}",
    //     to_arrange.iter().skip(arrange_from).map(|c| c.to_char()).collect::<String>(),
    //     group_lengths.iter().skip(group_from).map(|l| l.to_string() + ",").collect::<String>()
    // );

    if group_from >= group_lengths.len() {
        //no more groups... following must be just non-damaged to be valid
        if to_arrange[arrange_from..].iter().all(|c| *c != Condition::Damaged) {
            // println!("==> Found: {}", to_arrange.iter().skip(arrange_from).map(|c| c.to_char()).collect::<String>());
            return (false, 1)
        } else {
            return (false, 0)
        }
    }

    let group_length = group_lengths[group_from];
    let remaining_to_fill = to_arrange.len() - arrange_from;

    //Too few left to fill the group?
    if group_length > remaining_to_fill {
        return (false, 0)
    }

    if to_arrange[arrange_from..(arrange_from + group_length)].iter().all(|c| *c != Condition::Operational) {
        //fillable items for all of the group length
        let after_group_position = arrange_from + group_length;
        let item_after_group = if after_group_position < to_arrange.len() {
            Some(to_arrange[after_group_position])
        } else {
            None
        };
        //fill the group right now, if we can
        let arrangements1 = if let Some((was_memoized, arrangements, next_arrange_from, next_group_from)) = match item_after_group {
            Some(Condition::Damaged) => {
                //followed by a damaged, not fillable, do nothing - we'll attempt moving to the next character in the arrangement 2 part
                None
            },
            Some(_) => {
                //either an operational or an uknown - fill group up to the position and continue after the operational
                let (was_memoized, arrangements) =
                    calculate_arrangements_from(to_arrange, after_group_position + 1, group_lengths, group_from + 1, memoized);
                Some((was_memoized, arrangements, after_group_position + 1, group_from + 1))
            },
            None => {
                //nothing after the position, we can just fill it and 'continue' after it
                let (was_memoized, arrangements) =
                    calculate_arrangements_from(to_arrange, after_group_position, group_lengths, group_from + 1, memoized);
                Some((was_memoized, arrangements, after_group_position, group_from + 1))
            }
        } {
            if !was_memoized {
                memoized.insert(ResultKey {
                    arrange_from: next_arrange_from,
                    group_from: next_group_from,
                }, arrangements);
            };
            arrangements
        } else {
            0
        };
        //and if the start of the group is an unknown, also try to fill from after it
        let arrangements2 = if to_arrange[arrange_from] == Condition::Unknown {
            let (was_memoized, arrangements) =
                calculate_arrangements_from(to_arrange, arrange_from + 1, group_lengths, group_from, memoized);
            if !was_memoized {
                memoized.insert(ResultKey {
                    arrange_from: arrange_from + 1,
                    group_from,
                }, arrangements);
            };
            arrangements
        } else {
            0
        };
        (false, arrangements1 + arrangements2)
    } else {
        //group is not fillable here, progress to the operational and try again after that
        let mut operational_position = arrange_from;
        while to_arrange[operational_position] != Condition::Operational {
            operational_position += 1
        }
        let (was_memoized, count) = calculate_arrangements_from(to_arrange, operational_position + 1, group_lengths, group_from, memoized);
        if !was_memoized {
            memoized.insert(ResultKey {
                arrange_from: operational_position + 1,
                group_from,
            }, count);
        }
        return (false, count)
    }
}

fn is_possible_arrangement(to_arrange: &Vec<Condition>, group_lengths: &Vec<usize>, final_check: bool) -> bool {
    let mut arrangement_lengths = Vec::default();
    let mut current_length = 0usize;
    for condition in to_arrange {
        match condition {
            Condition::Damaged => current_length += 1,
            Condition::Operational => {
                if current_length > 0 {
                    arrangement_lengths.push(current_length)
                }
                current_length = 0;
            },
            Condition::Unknown => break,
        }
    }
    if current_length > 0 {
        arrangement_lengths.push(current_length)
    }
    //If we have more arrangement lengths then this is out
    if arrangement_lengths.len() > group_lengths.len() {
        return false;
    }
    //If this is the final one, then they need to be the same length
    if final_check && arrangement_lengths.len() != group_lengths.len() {
        return false;
    }
    let is_possible = arrangement_lengths.iter().zip(group_lengths).enumerate().all(|(index, (arrangement_length, group_length))| {
        //If this is the final one then the lengths must match, otherwise we can be a bit lenient on the last one
        if !final_check && index == arrangement_lengths.len() - 1 {
            arrangement_length <= group_length
        } else {
            arrangement_length == group_length
        }
    });
    is_possible
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ArrangementKey {
    to_arrange: usize,
    group_lengths: usize,
}

fn construct_arrangement_key(to_arrange: &Vec<Condition>, last_flipped: usize, flipped_to: Condition, group_lengths: &Vec<usize>) -> ArrangementKey {
    ArrangementKey {
        to_arrange: to_arrange.len(),
        last_flipped,
        last_flipped_to: flipped_to,
        group_lengths_length: group_lengths.len(),
    }
}

fn calculate_arrangements(to_arrange: &Vec<Condition>,
                          last_flipped: usize,
                          last_flipped_to: Option<Condition>,
                          group_lengths: &Vec<usize>,
                          memoized: &mut HashMap<ArrangementKey, usize>) -> (bool, usize) {
    if let Some(flipped_to) = last_flipped_to {
        let key = construct_arrangement_key(to_arrange, last_flipped, flipped_to, group_lengths);
        if let Some(arrangements) = memoized.get(&key) {
            return (true, *arrangements);
        }
    }

    // println!("==> Arrangement: {}", to_arrange.iter().map(|c| c.to_char()).collect::<String>());
    if !is_possible_arrangement(to_arrange, group_lengths, false) {
        return (false, 0);
    }

    //Can we consume the first group?
    let (to_arrange, last_flipped, last_flipped_to, group_lengths) =
        if let Some(group_length) = group_lengths.first() {
        //drop any operationals and get the
        let group = to_arrange.iter().enumerate()
            .skip_while(|(_, c)| **c == Condition::Operational)
            .take_while(|(_, c)| **c == Condition::Damaged)
            .collect::<Vec<_>>();
        if group.len() == *group_length {
            let (last, _) = group.last().unwrap();
            if let Some(Condition::Operational) = to_arrange.get(last + 1) {
                (
                    to_arrange.iter().skip(last + 1).map(|c| *c).collect::<Vec<_>>(),
                    0,
                    None,
                    group_lengths.iter().skip(1).map(|s| *s).collect(),
                )
            } else {
                (to_arrange.to_vec(), last_flipped, last_flipped_to, group_lengths.to_vec())
            }
        } else {
            (to_arrange.to_vec(), last_flipped, last_flipped_to, group_lengths.to_vec())
        }
    } else {
        (to_arrange.to_vec(), last_flipped, last_flipped_to, group_lengths.to_vec())
    };

    //find next flippable and flip it
    if let Some((index_of_unknown, _)) = to_arrange.iter().enumerate().skip(last_flipped)
        .find(|(_, condition)| **condition == Condition::Unknown)
    {
        let mut to_arrange1 = to_arrange.to_vec();
        *to_arrange1.get_mut(index_of_unknown).unwrap() = Condition::Damaged;
        let (was_cached, arrangements1) = calculate_arrangements(&to_arrange1, index_of_unknown, Some(Condition::Damaged), &group_lengths, memoized);
        if !was_cached {
            let key = construct_arrangement_key(&to_arrange1, last_flipped, Condition::Damaged, &group_lengths);
            memoized.insert(key, arrangements1);
        }

        let mut to_arrange2 = to_arrange.to_vec();
        *to_arrange2.get_mut(index_of_unknown).unwrap() = Condition::Operational;
        let (was_cached, arrangements2) = calculate_arrangements(&to_arrange2, index_of_unknown, Some(Condition::Operational), &group_lengths, memoized);
        if !was_cached {
            let key = construct_arrangement_key(&to_arrange2, last_flipped, Condition::Operational, &group_lengths);
            memoized.insert(key, arrangements2);
        }

        (false, arrangements1 + arrangements2)
    } else {
        if is_possible_arrangement(&to_arrange, &group_lengths, true) {
            // println!("==> Found: {}", to_arrange.iter().map(|c| c.to_char()).collect::<String>());
            (false, 1)
        } else {
            (false, 0)
        }
    }
}

fn calculate_possible_arrangements(line: &Line) -> usize {
    //calculate_arrangements_from(&line.conditions, 0, &line.group_lengths, 0, &mut HashMap::default()).1
    let (_, result) = calculate_arrangements(&line.conditions, 0, None, &line.group_lengths, &mut HashMap::default());
    result
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let mut line_num = 0;
    Ok(state.iter().map(|line| {
        let result = calculate_possible_arrangements(&line);
        line_num += 1;
        println!("processed line {line_num}: {result}");
        result
    }).collect())
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state.iter().sum())
}

fn calc_result_2(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state.iter().sum())
}

fn main() {
    let file = "test-input.txt";
    //let file = "test-input2.txt";
    //let file = "input.txt";

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
        finalise_state_2,
        perform_processing,
        calc_result_2,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_1() {
        let line = parse_condition_line("???.### 1,1,3").unwrap();
        let arrangements = calculate_possible_arrangements(&line);
        assert_eq!(arrangements, 1);
    }

    #[test]
    fn test_line_2() {
        let line = parse_condition_line(".??..??...?##. 1,1,3").unwrap();
        let arrangements = calculate_possible_arrangements(&line);
        assert_eq!(arrangements, 4);
    }

    #[test]
    fn test_line_2_part_2() {
        let mut line = parse_condition_line(".??..??...?##. 1,1,3").unwrap();
        let line = expand_line(&mut line);
        let arrangements = calculate_possible_arrangements(&line);
        assert_eq!(arrangements, 16384);
    }

    #[test]
    fn test_line_5() {
        let line = parse_condition_line("????.######..#####. 1,6,5").unwrap();
        let arrangements = calculate_possible_arrangements(&line);
        assert_eq!(arrangements, 4);
    }

    #[test]
    fn test_real_line_6() {
        let line = parse_condition_line("????##?..??#?? 1,4,5").unwrap();
        let arrangements = calculate_possible_arrangements(&line);
        //assert_eq!(arrangements, 4);
    }

    #[test]
    fn test_last_line() {
        let line = parse_condition_line("?###???????? 3,2,1").unwrap();
        let arrangements = calculate_possible_arrangements(&line);
        assert_eq!(arrangements, 10);
    }

    #[test]
    fn test_slow_line() {
        let mut line = parse_condition_line(".#.??#???????.????# 1,3,1,1,1,4").unwrap();
        let line = expand_line(&mut line);
        let start_at = std::time::Instant::now();
        calculate_possible_arrangements(&line);
        println!("Took {}", start_at.elapsed().as_secs());
    }

}
