//! `castle` CLI — binary entry point.
//!
//! Follows the ggen-marketplace `clap-noun-verb` convention (see
//! `~/ggen-marketplace/packs/clap-noun-verb-crate-pack` and
//! `~/ggen/examples/clap-noun-verb-cli`): a thin `main.rs` that hands off to
//! `clap_noun_verb::run()`, which auto-discovers every `#[verb]` command
//! registered by `verbs::routes` via `linkme::distributed_slice`. No
//! explicit command wiring needed here.

use std::process::ExitCode;

mod verbs;

fn main() -> ExitCode {
    match clap_noun_verb::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ERROR: {error}");
            ExitCode::FAILURE
        }
    }
}
