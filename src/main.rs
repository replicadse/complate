extern crate clap;
use clap::{App, Arg};

extern crate dialoguer;

fn main() -> std::io::Result<()> {
    env_logger::init();
    App::new("complate")
       .version("0.1.0")
       .about("A git commit buddy.")
       .author("Weber, Heiko Alexander <heiko.a.weber@gmail.com>")
       .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("The configuration file to use.")
            .default_value("./res/config.yml")
            .multiple(false)
            .required(false)
            .takes_value(true))
       .get_matches();

    println!("Please select an option:");
    let options = vec!("Hello", "World", "You fool.");
    let selection = dialoguer::Select::new()
        .items(options.as_slice())
        .default(0)
        .interact().unwrap();
    println!("You chose option: {:?}, aka \"{}\"", selection, options[selection]);

    let final_content = dialoguer::Editor::new().edit(options[selection]).unwrap().unwrap();
    println!("Final content: \"{}\"", final_content);

    Ok(())
}
