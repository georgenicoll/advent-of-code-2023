use processor::process;

type AError = anyhow::Error;
type InitialState = Vec<String>;
type LoadedState = InitialState;
type ProcessedState = LoadedState;
type FinalResult = ProcessedState;

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
        finalise_state,
        perform_processing,
        calc_result,
    );
    match result2 {
        Ok(res) => println!("Result 2: {:?}", res),
        Err(e) => println!("Error on 2: {}", e),
    }
}

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
