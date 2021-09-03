use clap::IntoApp;
use clap_generate::{generators::*, generate_to};

include!("src/cli.rs");

fn main() -> std::io::Result<()> {
    let mut app = SteamLaunchArgs::into_app();

    let outdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("completions/");
    std::fs::create_dir_all(&outdir)?;
    generate_to::<Bash, _, _>(&mut app, "steam-launch", &outdir)?;
    generate_to::<Fish, _, _>(&mut app, "steam-launch", &outdir)?;
    generate_to::<Zsh, _, _>(&mut app, "steam-launch", &outdir)?;
    generate_to::<PowerShell, _, _>(&mut app, "steam-launch", &outdir)?;
    generate_to::<Elvish, _, _>(&mut app, "steam-launch", &outdir)?;
    Ok(())
}
