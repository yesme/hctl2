use std::process::ExitCode;

fn main() -> ExitCode {
    match hctl2_tool::run(std::env::args_os().skip(1)) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("error[{}]: {}", error.code(), error.message());
            ExitCode::FAILURE
        }
    }
}
