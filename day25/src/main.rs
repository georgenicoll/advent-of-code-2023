use std::time;

use processor::process;

type AError = anyhow::Error;
type InitialState = Vec<String>;
type LoadedState = InitialState;
type ProcessedState = LoadedState;
type FinalResult = ProcessedState;

fn parse_line(mut state: InitialState, line: String) -> Result<InitialState, AError> {
    state.push(line);
    Ok(state)
}

fn finalise_state(state: InitialState) -> Result<LoadedState, AError> {
    Ok(state)
}

fn perform_processing(state: LoadedState) -> Result<ProcessedState, AError> {
    Ok(state)
}

fn calc_result(state: ProcessedState) -> Result<FinalResult, AError> {
    Ok(state)
}

fn main() {
    let file = "test-input.txt";
    //let file = "test-input2.txt";
    //let file = "input.txt";

    let started1_at = time::Instant::now();
    let result1 = process(
        file,
        Vec::default(),
        parse_line,
        finalise_state,
        perform_processing,
        calc_result,
    );
    match result1 {
        Ok(res) => println!(
            "Result 1: {:?} (took: {}s)",
            res,
            started1_at.elapsed().as_secs_f32()
        ),
        Err(e) => println!("Error on 1: {}", e),
    }

    let started2_at = time::Instant::now();
    let result2 = process(
        file,
        Vec::default(),
        parse_line,
        finalise_state,
        perform_processing,
        calc_result,
    );
    match result2 {
        Ok(res) => println!(
            "Result 2: {:?} (took: {}s)",
            res,
            started2_at.elapsed().as_secs_f32()
        ),
        Err(e) => println!("Error on 2: {}", e),
    }
}
