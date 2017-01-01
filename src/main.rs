extern crate getopts;
extern crate librespot;
extern crate env_logger;
extern crate ini;
extern crate xdg;
#[macro_use]
extern crate log;
extern crate simple_signal;

use std::process::exit;
use std::thread;
use std::env;

use librespot::spirc::SpircManager;
use librespot::main_helper;
use librespot::session::Session;
use librespot::player::Player;

use simple_signal::Signal;

mod config;
mod cli;

fn main() {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info,librespot=trace")
    }
    env_logger::init().unwrap();

    let opts = cli::command_line_argument_options();
    let args: Vec<String> = std::env::args().collect();

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            error!("Error: {}\n{}", f.to_string(), cli::usage(&args[0], &opts));
            exit(1)
        }
    };

    if matches.opt_present("backends") {
        cli::print_backends();
        exit(0);
    }

    let config = config::get_config();

    let cache = config.cache;
    let backend = config.backend;
    let session_config = config.session_config;
    let device_name = session_config.device_name.clone();
    let session = Session::new(session_config, cache);
    let credentials = main_helper::get_credentials(&session, &matches);
    session.login(credentials).unwrap();
    let player = Player::new(session.clone(), move || {
        main_helper::find_backend(backend.as_ref()
            .map(AsRef::as_ref))(Some(device_name.as_ref()))
    });

    let spirc = SpircManager::new(session.clone(), player);
    let spirc_signal = spirc.clone();
    thread::spawn(move || spirc.run());

    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |signals| {
        info!("Signal received: {:?}. Say goodbye and exit.", signals);
        spirc_signal.send_goodbye();
        exit(0);
    });

    loop {
        session.poll();
    }

}
