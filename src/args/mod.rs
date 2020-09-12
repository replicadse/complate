pub struct CallArgs {
    pub call_mode: CallMode,
    pub command: Command,
}

impl CallArgs {
    pub async fn validate(&self) {
    }
}

pub enum CallMode {
    Normal,
    Experimental
}

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
    Ultimate,
}

pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub async fn load_from_cli() -> std::io::Result<CallArgs> {
        let command = clap::App::new("complate")
            .version(env!("CARGO_PKG_VERSION"))
            .about("A git commit buddy.")
            .author("Weber, Heiko Alexander <heiko.a.weber@gmail.com>")
            .arg(clap::Arg::with_name("experimental")
                    .short("e")
                    .long("experimental")
                    .value_name("EXPERIMENTAL")
                    .help("Enables experimental features that do not count as stable.")
                    .required(false)
                    .takes_value(false))
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

        let call_mode = if command.is_present("experimental") {
            CallMode::Experimental
        } else {
            CallMode::Normal
        };

        if command.subcommand_matches("init").is_some() {
            return Ok(CallArgs{
                call_mode,
                command: Command::Init,
            });
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
                        _ => ShellTrust::None,
                    },
                    None => ShellTrust::None,
                };

                Ok(CallArgs{
                    call_mode,
                    command: Command::Print(PrintArguments {
                        configuration: config,
                        shell_trust,
                    })
                })
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "could not resolve subcommand",
            )),
        }
    }
}
