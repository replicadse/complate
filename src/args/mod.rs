pub enum Command {
    Init,
    Print(PrintArguments),
}

pub struct PrintArguments {
    pub configuration: String,
    pub shell_trust: ShellTrust,
}

pub enum ShellTrust {
    None,
    Prompt,
    Ultimate
}

pub struct ClapArgumentLoader {
}

impl ClapArgumentLoader {
    pub async fn load_from_cli() -> std::io::Result<Command> {
        let command = clap::App::new("complate")
            .version("0.0.0")
            .about("A git commit buddy.")
            .author("Weber, Heiko Alexander <heiko.a.weber@gmail.com>")
            .subcommand(clap::App::new("init"))
            .subcommand(clap::App::new("print")
                .arg(clap::Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("The configuration file to use.")
                    .default_value("./.complate/config.yml")
                    .multiple(false)
                    .required(false)
                    .takes_value(true))
                .arg(clap::Arg::with_name("shell-trust")
                    .long("shell-trust")
                    .help("Enables the shell mode. This is potentially insecure and should only be done for trustworthy sources.")
                    .multiple(false)
                    .required(false)
                    .default_value("none")
                    .takes_value(true)))
            .get_matches();

        if command.subcommand_matches("init").is_some() {
            return Ok(Command::Init);
        }

        match command.subcommand_matches("print") {
            Some(x) => {
                let config_file = x.value_of("config").unwrap().to_owned();
                let config = std::fs::read_to_string(config_file)?;
        
                let shell_trust = match x.value_of("shell-trust") {
                    Some(x) => match x {
                        "none" => ShellTrust::None,
                        "prompt" => ShellTrust::Prompt,
                        "ultimate" => ShellTrust::Ultimate,
                        _ => ShellTrust::None
                    },
                    None => ShellTrust::None,
                };

                Ok(Command::Print(PrintArguments {
                    configuration: config,
                    shell_trust
                }))
            }
            None => {
                Err(std::io::Error::new(std::io::ErrorKind::Other, "could not resolve subcommand", ))
            }
        }
    }
}
