extern crate clap;
use clap::{App, Arg};

fn main() -> std::io::Result<()> {
    env_logger::init();
    App::new("complate")
       .version("0.1.0")
       .about("A git commit buddy.")
       .author("Weber, Heiko Alexander <heiko.a.weber@gmail.com>")
       .arg(Arg::with_name("sample")
            .short("s")
            .long("sample")
            .value_name("SAMPLE")
            .help("Just a config example.")
            .takes_value(true))
       .get_matches();
    Ok(())
}
