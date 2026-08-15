use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::process::ExitCode;

use ggen_engine::graph::{EngineQueryResults, EngineValue, GraphEngine, GraphLawStore};
use serde_json::{json, Map, Value};

const GGEN_VERSION: &str = "26.8.15";
const GGEN_COMMIT: &str = "162e466d8f07d0a75a468b4441b4bc8b1aad369b";

fn value_to_json(value: EngineValue) -> Value {
    match value {
        EngineValue::Bool(value) => Value::Bool(value),
        EngineValue::Int(value) => Value::Number(value.into()),
        EngineValue::Float(value) => serde_json::Number::from_f64(value)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(value.to_string())),
        EngineValue::String(value) => Value::String(value),
    }
}

fn required_arg(args: &[String], name: &str) -> Result<String, String> {
    let index = args
        .iter()
        .position(|value| value == name)
        .ok_or_else(|| format!("REFUSED:MISSING_ARGUMENT:{name}"))?;
    args.get(index + 1)
        .cloned()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("REFUSED:MISSING_ARGUMENT_VALUE:{name}"))
}

fn load_graph(graph_file: &str) -> Result<GraphLawStore, String> {
    let turtle = fs::read_to_string(graph_file)
        .map_err(|error| format!("REFUSED:GRAPH_READ:{graph_file}:{error}"))?;
    let graph = GraphLawStore::new().map_err(|error| format!("REFUSED:GGEN_GRAPH_INIT:{error}"))?;
    graph
        .insert_turtle(&turtle)
        .map_err(|error| format!("REFUSED:GGEN_TURTLE_LOAD:{error}"))?;
    Ok(graph)
}

fn query_envelope(graph: &GraphLawStore, sparql: &str) -> Result<Value, String> {
    match graph
        .query(sparql)
        .map_err(|error| format!("REFUSED:GGEN_SPARQL:{error}"))?
    {
        EngineQueryResults::Boolean(value) => Ok(json!({
            "kind": "boolean",
            "boolean": value,
            "variables": [],
            "bindings": [],
            "result_count": 1
        })),
        EngineQueryResults::Solutions(rows) => {
            let mut variables = BTreeSet::new();
            let mut bindings = Vec::with_capacity(rows.len());
            for row in rows {
                let mut binding = Map::new();
                for (name, value) in row {
                    variables.insert(name.clone());
                    binding.insert(name, value_to_json(value));
                }
                bindings.push(Value::Object(binding));
            }
            Ok(json!({
                "kind": "solutions",
                "variables": variables.into_iter().collect::<Vec<_>>(),
                "result_count": bindings.len(),
                "bindings": bindings
            }))
        }
        EngineQueryResults::Graph(triples) => Ok(json!({
            "kind": "graph",
            "variables": [],
            "bindings": [],
            "result_count": triples.len(),
            "triples": triples
                .into_iter()
                .map(|triple| json!({
                    "subject": triple.subject,
                    "predicate": triple.predicate,
                    "object": triple.object_value,
                    "ntriples": triple.ntriples
                }))
                .collect::<Vec<_>>()
        })),
    }
}

fn execute(args: &[String]) -> Result<Value, String> {
    let command = args
        .first()
        .map(String::as_str)
        .ok_or_else(|| "REFUSED:MISSING_COMMAND".to_string())?;

    match command {
        "version" => Ok(json!({
            "engine": "ggen-engine",
            "version": GGEN_VERSION,
            "commit": GGEN_COMMIT
        })),
        "validate" => {
            let graph_file = required_arg(args, "--graph-file")?;
            let graph = load_graph(&graph_file)?;
            let quads = graph
                .canonical_quads()
                .map_err(|error| format!("REFUSED:GGEN_CANONICALIZE:{error}"))?;
            let hash = graph
                .state_hash()
                .map_err(|error| format!("REFUSED:GGEN_STATE_HASH:{error}"))?;
            let state_hash = hash
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<Vec<_>>()
                .join("");
            Ok(json!({
                "valid": true,
                "quad_count": quads.len(),
                "state_hash": state_hash
            }))
        }
        "query" => {
            let graph_file = required_arg(args, "--graph-file")?;
            let sparql = required_arg(args, "--sparql")?;
            if sparql.trim().is_empty() {
                return Err("REFUSED:EMPTY_SPARQL_QUERY".to_string());
            }
            let graph = load_graph(&graph_file)?;
            query_envelope(&graph, &sparql)
        }
        _ => Err(format!("REFUSED:UNKNOWN_COMMAND:{command}")),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match execute(&args) {
        Ok(value) => match serde_json::to_string(&value) {
            Ok(encoded) => {
                println!("{encoded}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("REFUSED:JSON_SERIALIZATION:{error}");
                ExitCode::from(2)
            }
        },
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}
