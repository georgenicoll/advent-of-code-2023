use std::{collections::HashSet, cmp::Ordering};

use once_cell::sync::Lazy;
use processor::{process, read_word, read_next};

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
        keep_reading = match read_next::<usize>(&mut chars, &DELIMITERS) {
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
    match read_next::<usize>(&mut chars, &DELIMITERS) {
        Ok((destination_start, _)) => {
            let (source_start, _) = read_next::<usize>(&mut chars, &DELIMITERS).unwrap();
            let (length, _) = read_next::<usize>(&mut chars, &DELIMITERS).unwrap();
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

fn add_destination_ranges(start: usize, length: usize, mapping: &Mapping, destination_ranges: &mut Vec<(usize, usize)>) {
    let mut length_remaining = length;
    let mut current_index = start;

    let mut mapping_iter = mapping.iter();
    let mut current_index_map = mapping_iter.next();

    while length_remaining > 0 && current_index_map.is_some() {
        let index_map = current_index_map.unwrap();
        let last_index = current_index + length_remaining - 1;
        let last_map_index = index_map.source_start + index_map.length - 1;
        //everything before the index_map, if so drop out
        if last_index < index_map.source_start {
            break;
        }
        //are we after the index map? Move to the next one
        if current_index > last_map_index {
            current_index_map = mapping_iter.next();
            continue;
        }
        //anything before the index map?
        if current_index < index_map.source_start {
            //something is in the index map - add a range up to the map and adjust
            let length_to_consume = index_map.source_start - current_index;
            destination_ranges.push((current_index, length_to_consume));
            current_index = index_map.source_start;
            length_remaining -= length_to_consume;
            continue;
        }
        //must be in the index map then
        let next_index = last_index.min(last_map_index) + 1;
        let length_to_consume = next_index - current_index;
        let destination_index = index_map.destination_start + (current_index - index_map.source_start);
        destination_ranges.push((destination_index, length_to_consume));
        current_index = next_index;
        length_remaining -= length_to_consume;
    }

    if length_remaining > 0 {
        destination_ranges.push((current_index, length_remaining));
    }
}

fn get_destination_ranges(source_ranges: Vec<(usize, usize)>, mapping: &Mapping) -> Vec<(usize, usize)> {
    let mut destination_ranges = Vec::new();
    for (start, length) in source_ranges {
        add_destination_ranges(start, length, mapping, &mut destination_ranges);
    }
    destination_ranges
}

fn get_location_ranges(start_seed: usize, length: usize, mappings: &Mappings) -> Vec<(usize, usize)> {
    let soil_ranges = get_destination_ranges(Vec::from([(start_seed, length)]), &mappings.seed_to_soil);
    let fertilizer_ranges = get_destination_ranges(soil_ranges, &mappings.soil_to_fertilizer);
    let water_ranges = get_destination_ranges(fertilizer_ranges, &mappings.fertilizer_to_water);
    let light_ranges = get_destination_ranges(water_ranges, &mappings.water_to_light);
    let temperature_ranges = get_destination_ranges(light_ranges, &mappings.light_to_temperature);
    let humidity_ranges = get_destination_ranges(temperature_ranges, &mappings.temperature_to_humidity);
    get_destination_ranges(humidity_ranges, &mappings.humidity_to_location)
}

fn perform_processing_2(state: LoadedState) -> Result<ProcessedState, AError> {
    let minimum = state.seeds.chunks_exact(2).fold(usize::MAX, |min_so_far, start_length| {
        let start = start_length[0];
        let length = start_length[1];
        let location_ranges = get_location_ranges(start, length, &state.mappings);
        location_ranges.iter().fold(min_so_far, |min, (start, _)| min.min(*start))
    });
    Ok(minimum)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_before_any_index_map() {
        let mapping = vec![
            IndexMap { source_start: 10, destination_start: 20, length: 5, },
        ];
        let mut ranges = Vec::new();
        add_destination_ranges(3, 6, &mapping, &mut ranges);
        assert_eq!(ranges, vec![(3, 6)]);
    }

    #[test]
    fn range_just_before_any_index_map() {
        let mapping = vec![
            IndexMap { source_start: 10, destination_start: 20, length: 5, },
        ];
        let mut ranges = Vec::new();
        add_destination_ranges(3, 7, &mapping, &mut ranges);
        assert_eq!(ranges, vec![(3, 7)]);
    }

    #[test]
    fn range_overlapping_start_of_first_index_map() {
        let mapping = vec![
            IndexMap { source_start: 10, destination_start: 20, length: 5, },
        ];
        let mut ranges = Vec::new();
        add_destination_ranges(8, 6, &mapping, &mut ranges);
        assert_eq!(ranges, vec![(8, 2), (20, 4)]);
    }

    #[test]
    fn range_overlapping_first_index_map() {
        let mapping = vec![
            IndexMap { source_start: 10, destination_start: 20, length: 2, },
        ];
        let mut ranges = Vec::new();
        add_destination_ranges(8, 6, &mapping, &mut ranges);
        assert_eq!(ranges, vec![(8, 2), (20, 2), (12, 2)]);
    }

    #[test]
    fn range_overlapping_first_and_second_map() {
        let mapping = vec![
            IndexMap { source_start: 10, destination_start: 20, length: 2, },
            IndexMap { source_start: 14, destination_start: 24, length: 2, },
        ];
        let mut ranges = Vec::new();
        add_destination_ranges(8, 10, &mapping, &mut ranges);
        assert_eq!(ranges, vec![(8, 2), (20, 2), (12, 2), (24, 2), (16, 2)]);
    }

    #[test]
    fn range_overlapping_first_and_second_map_maps_next_to_each_other() {
        let mapping = vec![
            IndexMap { source_start: 10, destination_start: 20, length: 2, },
            IndexMap { source_start: 12, destination_start: 30, length: 2, },
        ];
        let mut ranges = Vec::new();
        add_destination_ranges(8, 8, &mapping, &mut ranges);
        assert_eq!(ranges, vec![(8, 2), (20, 2), (30, 2), (14, 2)]);
    }

    #[test]
    fn range_after_the_maps() {
        let mapping = vec![
            IndexMap { source_start: 10, destination_start: 20, length: 2, },
            IndexMap { source_start: 12, destination_start: 30, length: 2, },
        ];
        let mut ranges = Vec::new();
        add_destination_ranges(14, 2, &mapping, &mut ranges);
        assert_eq!(ranges, vec![(14, 2)]);
    }

}