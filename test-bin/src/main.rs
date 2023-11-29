use processor::process;

type LoadingState = Vec<Vec<String>>;
type State = LoadingState;
type ProcessedState = Vec<i64>;
type Res = i64;

fn main() {
    let res = process(
        "test-bin/input.txt",
        LoadingState::new(),
        parse_line,
        finalise_state,
        perform_processing,
        calc_result,
    );
    match res {
        Ok(res) => println!("{}", res),
        Err(e) => panic!("{}", e),
    };
}

fn parse_line(mut state: LoadingState, line: String) -> Result<LoadingState, anyhow::Error> {
    if state.is_empty() {
        state.push(Vec::new());
    }
    let trimmed_line = line.trim();
    if trimmed_line.is_empty() {
        state.push(Vec::new());
    } else {
        state.last_mut().unwrap().push(trimmed_line.into());
    }
    Ok(state)
}

fn finalise_state(state: LoadingState) -> Result<State, anyhow::Error> {
    Ok(state)
}

fn perform_processing(state: State) -> Result<ProcessedState, anyhow::Error> {
    let mut results = ProcessedState::new();
    for calc in state.iter() {
        let a = calc.get(0).unwrap().parse::<i64>()?;
        let operator = calc.get(1).unwrap();
        let b = calc.get(2).unwrap().parse::<i64>()?;
        let result = match operator.as_str() {
            "+" => a + b,
            "*" => a * b,
            _ => {
                return Err(anyhow::Error::msg(format!(
                    "Unrecognised operator: {}",
                    operator
                )))
            }
        };
        results.push(result);
    }
    Ok(results)
}

fn calc_result(state: ProcessedState) -> Result<Res, anyhow::Error> {
    Ok(state.iter().sum())
}
