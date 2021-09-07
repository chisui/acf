use std::{
    io,
    env,
    str,
    collections::HashMap,
    process::{self, Command, Stdio},
    fs::File,
    path::PathBuf,
};
use clap::Clap;
use thiserror::Error;
use steamacf::{AcfTokenStream, AcfToken, StreamError, StructuredAcfTokenStream};
mod cli;
use crate::cli::{SteamLaunchArgs, SteamLaunchCmds, ListCmd, StartCmd};


#[derive(Debug, Error)]
pub enum SteamLaunchError {
    #[error("Steam file malformed: {0}")]
    Stream(#[from] StreamError),
    #[error("Generic I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("password command failed unexpectedly: {0:?}")]
    PasswordCmdFailed(String, i32),
    #[error("Child failed")]
    ChildFailed(i32),
}
type Res<A> = Result<A, SteamLaunchError>;

impl SteamLaunchArgs {
    fn exec(&self) -> Res<()> {
        let steam_dir = steam_dir(&self.steam_dir)?;
        
        self.cmd.exec(SteamLaunchCtx {
            steam_dir,
            user: self.user.clone(),
            password: self.password.clone(),
            password_cmd: self.password_cmd.clone(),
        })
    }
}

struct SteamLaunchCtx {
    steam_dir: PathBuf,
    user: Option<String>,
    password: Option<String>,
    password_cmd: Option<String>,
}
impl SteamLaunchCtx {
    fn login(&self) -> Res<Option<(String, String)>> {
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
    fn password(&self) -> Res<Option<String>> {
        match &self.password_cmd {
            Some(cmd) => {
                let output = Command::new(cmd).output()?;
                if let Some(status) = output.status.code() {
                    Err(match str::from_utf8(&output.stderr) {
                        Ok(err) => SteamLaunchError::PasswordCmdFailed(err.to_string(), status),
                        Err(_)  => SteamLaunchError::PasswordCmdFailed("failed with unreadable error message".to_string(), status),
                    })
                } else {
                    let raw = str::from_utf8(&output.stdout)
                        .map_err(|_| SteamLaunchError::PasswordCmdFailed("malformed output".to_string(), 1))?;
                    Ok(Some(raw.trim().to_owned()))
                }
            },
            None => Ok(self.password.clone()),
        }
    }
    fn load_registry(&self) -> Res<SteamRegistry> {
        let mut reg_file = self.steam_dir.clone();
        reg_file.push("registry.vdf");
        
        let f = File::open(reg_file)?;
        let mut tokens = StructuredAcfTokenStream::new(AcfTokenStream::new(f));
        tokens.select_path(&["Registry", "HKCU", "Software", "Valve", "Steam", "Apps"])?;

        tokens.expect(AcfToken::DictStart)?;
        let mut reg = HashMap::new();
        while let Some(AcfToken::String(id)) = tokens.try_next()? {
            tokens.expect(AcfToken::DictStart)?;
            if let Some(_) = tokens.select("name")? {
                match tokens.expect_next()? {
                    AcfToken::String(name) => { reg.insert(name, id); },
                    t => { return Err(SteamLaunchError::from(StreamError::UnexpectedToken(t))); },
                }
                tokens.close_dict()?;
            }
        }
        Ok(SteamRegistry(reg))
    }
}
#[derive(Debug)]
struct SteamRegistry(HashMap<String, String>);


trait Cmd {
    fn exec(&self, ctx: SteamLaunchCtx) -> Res<()>;
}
impl Cmd for SteamLaunchCmds {
    fn exec(&self, ctx: SteamLaunchCtx) -> Res<()> {
        match self {
            SteamLaunchCmds::Start(cmd)      => cmd.exec(ctx),
            SteamLaunchCmds::List(cmd)       => cmd.exec(ctx),
        }
    }
}

impl Cmd for ListCmd {
    fn exec(&self, ctx: SteamLaunchCtx) -> Res<()> {        
        let SteamRegistry(registry) = ctx.load_registry()?;
        let max_key_len = registry
            .keys()
            .map(|k| k.len())
            .max()
            .unwrap_or_default();
        for (k, v) in registry {
            print!("{}", k);
            for _ in 0 .. (2 + max_key_len - k.len()) {
                print!(" ");
            }
            println!("{}", v);
        }
        Ok(())
    }
}

impl Cmd for StartCmd {
    fn exec(&self, ctx: SteamLaunchCtx) -> Res<()> {
        let app_id = self.app.clone();
        let mut cmd = Command::new("echo");
        if let Some((user, password)) = ctx.login()? {
            cmd.args(&["-login", user.as_str(), password.as_str()]);
        }
        let out = cmd
            .arg("-applaunch")
            .arg(app_id)
            .args(self.args.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;
        if let Some(s) = out.status.code() {
            Err(SteamLaunchError::ChildFailed(s))
        } else {
            Ok(())
        }
    }
}

fn steam_dir(steam_dir: &Option<PathBuf>) -> Res<PathBuf> {
    steam_dir.clone()
        .or_else(|| env::var("STEAM_DIR").ok().map(PathBuf::from))
        .or_else(|| {
            let mut path = env::var("HOME")
                .ok()
                .map(PathBuf::from)?;
            path.push(".steam");
            Some(path)
        }).ok_or(io::Error::new(io::ErrorKind::NotFound, "Environment varibale HOME, not defined.").into())
}

fn main() {
    process::exit(match SteamLaunchArgs::parse().exec() {
        Ok(_) => 0,
        Err(e) => {
            println!("{}", e);
            match e {
                SteamLaunchError::PasswordCmdFailed(_, i) => i,
                SteamLaunchError::ChildFailed(i) => i,
                _ => 1,
            }
        }
    })
}
