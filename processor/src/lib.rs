use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub fn process<LoadState, State, ProcessedState, FinalResult>(
    file_name: &str,
    initial_state: LoadState,
    parse_line: fn(LoadState, String) -> Result<LoadState, anyhow::Error>,
    finalise_state: fn(LoadState) -> Result<State, anyhow::Error>,
    perform_processing: fn(State) -> Result<ProcessedState, anyhow::Error>,
    calc_result: fn(ProcessedState) -> Result<FinalResult, anyhow::Error>,
) -> Result<FinalResult, anyhow::Error> {
    let file = File::open(file_name)?;
    let loaded_state = BufReader::new(file)
        .lines()
        .map(|l| l.unwrap())
        .try_fold(initial_state, parse_line)?;
    let finalised_state = finalise_state(loaded_state)?;
    let processed_state = perform_processing(finalised_state)?;
    calc_result(processed_state)
}

pub fn ok_identity<T>(t: T) -> Result<T, anyhow::Error> {
    Ok(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_file() {
        let initial_state: Vec<String> = Vec::new();
        let res = process(
            "test-input.txt",
            initial_state,
            |mut vec, line| {
                vec.push(line);
                Ok(vec)
            },
            ok_identity,
            |vec| Ok(vec.join("+")),
            ok_identity,
        );
        match res {
            Ok(message) => assert_eq!(message, "Some Input Here+It's Good".to_string()),
            Err(e) => panic!("{}", e),
        }
    }
}
