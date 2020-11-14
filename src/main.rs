use cgg::config::Config;
use clap::{load_yaml, App};
use log::error;

const EXAMPLES: &'static str = &"EXAMPLES:
    ./cgg -i /var/lib/collectd/marcin-manjaro/ -t \"last 4 hours\"\n
    ./cgg --input marcin@localhost:/var/lib/collectd/marcin-manjaro/ \\
-t \"last 10 days\" -w 2048 -h 1024 -o processes.png\n
    ./cgg -i marcin@192.168.0.163:/var/lib/collectd/marcin-manjaro/ \\
-t \"last 1 hour\" --processes \"firefox,spotify,visual studio code\"\n
    ./cgg -i marcin@localhost:/var/lib/collectd/marcin-manjaro/ \\
-p processes,memory -t \"last 1 hour\" --memory buffered,free,cached,used";

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let yaml = load_yaml!("cli.yml");
    let cli = App::from(yaml).after_help(EXAMPLES).get_matches();

    let config = match Config::new(&cli) {
        Ok(config) => config,
        Err(err) => {
            error!("Error: {:?}\n", err);
            help();
            std::process::exit(1);
        }
    };

    std::process::exit(match cgg::run(config) {
        Ok(()) => 0,
        Err(err) => {
            error!("Error: {:?}", err);
            1
        }
    })
}

fn help() {
    let yaml = load_yaml!("cli.yml");
    App::from(yaml).print_help().unwrap();
}
