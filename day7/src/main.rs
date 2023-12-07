use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Display,
};

use anyhow::Context;
use itertools::{
    FoldWhile::{Continue, Done},
    Itertools,
};
use once_cell::sync::Lazy;
use processor::{process, read_next, read_word};

type AError = anyhow::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
struct Card {
    name: char,
    strength: u8,
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

static ACE: Card = Card {
    name: 'A',
    strength: 14,
};
static KING: Card = Card {
    name: 'K',
    strength: 13,
};
static QUEEN: Card = Card {
    name: 'Q',
    strength: 12,
};
static JACK: Card = Card {
    name: 'J',
    strength: 11,
};
static TEN: Card = Card {
    name: 'T',
    strength: 10,
};
static NINE: Card = Card {
    name: '9',
    strength: 9,
};
static EIGHT: Card = Card {
    name: '8',
    strength: 8,
};
static SEVEN: Card = Card {
    name: '7',
    strength: 7,
};
static SIX: Card = Card {
    name: '6',
    strength: 6,
};
static FIVE: Card = Card {
    name: '5',
    strength: 5,
};
static FOUR: Card = Card {
    name: '4',
    strength: 4,
};
static THREE: Card = Card {
    name: '3',
    strength: 3,
};
static TWO: Card = Card {
    name: '2',
    strength: 2,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum HandType {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPair,
    OnePair,
    HighCard,
    NotCategorised,
}

#[derive(Debug)]
struct Hand {
    cards: Vec<Card>,
    bid: u64,
    hand_type: HandType,
}

impl Display for Hand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {:?}",
            self.cards.iter().join(""),
            self.bid,
            self.hand_type
        )
    }
}

type InitialState = Vec<Hand>;
type LoadedState = InitialState;
type ProcessedState = LoadedState;
type FinalResult = u64;

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        Vec::new(),
        parse_line,
        finalise_state_1,
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
        calc_result,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(|| HashSet::from([' ']));

fn convert_cards(cards: String) -> Vec<Card> {
    cards
        .chars()
        .map(|c| match c {
            'A' => ACE,
            'K' => KING,
            'Q' => QUEEN,
            'J' => JACK,
            'T' => TEN,
            '9' => NINE,
            '8' => EIGHT,
            '7' => SEVEN,
            '6' => SIX,
            '5' => FIVE,
            '4' => FOUR,
            '3' => THREE,
            '2' => TWO,
            _ => panic!("Unknown card: {}", c),
        })
        .collect()
}

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    let mut chars = line.chars();
    let (cards, _) = read_word(&mut chars, &DELIMITERS)
        .ok_or_else(|| anyhow::anyhow!(format!("No cards on line {}", state.len())))?;
    if cards.len() != 5 {
        return Err(anyhow::anyhow!(format!(
            "Wrong number of cards on line {}",
            line
        )));
    }

    let cards = convert_cards(cards);
    let (bid, _) = read_next::<u64>(&mut chars, &DELIMITERS)
        .with_context(|| anyhow::anyhow!(format!("Failed to read bid on line: {}", line)))?;

    state.push(Hand {
        cards,
        bid,
        hand_type: HandType::NotCategorised,
    });
    Ok(state)
}

fn categorize_hand(hand: &Hand, cards_grouped_lengths_most_to_least: &[usize]) -> HandType {
    match cards_grouped_lengths_most_to_least {
        [5] => HandType::FiveOfAKind,
        [4, 1] => HandType::FourOfAKind,
        [3, 2] => HandType::FullHouse,
        [3, 1, 1] => HandType::ThreeOfAKind,
        [2, 2, 1] => HandType::TwoPair,
        [2, 1, 1, 1] => HandType::OnePair,
        [1, 1, 1, 1, 1] => HandType::HighCard,
        _ => panic!("Failed to categorize hand {:?}", hand),
    }
}

