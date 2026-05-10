use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    match dappswarm::run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
