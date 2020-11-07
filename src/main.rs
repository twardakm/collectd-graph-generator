use cgg::config::Config;
use clap::{load_yaml, App};

const EXAMPLES: &'static str = &"EXAMPLES:
    ./cgg -i /var/lib/collectd/marcin-manjaro/ -t \"last 4 hours\"\n
    ./cgg --input marcin@localhost:/var/lib/collectd/marcin-manjaro/ \\
-t \"last 10 days\" -w 2048 -h 1024 -o processes.png\n
    ./cgg -i marcin@192.168.0.163:/var/lib/collectd/marcin-manjaro/ \\
-t \"last 1 hour\" --processes \"firefox,spotify,visual studio code\"";

fn main() {
    let yaml = load_yaml!("cli.yml");
    let cli = App::from(yaml).after_help(EXAMPLES).get_matches();

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
