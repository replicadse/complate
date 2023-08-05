use {
    crate::args::ClapArgumentLoader,
    anyhow::Result,
    clap_complete::Shell,
    clap_mangen::Man,
    std::{
        fs::File,
        io::{
            Error,
            Write,
        },
        path::Path,
    },
};

fn collect_commands() -> Vec<(String, clap::Command)> {
    let mut cmds: Vec<(String, clap::Command)> = Vec::new();
    fn rec_add(path: &str, cmds: &mut Vec<(String, clap::Command)>, parent: &clap::Command) {
        let new_path = &format!("{}-{}", path, parent.get_name());
        cmds.push((new_path.into(), parent.clone()));
        for subc in parent.get_subcommands() {
            rec_add(new_path, cmds, subc);
        }
    }
    rec_add("", &mut cmds, &ClapArgumentLoader::root_command());
    cmds
}

pub fn build_shell_completion(outdir: &Path, shell: &Shell) -> Result<()> {
    let mut app = ClapArgumentLoader::root_command();
    clap_complete::generate_to(*shell, &mut app, "complate", &outdir)?;

    Ok(())
}

pub fn build_markdown(outdir: &Path) -> Result<(), Error> {
    for cmd in collect_commands() {
        let file = Path::new(&outdir).join(&format!("{}.md", cmd.0.strip_prefix("-").unwrap()));
        let mut file = File::create(&file)?;
        file.write(clap_markdown::help_markdown_command(&cmd.1).as_bytes())?;
    }
    Ok(())
}

pub fn build_manpages(outdir: &Path) -> Result<(), Error> {
    for cmd in collect_commands() {
        let file = Path::new(&outdir).join(&format!("{}.1", cmd.0.strip_prefix("-").unwrap()));
        let mut file = File::create(&file)?;
        Man::new(cmd.1).render(&mut file)?;
    }

    Ok(())
}
