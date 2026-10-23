use std::process::ExitCode;

mod cli;
mod context_cli;
mod doctor_cli;
mod eval_cli;
mod fabricate_cli;
mod federation_cli;
mod gateway_cli;
mod gate_cli;
mod harness_cli;
mod hooks_cli;
mod host_cli;
mod learning_cli;
mod provenance_cli;
mod seed_cli;
mod skill_cli;
mod trace_cli;

fn main() -> ExitCode {
    match cli::run() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::from(2)
        }
    }
}
