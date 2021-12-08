include!("check_features.rs");

use futures::executor::block_on;

use std::io::Write;
use std::result::Result;

pub mod args;
pub mod config;
pub mod render;
pub mod error;

async fn async_main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = crate::args::ClapArgumentLoader::load_from_cli().await?;
    cmd.validate().await?;

    match cmd.command {
        crate::args::Command::Init => {
            std::fs::create_dir_all("./.complate")?;
            std::fs::write("./.complate/config.yaml", crate::config::default_config().await)?;
            Ok(())
        }
        crate::args::Command::Render(x) => {
            let res = crate::render::select_and_render(x).await?;
            std::io::stdout().write_all(res.as_bytes())?;
            Ok(())
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    block_on(async_main())
}
