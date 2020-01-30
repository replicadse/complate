pub struct Arguments {
    pub configuration: String,
    pub shell_mode: ShellMode,
}

pub enum Trust {
    None,
    Prompt,
    Ultimate
}

pub enum ShellMode {
    Disabled,
    Enabled(Trust),
}

pub struct ClapArgumentLoader {
}

impl ClapArgumentLoader {
    pub async fn load_from_cli() -> std::io::Result<Arguments> {
        let args = clap::App::new("complate")
            .version("0.1.0")
            .about("A git commit buddy.")
            .author("Weber, Heiko Alexander <heiko.a.weber@gmail.com>")
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
                .takes_value(true))
            .get_matches();

        let config_file = args.value_of("config").unwrap().to_owned();
        let config = std::fs::read_to_string(config_file)?;

        let shell_trust = match args.value_of("shell") {
            Some(x) => match x {
                "none" => ShellMode::Disabled,
                "prompt" => ShellMode::Enabled(Trust::Prompt),
                "ultimate" => ShellMode::Enabled(Trust::Ultimate),
                _ => panic!("unrecognized trust option")
            },
            None => ShellMode::Disabled,
        };

        Ok(Arguments {
            configuration: config,
            shell_mode: shell_trust,
        })
    }
}
