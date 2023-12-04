use std::{
    collections::{HashSet, VecDeque},
    rc::Rc,
};

use once_cell::sync::Lazy;
use processor::{process, read_i64, read_u64, read_word};

type AError = anyhow::Error;
type InitialState = Vec<Rc<Card>>;
type LoadedState = InitialState;
type ProcessedState = u64;
type FinalResult = ProcessedState;

#[derive(Debug, Clone)]
struct Card {
    _card_number: u64,
    card_index: usize,
    winning_numbers: HashSet<i64>,
    numbers: HashSet<i64>,
}

impl Card {
    fn num_matching(&self) -> usize {
        self.numbers
            .iter()
            .filter(|number| self.winning_numbers.contains(*number))
            .count()
    }

    fn calculate_points(&self) -> u64 {
        let num_matching = self.num_matching();
        if num_matching == 0 {
            return 0;
        }
        2u64.pow(num_matching as u32 - 1)
    }
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
        finalise_state,
        perform_processing_2,
        calc_result,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ', ':']));

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    //Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
    let mut chars = line.chars();
    if let Some(_card) = read_word(&mut chars, &DELIMITERS) {
        let (card_number, _) = read_u64(&mut chars, &DELIMITERS)?;

        let mut winning_numbers: HashSet<i64> = HashSet::new();
        let mut done_winning = false;
        while !done_winning {
            let number_and_delim = read_i64(&mut chars, &DELIMITERS);
            done_winning = match number_and_delim {
                Ok((number, _delimiter)) => {
                    winning_numbers.insert(number);
                    false
                }
                Err(_) => true,
            }
        }

        let mut numbers: HashSet<i64> = HashSet::new();
        let mut done_numbers = false;
        while !done_numbers {
            let number_and_delim = read_i64(&mut chars, &DELIMITERS);
            done_numbers = match number_and_delim {
                Ok((number, _delimiter)) => {
                    numbers.insert(number);
                    false
                }
                Err(_) => true,
            }
        }

        state.push(Rc::new(Card {
            _card_number: card_number,
            card_index: state.len(),
            winning_numbers,
            numbers,
        }))
    }
    Ok(state)
}

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
    // for card in state.iter() {
    //     println!("Card {}, points: {}", card.card_number, card.calculate_points());
    // }
    Ok(state)
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let total_points = state.iter().map(|card| card.calculate_points()).sum();
    Ok(total_points)
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let mut cards_won: u64 = 0;
    let mut to_process: VecDeque<Rc<Card>> = state.iter().fold(VecDeque::new(), |mut acc, card| {
        acc.push_front(Rc::clone(card));
        acc
    });

    while !to_process.is_empty() {
        if let Some(card) = to_process.pop_front() {
            cards_won += 1;
            let num_to_add = card.num_matching();
            for i in 0..num_to_add {
                if let Some(card_to_add) = state.get(card.card_index + i + 1) {
                    to_process.push_back(Rc::clone(card_to_add));
                }
            }
        }
    }

    Ok(cards_won)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}
