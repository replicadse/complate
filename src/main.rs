include!("check_features.rs");

use std::io::Write;
use std::path::PathBuf;
use std::result::Result;

pub mod args;
pub mod config;
pub mod error;
pub mod reference;
pub mod render;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = crate::args::ClapArgumentLoader::load_from_cli().await?;
    cmd.validate().await?;

    match cmd.command {
        | crate::args::Command::Man(path) => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path).unwrap();
            reference::build_manpages(&out_path)?;
            Ok(())
        },
        | crate::args::Command::Autocomplete(path, shell) => {
            let out_path = PathBuf::from(path);
            std::fs::create_dir_all(&out_path).unwrap();
            reference::build_shell_completion(&out_path, &shell)?;
            Ok(())
        },
        | crate::args::Command::Init => {
            std::fs::create_dir_all("./.complate")?;
            std::fs::write("./.complate/config.yaml", crate::config::default_config().await)?;
            Ok(())
        },
        | crate::args::Command::Render(x) => {
            let res = crate::render::select_and_render(x).await?;
            std::io::stdout().write_all(res.as_bytes())?;
            Ok(())
        },
    }
}