fn categorize_hand_1(hand: &Hand) -> HandType {
    let cards_grouped: HashMap<char, Vec<Card>> =
        hand.cards.iter().fold(HashMap::new(), |mut acc, card| {
            let cards = acc.entry(card.name).or_default();
            cards.push(*card);
            acc
        });
    let mut cards_lengths = cards_grouped
        .values()
        .map(|cards| cards.len())
        .collect::<Vec<_>>();
    cards_lengths.sort();
    cards_lengths.reverse();
    categorize_hand(hand, &cards_lengths)
}

fn finalise_state_1(mut state: InitialState) -> Result<LoadedState, AError> {
    state.iter_mut().for_each(|hand| {
        hand.hand_type = categorize_hand_1(hand);
    });
    Ok(state)
}

fn update_jack_strength(hand: &Hand) -> Hand {
    let updated_cards = hand
        .cards
        .iter()
        .map(|card| {
            if card.name == 'J' {
                Card {
                    name: 'J',
                    strength: 1,
                }
            } else {
                *card
            }
        })
        .collect();
    Hand {
        cards: updated_cards,
        bid: hand.bid,
        hand_type: hand.hand_type,
    }
}

fn categorize_hand_2(hand: &Hand) -> HandType {
    let cards_non_joker: Vec<Card> = hand
        .cards
        .iter()
        .filter(|card| card.name != 'J')
        .cloned()
        .collect();
    let mut card_jokers: Vec<Card> = hand
        .cards
        .iter()
        .filter(|card| card.name == 'J')
        .cloned()
        .collect();

    let cards_grouped: HashMap<char, Vec<Card>> =
        cards_non_joker
            .iter()
            .fold(HashMap::new(), |mut acc, card| {
                let cards = acc.entry(card.name).or_default();
                cards.push(*card);
                acc
            });
    let mut cards_grouped = if cards_grouped.is_empty() {
        Vec::from([Vec::new()])
    } else {
        cards_grouped.into_values().collect::<Vec<_>>()
    };
    cards_grouped.sort_by(|cards1, cards2| {
        let length_sort = cards1.len().cmp(&cards2.len()).reverse();
        match length_sort {
            Ordering::Equal => {
                let card1 = cards1.get(0).unwrap();
                let card2 = cards2.get(0).unwrap();
                card1.strength.cmp(&card2.strength).reverse()
            }
            other => other,
        }
    });
    //Always add jokers to the first one
    cards_grouped.get_mut(0).unwrap().append(&mut card_jokers);
    let cards_lengths: Vec<usize> = cards_grouped.iter().map(|cards| cards.len()).collect();
    categorize_hand(hand, &cards_lengths)
}

fn finalise_state_2(state: InitialState) -> Result<LoadedState, AError> {
    let mut jacks_strength_updated: Vec<Hand> = state.iter().map(update_jack_strength).collect();
    jacks_strength_updated.iter_mut().for_each(|hand| {
        hand.hand_type = categorize_hand_2(hand);
    });
    Ok(jacks_strength_updated)
}

fn compare_cards(cards1: &[Card], cards2: &[Card]) -> Ordering {
    cards1
        .iter()
        .zip(cards2.iter())
        .fold_while(Ordering::Equal, |_latest, (card1, card2)| {
            match card1.strength.cmp(&card2.strength) {
                Ordering::Equal => Continue(Ordering::Equal),
                ordering => Done(ordering),
            }
        })
        .into_inner()
}

fn perform_processing(mut state: LoadedState) -> Result<ProcessedState, AError> {
    state.sort_by(|h1, h2| match h1.hand_type.cmp(&h2.hand_type) {
        Ordering::Equal => compare_cards(&h1.cards, &h2.cards),
        ordering => ordering.reverse(),
    });
    //state.iter().for_each(|hand| println!("{hand}"));
    Ok(state)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    let res = state
        .iter()
        .enumerate()
        .map(|(index, card)| (index as u64 + 1) * card.bid)
        .sum();
    Ok(res)
}
