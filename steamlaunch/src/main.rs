use std::io::{Error, ErrorKind, Result};
use std::collections::HashMap;
use std::array::IntoIter;
use std::env;
use std::str;
use std::fmt;
use std::error;
use std::process::{Command, Stdio};
use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use steamacf::{AcfTokenStream, AcfToken, UnexpectedToken};


#[derive(Debug, StructOpt)]
#[structopt(
    name = "steam-launch",
    about = "launch steam games from the command line"
)]
struct SteamLaunchArgs {
    #[structopt(parse(from_os_str), short="s", long="steam-dir")]
    steam_dir: Option<PathBuf>,

    #[structopt(short, long, env="STEAM_USER")]
    user: Option<String>,
    #[structopt(short, long)]
    password: Option<String>,
    #[structopt(short="x", long="password-cmd", env="STEAM_PASSWORD_CMD")]
    password_cmd: Option<String>,

    #[structopt(subcommand)]
    cmd: SteamLaunchCmds
}
impl SteamLaunchArgs {
    fn exec(&self) -> Result<i32> {
        let steam_dir = steam_dir(&self.steam_dir)?;
        
        self.cmd.exec(SteamLaunchCtx {
            steam_dir,
            user: self.user.clone(),
            password: self.password.clone(),
            password_cmd: self.password_cmd.clone(),
        })
    }
}
#[derive(Debug)]
struct PasswordCmdFailed(Vec<u8>);
impl error::Error for PasswordCmdFailed {}
impl fmt::Display for PasswordCmdFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "password command failed unexpectedly:")?;
        let stderr = str::from_utf8(&self.0)
            .map_err(|_| fmt::Error)?;
        write!(f, "{}", stderr)
    }
}
struct SteamLaunchCtx {
    steam_dir: PathBuf,
    user: Option<String>,
    password: Option<String>,
    password_cmd: Option<String>,
}
impl SteamLaunchCtx {
    fn login(&self) -> Result<Option<(String, String)>> {
        match &self.user {
            Some(user) => {
                let password = self.password()?;
                match password {
                    Some(pw) => Ok(Some((user.clone(), pw))),
                    None => Ok(None),
                }
            },
            None => Ok(None)
        }
    }
    fn password(&self) -> Result<Option<String>> {
        match &self.password_cmd {
            Some(cmd) => {
                let output = Command::new(cmd).output()?;
                if let Some(_) = output.status.code() {
                    Err(Error::new(ErrorKind::Other, PasswordCmdFailed(output.stderr)))
                } else {
                    let raw = str::from_utf8(&output.stdout)
                        .map_err(|_| Error::new(ErrorKind::Other,PasswordCmdFailed("malformed output".as_bytes().to_vec())))?;
                    Ok(Some(raw.trim().to_owned()))
                }
            },
            None => Ok(self.password.clone()),
        }
    }
    fn load_registry(&self) -> Result<SteamRegistry> {
        let mut reg_file = self.steam_dir.clone();
        reg_file.push("registry.vdf");
        
        let f = File::open(reg_file)?;
        let mut tokens = AcfTokenStream::new(f);
        tokens.select_path(IntoIter::new(["Registry", "HKCU", "Software", "Valve", "Steam", "Apps"]))?;

        tokens.expect(AcfToken::DictStart)?;
        let mut reg = HashMap::new();
        while let Some(AcfToken::String(id)) = tokens.try_next()? {
            tokens.expect(AcfToken::DictStart)?;
            if let Some(_) = tokens.select("name")? {
                match tokens.expect_next()? {
                    AcfToken::String(name) => { reg.insert(name, id); },
                    t => { return Err(UnexpectedToken(t).into()); },
                }
                tokens.close_dict()?;
            }
        }
        Ok(SteamRegistry(reg))
    }
}
#[derive(Debug)]
struct SteamRegistry(HashMap<String, String>);

#[derive(Debug, StructOpt)]
enum SteamLaunchCmds {
    Start(StartCmd),
    List(ListCmd),
    Completion(CompletionCmd),
}
trait Cmd {
    fn exec(&self, ctx: SteamLaunchCtx) -> Result<i32>;
}
impl Cmd for SteamLaunchCmds {
    fn exec(&self, ctx: SteamLaunchCtx) -> Result<i32> {
        match self {
            SteamLaunchCmds::Start(cmd)      => cmd.exec(ctx),
            SteamLaunchCmds::List(cmd)       => cmd.exec(ctx),
            SteamLaunchCmds::Completion(cmd) => cmd.exec(ctx),
        }
    }
}

#[derive(Debug, StructOpt)]
enum CompletionCmd {
    Bash,
    Zsh,
}
impl Cmd for CompletionCmd {
    fn exec(&self, _: SteamLaunchCtx) -> Result<i32> {
        println!("{:?}", self);
        Ok(0)
    }
}

#[derive(Debug, StructOpt)]
struct ListCmd {}
impl Cmd for ListCmd {
    fn exec(&self, ctx: SteamLaunchCtx) -> Result<i32> {        
        let registry = ctx.load_registry()?;
        print!("{:?}", registry);
        Ok(0)
    }
}

#[derive(Debug, StructOpt)]
struct StartCmd {
    #[structopt(parse(from_os_str), short="s", long="steam-dir")]
    steam_dir: Option<PathBuf>,

    #[structopt()]
    app: String,

    #[structopt()]
    args: Vec<String>,
}
impl Cmd for StartCmd {
    fn exec(&self, ctx: SteamLaunchCtx) -> Result<i32> {
        let app_id = self.app.clone();
        let mut cmd = Command::new("echo");
        if let Some((user, password)) = ctx.login()? {
            cmd.args(["-login", user.as_str(), password.as_str()]);
        }
        let out = cmd
            .arg("-applaunch")
            .arg(app_id)
            .args(self.args.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        Ok(out.status.code().unwrap_or(0))
    }
}

fn steam_dir(steam_dir: &Option<PathBuf>) -> Result<PathBuf> {
    steam_dir.clone()
        .or_else(|| env::var("STEAM_DIR").ok().map(PathBuf::from))
        .or_else(|| {
            let mut path = env::var("HOME")
                .ok()
                .map(PathBuf::from)?;
            path.push(".steam");
            Some(path)
        }).ok_or(Error::new(ErrorKind::NotFound, "Environment varibale HOME, not defined."))
}

fn main() -> Result<()> {
    let status = SteamLaunchArgs::from_args()
        .exec()?;
    std::process::exit(status)
}
