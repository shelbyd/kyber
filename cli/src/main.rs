use serde::{Deserialize, Serialize};
use structopt::StructOpt;

mod refactorings;
use refactorings::Mutation;

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
    Perform,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();

    match options.command {
        Command::Rpc(RpcMethod::Suggest) => {
            let request: SuggestRequest = serde_json::from_reader(std::io::stdin())?;
            let suggestions = suggestions_for_context(&request.context);
            serde_json::to_writer(std::io::stdout(), &SuggestResponse { suggestions })?;
        }
        Command::Rpc(RpcMethod::Perform) => {
            let request: PerformRequest = serde_json::from_reader(std::io::stdin())?;
            let refactoring = refactorings::all()
                .find(|r| r.id() == request.id)
                .ok_or_else(|| format!("Could not find refactoring with id {}", request.id))?;
            let mutations = refactoring.perform(&request.context)?;
            serde_json::to_writer(std::io::stdout(), &PerformResponse { mutations })?;
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
struct PerformRequest {
    context: EditorContext,
    id: String,
}

#[derive(Serialize, Debug)]
struct PerformResponse {
    mutations: Vec<Mutation>,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EditorContext {
    contents: Vec<ContentRegion>,
}

impl EditorContext {
    pub fn contents_ref(&self) -> Vec<ContentRegion<&str>> {
        self.contents
            .iter()
            .map(|c| ContentRegion {
                text: c.text.as_str(),
                selected: c.selected,
            })
            .collect()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ContentRegion<S = String> {
    text: S,
    selected: bool,
}

fn suggestions_for_context(_context: &EditorContext) -> Vec<Refactoring> {
    refactorings::all()
        .filter(|r| r.applies_to(_context))
        .map(|r| Refactoring {
            name: r.name(),
            description: r.description(),
            id: r.id(),
        })
        .collect()
}

#[derive(Serialize, Debug)]
struct Refactoring {
    name: String,
    description: String,
    id: String,
}
