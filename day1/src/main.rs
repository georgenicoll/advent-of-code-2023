use std::collections::VecDeque;

use processor::{ok_identity, process};
use substring::{self, Substring};

type State = Vec<i64>;
type FinalState = i64;

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        Vec::new(),
        parse_line_1,
        ok_identity,
        perform_processing,
        ok_identity,
    );
    match result1 {
        Ok(res) => println!("Result 1: {}", res),
        Err(e) => println!("Error on 1: {}", e),
    }

    let result2 = process(
        file,
        Vec::new(),
        parse_line_2,
        ok_identity,
        perform_processing,
        ok_identity,
    );
    match result2 {
        Ok(res) => println!("Result 2: {}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

fn parse_line_1(mut state: State, line: String) -> Result<State, anyhow::Error> {
    let mut first: Option<i64> = None;
    let mut second: Option<i64> = None;
    for c in line.chars() {
        if let Some(d) = c.to_digit(10) {
            if first.is_none() {
                first = Some(d.into());
            }
            second = Some(d.into());
        }
    }
    let (a, b) = first
        .and_then(|a| second.map(|b| (a, b)))
        .ok_or(anyhow::Error::msg(format!(
            "Didn't get the 2 numbers on line: {}",
            line
        )))?;
    state.push(10 * a + b);
    Ok(state)
}

fn window_digit(window: &VecDeque<char>) -> Option<i64> {
    let window_string: String = window.iter().cloned().collect::<String>();
    let first_try = match window_string.substring(0, 1) {
        "1" => Some(1),
        "2" => Some(2),
        "3" => Some(3),
        "4" => Some(4),
        "5" => Some(5),
        "6" => Some(6),
        "7" => Some(7),
        "8" => Some(8),
        "9" => Some(9),
        _ => None,
    };
    let second_try = first_try.or_else(|| match window_string.substring(0, 3) {
        "one" => Some(1),
        "two" => Some(2),
        "six" => Some(6),
        _ => None,
    });
    let third_try = second_try.or_else(|| match window_string.substring(0, 4) {
        "four" => Some(4),
        "five" => Some(5),
        "nine" => Some(9),
        _ => None,
    });
    third_try.or_else(|| match window_string.substring(0, 5) {
        "three" => Some(3),
        "seven" => Some(7),
        "eight" => Some(8),
        _ => None,
    })
}

fn parse_line_2(mut state: State, line: String) -> Result<State, anyhow::Error> {
    let mut first: Option<i64> = None;
    let mut second: Option<i64> = None;

    let mut window: VecDeque<char> = VecDeque::new();
    let max_window_size = 5;

    'outer_forwards: {
        for char in line.chars() {
            window.push_back(char);
            if window.len() > max_window_size {
                window.pop_front();
            }
            let converted = window_digit(&window);
            if converted.is_some() {
                first = converted;
                break 'outer_forwards;
            }
        }
        while !window.is_empty() {
            window.pop_front();
            let converted = window_digit(&window);
            if converted.is_some() {
                first = converted;
                break 'outer_forwards;
            }
        }
    }

    'outer_backwards: {
        for char in line.chars().rev() {
            window.push_front(char);
            if window.len() > max_window_size {
                window.pop_back();
            }
            let converted = window_digit(&window);
            if converted.is_some() {
                second = converted;
                break 'outer_backwards;
            }
        }
        while !window.is_empty() {
            window.pop_back();
            let converted = window_digit(&window);
            if converted.is_some() {
                second = converted;
                break 'outer_backwards;
            }
        }
    }

    let (a, b) = first
        .and_then(|a| second.map(|b| (a, b)))
        .ok_or(anyhow::Error::msg(format!(
            "Didn't get the 2 numbers on line: {}",
            line
        )))?;
    state.push(10 * a + b);
    Ok(state)
}

fn perform_processing(state: State) -> Result<FinalState, anyhow::Error> {
    //println!("State: {:?}", state);
    Ok(state.iter().sum())
}
