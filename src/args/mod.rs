use std::io::{stdin, Read, Result};

pub struct CallArgs {
    pub privileges: Privilege,
    pub command: Command,
}

impl CallArgs {
    #[allow(clippy::single_match)]
    pub async fn validate(&self) -> Result<()> {
        match self.privileges {
            Privilege::Normal => match &self.command {
                Command::Render(args) => {
                    match args.backend {
                        #[cfg(feature = "backend+ui")]
                        Backend::UI => return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "can not use backend+ui without experimental features being activated",
                        )),
                        #[cfg(feature = "backend+cli")]
                        Backend::CLI => {}
                    };
                    #[allow(unreachable_code)]
                    Ok(())
                }
                _ => Ok(()),
            },
            Privilege::Experimental => Ok(()),
        }
    }
}

pub enum Privilege {
    Normal,
    Experimental,
}

pub enum Command {
    Init,
    Render(RenderArguments),
}

pub struct RenderArguments {
    pub configuration: String,
    pub template: Option<String>,
    pub shell_trust: ShellTrust,
    pub backend: Backend,
}

pub enum ShellTrust {
    None,
    Prompt,
    Ultimate,
}

pub enum Backend {
    #[cfg(feature = "backend+cli")]
    CLI,
    #[cfg(feature = "backend+ui")]
    UI,
}

pub struct ClapArgumentLoader {}

impl ClapArgumentLoader {
    pub async fn load_from_cli() -> std::io::Result<CallArgs> {
        let mut backend_values = Vec::new();
        if cfg!(feature = "backend+cli") {
            backend_values.push("cli");
        }
        if cfg!(feature = "backend+ui") {
            backend_values.push("ui");
        }

        let command = clap::App::new("complate")
            .version(env!("CARGO_PKG_VERSION"))
            .about("A git commit buddy.")
            .author("Weber, Heiko Alexander <haw@voidpointergroup.com>")
            .arg(clap::Arg::with_name("experimental")
                    .short("e")
                    .long("experimental")
                    .value_name("EXPERIMENTAL")
                    .help("Enables experimental features that do not count as stable.")
                    .required(false)
                    .takes_value(false))
            .subcommand(clap::App::new("init"))
            .subcommand(clap::App::new("render")
                .arg(clap::Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("The configuration file to use.")
                    .default_value("./.complate/config.yml")
                    .multiple(false)
                    .required(false)
                    .takes_value(true))
                .arg(clap::Arg::with_name("template")
                    .short("t")
                    .long("template")
                    .value_name("TEMPLATE")
                    .help("Specify the template to use from the config and skip it's selection.")
                    .multiple(false)
                    .required(false)
                    .takes_value(true))
                .arg(clap::Arg::with_name("shell-trust")
                    .long("shell-trust")
                    .value_name("SHELL_TRUST")
                    .help("Enables the shell mode. This is potentially insecure and should only be done for trustworthy sources.")
                    .possible_values(&["none", "prompt", "ultimate"])
                    .multiple(false)
                    .required(false)
                    .default_value("none")
                    .takes_value(true))
                .arg(clap::Arg::with_name("backend")
                    .short("b")
                    .long("backend")
                    .value_name("BACKEND")
                    .help("The execution backend (cli=native-terminal, ui=ui emulator in terminal).")
                    .possible_values(backend_values.as_slice())
                    .default_value(backend_values.first().unwrap_or(&""))
                    .multiple(false)
                    .required(false)
                    .takes_value(true)))
            .get_matches();

        let privileges = if command.is_present("experimental") {
            Privilege::Experimental
        } else {
            Privilege::Normal
        };

        if command.subcommand_matches("init").is_some() {
            return Ok(CallArgs {
                privileges,
                command: Command::Init,
            });
        }

        match command.subcommand_matches("render") {
            Some(x) => {
                let config = if x.is_present("config") {
                    let config_param = x.value_of("config").unwrap();
                    if config_param == "-" {
                        match privileges {
                            Privilege::Normal => {
                                return Err(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "can not use pipe argument without experimental features being activated",
                                ));
                            }
                            Privilege::Experimental => {}
                        }
                        let mut buffer = String::new();
                        let stdin = stdin();
                        stdin.lock().read_to_string(&mut buffer)?;
                        buffer
                    } else {
                        std::fs::read_to_string(config_param.to_owned())?
                    }
                } else {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "configuration not specified",
                    ));
                };

                let template = if x.is_present("template") {
                    Some(x.value_of("template").unwrap().to_owned())
                } else {
                    None
                };

                let shell_trust = match x.value_of("shell-trust") {
                    Some(x) => match x {
                        "none" => ShellTrust::None,
                        "prompt" => ShellTrust::Prompt,
                        "ultimate" => ShellTrust::Ultimate,
                        _ => ShellTrust::None,
                    },
                    None => ShellTrust::None,
                };

                let backend = match x.value_of("backend") {
                    Some(x) => match x {
                        #[cfg(feature = "backend+cli")]
                        "cli" => Backend::CLI,
                        #[cfg(feature = "backend+ui")]
                        "ui" => Backend::UI,
                        _ => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "unknown backend configuration",
                            ))
                        }
                    },
                    #[cfg(feature = "backend+cli")]
                    None => Backend::CLI,
                    #[cfg(feature = "backend+ui")]
                    #[allow(unreachable_patterns)]
                    None => Backend::UI,
                };

                Ok(CallArgs {
                    privileges,
                    command: Command::Render(RenderArguments {
                        configuration: config,
                        template,
                        shell_trust,
                        backend,
                    }),
                })
            }
            None => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "could not resolve subcommand",
            )),
        }
    }
}
