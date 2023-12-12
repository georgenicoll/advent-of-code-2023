use std::{collections::{HashSet, HashMap}, fmt::Display};

use anyhow::anyhow;
use once_cell::sync::Lazy;
use processor::{process, read_word, read_next};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
                               mut arrange_from: usize,
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

    if arrange_from >= to_arrange.len() {
        if group_from >= group_lengths.len() {
            //println!("==> Found: {}", to_arrange.iter().skip(arrange_from).map(|c| c.to_char()).collect::<String>());
            return (false, 1);
        } else {
            return (false, 0);
        }
    }

    if group_from >= group_lengths.len() {
        if to_arrange[arrange_from..].iter().all(|condition| *condition != Condition::Damaged) {
            // println!("==> Found: {}", to_arrange.iter().skip(arrange_from).map(|c| c.to_char()).collect::<String>());
            return (false, 1);
        } else {
            return (false, 0);
        }
    }

    //consume any operational at the start
    while arrange_from < to_arrange.len() && to_arrange[arrange_from] == Condition::Operational {
        arrange_from += 1
    }
    if arrange_from >= to_arrange.len() {
        return (false, 0);
    }

    if group_from >= group_lengths.len() {
        //should only have operational or unknown in the remaining_to_process if we have no groups left to consume
        if to_arrange[arrange_from..].iter().all(|condition| *condition != Condition::Damaged) {
            // println!("==> Found: {}", to_arrange.iter().skip(arrange_from).map(|c| c.to_char()).collect::<String>());
            return (false, 1);
        } else {
            return (false, 0);
        }
    }

    let group_length = group_lengths[group_from];
    let is_last_group = group_from == group_lengths.len() - 1;
    let remaining_to_consume = to_arrange.len() - arrange_from;
    let remaining_groups = group_lengths.len() - group_from;

    //last group - will this fully consume?
    if is_last_group && remaining_to_consume == group_length {
        // println!("==> Found: {}", to_arrange.iter().skip(arrange_from).map(|c| c.to_char()).collect::<String>());
        return (false, 1);
    }

    //do we have enough space left (groups lengths plus operationals in-between)
    let space_required = group_lengths[group_from..].iter().sum::<usize>() + remaining_groups - 1;
    if remaining_to_consume < space_required {
        return (false, 0);
    }

    //need to have all non-operational in the space that we need to occupy
    if !to_arrange[arrange_from..(arrange_from + group_length)].iter().all(|condition| *condition != Condition::Operational) {
        //not all non-operationals - 'convert these to operationals' and try from the next non-operational
        let mut next_from = arrange_from;
        while next_from < to_arrange.len() && to_arrange[next_from] != Condition::Operational {
            next_from += 1;
        };
        //now consume any following operationals
        while next_from < to_arrange.len() && to_arrange[next_from] == Condition::Operational {
            next_from += 1;
        };
        //try from here
        if next_from > arrange_from {
            return calculate_arrangements_from(to_arrange, next_from, group_lengths, group_from, memoized);
        } else {
            return (false, 0);
        }
    }

    //last group - need to be all non-damaged after the group or we need an extra group (not possible) or a larger group (np)
    if remaining_groups == 1 &&
        !to_arrange[arrange_from + group_length..].iter().all(|condition| *condition != Condition::Damaged) {
        return (false, 0);
    };

    //Drop the group we are consuming
    let new_group_from = group_from + 1;
    let mut num_arrangements = 0;
    let arrange_from_after_group_at_start = arrange_from + group_length;
    //1st option, put the group right here
    // let remaining_after_group_at_start = Vec::from_iter(remaining_to_arrange.iter().skip(*group_length).map(|c| *c));
    {
        //group needs to be followed by n operationals
        let first_condition = to_arrange[arrange_from_after_group_at_start];
        if let Some((from, group_from, (was_memoized, result))) = match first_condition {
            Condition::Operational => {
                //consume to next non-operational
                let mut next_from = arrange_from_after_group_at_start;
                while next_from < to_arrange.len() && to_arrange[next_from] == Condition::Operational {
                    next_from += 1
                };
                Some((
                    next_from,
                    new_group_from,
                    calculate_arrangements_from(to_arrange, next_from, group_lengths, new_group_from, memoized)
                ))
            },
            Condition::Unknown => {
                //make the unknown an operational and consume it
                Some((
                    arrange_from_after_group_at_start + 1,
                    new_group_from,
                    calculate_arrangements_from(to_arrange, arrange_from_after_group_at_start + 1, group_lengths, new_group_from, memoized)
                ))
            },
            _ => None,
        } {
            if !was_memoized {
                memoized.insert(ResultKey {
                    arrange_from: from,
                    group_from
                }, result);
            }
            num_arrangements += result;
        }
    }
    //2nd option, if we have an unknown at the start, and a non-operational after the group, also try moving by 1 with the same groups
    if to_arrange[arrange_from] == Condition::Unknown &&
        !(to_arrange[arrange_from_after_group_at_start] == Condition::Operational) {
        let (was_memoized, result) = calculate_arrangements_from(to_arrange, arrange_from + 1, group_lengths, group_from, memoized);
        if !was_memoized {
            memoized.insert(ResultKey {
                arrange_from: arrange_from + 1,
                group_from
            }, result);
        }
        num_arrangements += result;
    }

    (false, num_arrangements)
}

fn calculate_possible_arrangements(line: &Line) -> usize {
    calculate_arrangements_from(&line.conditions, 0, &line.group_lengths, 0, &mut HashMap::default()).1
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    let mut line_num = 0;
    Ok(state.iter().map(|line| {
        let result = calculate_possible_arrangements(&line);
        line_num += 1;
        // println!("processed line {line_num}: {result}");
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
        assert_eq!(arrangements, 4);
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
