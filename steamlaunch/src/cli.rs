use std::path::PathBuf;
use clap::Clap;


#[derive(Debug, Clap)]
#[clap(
    name = "steam-launch",
    about = "launch steam games from the command line"
)]
pub struct SteamLaunchArgs {
    #[clap(parse(from_os_str), short='s', long="steam-dir")]
    pub steam_dir: Option<PathBuf>,

    #[clap(short, long, env="STEAM_USER")]
    pub user: Option<String>,
    #[clap(short, long)]
    pub password: Option<String>,
    #[clap(short='x', long="password-cmd", env="STEAM_PASSWORD_CMD")]
    pub password_cmd: Option<String>,

    #[clap(subcommand)]
    pub cmd: SteamLaunchCmds
}

#[derive(Debug, Clap)]
pub enum SteamLaunchCmds {
    List(ListCmd),
    Start(StartCmd),
}

#[derive(Debug, Clap)]
pub struct ListCmd {}

#[derive(Debug, Clap)]
pub struct StartCmd {
    #[clap(parse(from_os_str), short='s', long="steam-dir", )]
    pub steam_dir: Option<PathBuf>,

    #[clap()]
    pub app: String,

    #[clap()]
    pub args: Vec<String>,
}
