use std::process::ExitCode;

use hctl2_web_server::{Command, PROGRAM_NAME};

fn main() -> ExitCode {
    match hctl2_web_server::parse_command(std::env::args_os().skip(1)) {
        Ok(Command::Help(output) | Command::Version(output)) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Ok(Command::Serve(config)) => match hctl2_web_server::serve(&config) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("error[HCTL2_WEB_SERVER_RUNTIME]: {error}");
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprintln!("error[HCTL2_WEB_SERVER_ARGUMENT]: {error}");
            eprintln!("Try '{PROGRAM_NAME} --help' for usage.");
            ExitCode::from(2)
        }
    }
}
