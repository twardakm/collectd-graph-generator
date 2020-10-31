use cgg::config::Config;
use clap::{load_yaml, App};

fn main() {
    let yaml = load_yaml!("cli.yml");
    let cli = App::from(yaml).get_matches();

    let config = match Config::new(&cli) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error: {:?}\n", err);
            help();
            std::process::exit(1);
        }
    };

    std::process::exit(match cgg::run(config) {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("Error: {:?}\n", err);
            help();
            1
        }
    })
}

fn help() {
    let yaml = load_yaml!("cli.yml");
    App::from(yaml).print_help().unwrap();
}
