mod args;
mod error;

use clap::ValueEnum;
use clap_complete::generate_to;
use clap_mangen::Man;
use std::{
    env,
    fs::File,
    io::Error,
    path::{Path, PathBuf},
};

fn build_shell_completion(outdir: &Path) -> Result<(), Error> {
    let mut app = crate::args::ClapArgumentLoader::root_command();
    let shells = clap_complete::Shell::value_variants();

    for shell in shells {
        generate_to(*shell, &mut app, "example", &outdir)?;
    }

    Ok(())
}

fn build_manpages(outdir: &Path) -> Result<(), Error> {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let out_path = PathBuf::from(outdir);
    let mut path_root = out_path.ancestors().nth(4).unwrap().to_owned();
    path_root.push("assets");

    let mut path_shell = path_root.clone();
    path_shell.push("shell");
    std::fs::create_dir_all(&path_shell).unwrap();
    
    let mut path_man = path_root.clone();
    path_man.push("man");
    std::fs::create_dir_all(&path_man).unwrap();

    build_shell_completion(&path_shell)?;
    build_manpages(&path_man)?;

    Ok(())
}