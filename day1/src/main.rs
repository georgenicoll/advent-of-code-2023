use once_cell::sync::Lazy;
use processor::{ok_identity, process, reverse};
use regex::Regex;

type State = Vec<i64>;
type FinalState = i64;

fn main() {
    //let file = "day1/test-input.txt";
    //let file = "day1/test-input2.txt";
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

static RE_FORWARDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([1-9]|one|two|three|four|five|six|seven|eight|nine)").unwrap()
});
static RE_BACKWARDS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([1-9]|eno|owt|eerht|ruof|evif|xis|neves|thgie|enin)").unwrap()
});

fn parse_line_2(mut state: State, line: String) -> Result<State, anyhow::Error> {
    let mut first: Option<i64> = None;
    let mut second: Option<i64> = None;

    fn get_num(s: &str) -> Option<i64> {
        match s {
            "1" | "one" => Some(1),
            "2" | "two" => Some(2),
            "3" | "three" => Some(3),
            "4" | "four" => Some(4),
            "5" | "five" => Some(5),
            "6" | "six" => Some(6),
            "7" | "seven" => Some(7),
            "8" | "eight" => Some(8),
            "9" | "nine" => Some(9),
            _ => None,
        }
    }

    if let Some(m) = RE_FORWARDS.find(line.as_str()) {
        first = get_num(m.as_str())
    }

    let backwards_line: String = reverse(&line);
    if let Some(m) = RE_BACKWARDS.find(backwards_line.as_str()) {
        second = get_num(reverse(m.as_str()).as_str());
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
