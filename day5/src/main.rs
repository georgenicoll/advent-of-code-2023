use std::{collections::HashSet, cmp::Ordering};

use once_cell::sync::Lazy;
use processor::{process, read_word, read_usize};

type Seeds = Vec<usize>;

#[derive(Debug)]
struct IndexMap {
    source_start: usize,
    destination_start: usize,
    length: usize,
}

type Mapping = Vec<IndexMap>;

#[derive(Debug)]
struct Mappings {
    seed_to_soil: Mapping,
    soil_to_fertilizer: Mapping,
    fertilizer_to_water: Mapping,
    water_to_light: Mapping,
    light_to_temperature: Mapping,
    temperature_to_humidity: Mapping,
    humidity_to_location: Mapping,
}

impl Mappings {
    fn new() -> Mappings {
        Mappings{
            seed_to_soil: Mapping::new(),
            soil_to_fertilizer: Mapping::new(),
            fertilizer_to_water: Mapping::new(),
            water_to_light: Mapping::new(),
            light_to_temperature: Mapping::new(),
            temperature_to_humidity: Mapping::new(),
            humidity_to_location: Mapping::new(),
        }
    }
}

#[derive(Debug)]
struct State {
    seeds: Seeds,
    mappings: Mappings,
}

enum LoadingState {
    Seeds,
    SeedToSoil,
    SoilToFertilizer,
    FertilizerToWater,
    WaterToLight,
    LightToTemperature,
    TemperatureToHumidity,
    HumidityToLocation,
}

type AError = anyhow::Error;
type InitialState = (LoadingState, State);
type LoadedState = State;
type ProcessedState = usize;
type FinalResult = ProcessedState;

fn main() {
    //let file = "test-input.txt";
    //let file = "test-input2.txt";
    let file = "input.txt";

    let result1 = process(
        file,
        (LoadingState::Seeds, State { seeds: Seeds::new(), mappings: Mappings::new() }),
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
        (LoadingState::Seeds, State { seeds: Seeds::new(), mappings: Mappings::new() }),
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

fn get_next_loading_state(state: LoadingState) -> LoadingState {
    match state {
        LoadingState::Seeds => LoadingState::SeedToSoil,
        LoadingState::SeedToSoil => LoadingState::SoilToFertilizer,
        LoadingState::SoilToFertilizer => LoadingState::FertilizerToWater,
        LoadingState::FertilizerToWater => LoadingState::WaterToLight,
        LoadingState::WaterToLight => LoadingState::LightToTemperature,
        LoadingState::LightToTemperature => LoadingState::TemperatureToHumidity,
        LoadingState::TemperatureToHumidity => LoadingState::HumidityToLocation,
        LoadingState::HumidityToLocation => panic!("HumidityToLocation expected to be last state"),
    }
}

static DELIMITERS: Lazy<HashSet<char>> = Lazy::new(||
    HashSet::from([' ', ':'])
);

fn load_seeds(seeds: &mut Seeds, line: String) {
    let mut chars = line.chars();
    let _seeds = read_word(&mut chars, &DELIMITERS).unwrap();
    let mut keep_reading = true;
    while keep_reading {
        keep_reading = match read_usize(&mut chars, &DELIMITERS) {
            Ok((seed, delimiter)) => {
                seeds.push(seed);
                delimiter.is_some()
            },
            Err(e) => {
                panic!("Unexpected read error while loading seeds on '{}': {}", line, e);
            }
        }
    }
}

fn load_mapping_line(mapping: &mut Mapping, line: String) {
    let mut chars = line.chars();
    match read_usize(&mut chars, &DELIMITERS) {
        Ok((destination_start, _)) => {
            let (source_start, _) = read_usize(&mut chars, &DELIMITERS).unwrap();
            let (length, _) = read_usize(&mut chars, &DELIMITERS).unwrap();
            mapping.push(IndexMap {
                source_start,
                destination_start,
                length,
            });
        },
        Err(_) => (),
    }
}

fn parse_line(istate: InitialState, line: String) -> Result<InitialState, AError> {
    let (loading_state, mut state) = istate;
    let next_loading_state = if line.trim().len() == 0 {
        get_next_loading_state(loading_state)
    } else {
        match loading_state {
            LoadingState::Seeds => load_seeds(&mut state.seeds, line),
            LoadingState::SeedToSoil => load_mapping_line(&mut state.mappings.seed_to_soil, line),
            LoadingState::SoilToFertilizer => load_mapping_line(&mut state.mappings.soil_to_fertilizer, line),
            LoadingState::FertilizerToWater => load_mapping_line(&mut state.mappings.fertilizer_to_water, line),
            LoadingState::WaterToLight => load_mapping_line(&mut state.mappings.water_to_light, line),
            LoadingState::LightToTemperature => load_mapping_line(&mut state.mappings.light_to_temperature, line),
            LoadingState::TemperatureToHumidity => load_mapping_line(&mut state.mappings.temperature_to_humidity, line),
            LoadingState::HumidityToLocation => load_mapping_line(&mut state.mappings.humidity_to_location, line),
        }
        loading_state
    };
    Ok((next_loading_state, state))
}

fn finalise_state(istate: InitialState) -> Result<LoadedState, AError> {
    let (_, mut state) = istate;
    fn source_first(map1: &IndexMap, map2: &IndexMap) -> Ordering {
        map1.source_start.cmp(&map2.source_start)
    }
    state.mappings.seed_to_soil.sort_by(source_first);
    state.mappings.soil_to_fertilizer.sort_by(source_first);
    state.mappings.fertilizer_to_water.sort_by(source_first);
    state.mappings.water_to_light.sort_by(source_first);
    state.mappings.light_to_temperature.sort_by(source_first);
    state.mappings.temperature_to_humidity.sort_by(source_first);
    state.mappings.humidity_to_location.sort_by(source_first);
    Ok(state)
}

//Assuming sorted
fn get_destination(source: usize, mapping: &Mapping) -> usize {
    for index_map in mapping.iter() {
        if index_map.source_start > source {
            break; //before the index map
        }
        if source > index_map.source_start + index_map.length - 1 {
            continue; //try the next one, we're after the index map
        }
        //In the index map
        return index_map.destination_start + (source - index_map.source_start);
    }
    //None found - use the same as source
    source
}

fn calculate_location(seed: &usize, mappings: &Mappings) -> usize {
    let soil = get_destination(*seed, &mappings.seed_to_soil);
    let fertilizer = get_destination(soil, &mappings.soil_to_fertilizer);
    let water = get_destination(fertilizer, &mappings.fertilizer_to_water);
    let light = get_destination(water, &mappings.water_to_light);
    let temp = get_destination(light, &mappings.light_to_temperature);
    let humidity = get_destination(temp, &mappings.temperature_to_humidity);
    let location = get_destination(humidity, &mappings.humidity_to_location);
    location
}

fn perform_processing_1(state: LoadedState) -> Result<ProcessedState, AError> {
    let minimum = state.seeds.iter().fold(usize::MAX, |acc, seed| {
        let location = calculate_location(seed, &state.mappings);
        location.min(acc)
    });
    Ok(minimum)
}

//FIXME: Should be able to make this much more efficient
fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let minimum = state.seeds.chunks_exact(2).fold(usize::MAX, |min_so_far, start_length| {
        let mut minimum = min_so_far;
        let start = start_length[0];
        let length = start_length[1];
        for seed in start..(start + length) {
            let location = calculate_location(&seed, &state.mappings);
            minimum = minimum.min(location);
        }
        minimum
    });
    Ok(minimum)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}
