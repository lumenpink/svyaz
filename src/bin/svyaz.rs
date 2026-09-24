use std::env;
use std::process;
use svyaz::{execute, parse_args, print_help, Action, VERSION};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match parse_args(args) {
        Ok(Action::Run(config)) => {
            match execute(config) {
                Ok(exit_code) => process::exit(exit_code),
                Err(err) => {
                    eprintln!("svyaz: error: {}", err);
                    process::exit(1);
                }
            }
        }
        Ok(Action::Version) => {
            println!("svyaz v{}", VERSION);
        }
        Ok(Action::Help) => {
            print_help("svyaz");
        }
        Err(err) => {
            eprintln!("svyaz: {}", err);
            process::exit(2);
        }
    }
}
