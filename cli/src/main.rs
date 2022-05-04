use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    Rpc(RpcMethod),
}

#[derive(StructOpt, Debug)]
enum RpcMethod {
    Suggest,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    match options.command {
        Command::Rpc(RpcMethod::Suggest) => {
            let request: SuggestRequest = serde_json::from_reader(std::io::stdin())?;
            let suggestions = suggestions_for_context(&request.context);
            serde_json::to_writer(std::io::stdout(), &SuggestResponse { suggestions })?;
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct SuggestRequest {
    context: EditorContext,
}

#[derive(Serialize, Debug)]
struct SuggestResponse {
    suggestions: Vec<Refactoring>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct EditorContext {
    contents: Vec<ContentRegion>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct ContentRegion {
    text: String,
    selected: bool,
}

fn suggestions_for_context(_context: &EditorContext) -> Vec<Refactoring> {
    vec![Refactoring {
        name: "Extract ! from !=".to_string(),
        description: "Replace a != b with !(a == b)".to_string(),
    }]
}

#[derive(Serialize, Debug)]
struct Refactoring {
    name: String,
    description: String,
}
