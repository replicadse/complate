use clap_complete::Shell;
use clap_mangen::Man;
use std::{fs::File, io::Error, path::Path};

pub fn build_shell_completion(outdir: &Path, shell: &Shell) -> Result<(), Error> {
    let mut app = crate::args::ClapArgumentLoader::root_command();
    clap_complete::generate_to(*shell, &mut app, "complate", &outdir)?;

    Ok(())
}

pub fn build_manpages(outdir: &Path) -> Result<(), Error> {
    let mut cmds: Vec<(String, clap::Command)> = Vec::new();
    fn rec_add(path: &str, cmds: &mut Vec<(String, clap::Command)>, parent: &clap::Command) {
        let new_path = &format!("{}-{}", path, parent.get_name());
        cmds.push((new_path.to_owned(), parent.clone()));
        for subc in parent.get_subcommands() {
            rec_add(new_path, cmds, subc);
        }
    }
    rec_add("", &mut cmds, &crate::args::ClapArgumentLoader::root_command());

    for cmd in cmds {
        let file = Path::new(&outdir).join(&format!("{}.1", cmd.0.strip_prefix("-").unwrap()));
        let mut file = File::create(&file)?;
        Man::new(cmd.1).render(&mut file)?;
    }

    Ok(())
}
