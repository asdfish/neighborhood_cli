mod api;
mod cache;
mod subcommand;

use {
    crate::cache::GetCacheError,
    cfg_if::cfg_if,
    clap::{
        builder::{Arg, Command},
        ArgAction,
    },
    rustyline::error::ReadlineError,
    std::{
        ffi::{c_char, c_int},
        fmt::{self, Display, Formatter, Write},
        fs, io,
        path::PathBuf,
        process::ExitCode,
    },
};

const NAME: &str = "neighborhood_cli";
const VERSION: &str = "0.1.0";

fn root_command() -> Command {
    Command::new(NAME)
        .about("Cli for the hackclub's neighborhood event")
        .version(VERSION)
        .subcommand(
            Command::new("auth")
                .about("Login/signup into neighborhood")
                .arg(
                    Arg::new("email")
                        .help("The email that will be used for authentication")
                        .value_name("EMAIL")
                        .required(true),
                )
                .subcommand(Command::new("send").about("Send an otp to this email address"))
                .subcommand(
                    Command::new("login")
                        .about("Authenticate using an otp")
                        .arg(
                            Arg::new("otp")
                                .help("The received otp")
                                .value_name("INT")
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            Command::new("tag").about("Tag an achievement").subcommand(
                Command::new("devlog")
                    .about("Tag an achievement with a video vlog")
                    .arg(
                        Arg::new("photobooth")
                            .help("The video containing your explaination")
                            .value_name("PATH")
                            .short('p'),
                    )
                    .arg(
                        Arg::new("demo")
                            .help("The video containing a demo")
                            .value_name("PATH")
                            .short('d'),
                    )
                    .arg(
                        Arg::new("message")
                            .help("What did you do in this devlog?")
                            .value_name("STRING")
                            .short('m'),
                    ),
            ),
        )
}

fn main() -> ExitCode {
    (|| {
        root_command()
            .try_get_matches()
            .map(subcommand::execute)
            .unwrap()
    })()
    .map(|_| ExitCode::SUCCESS)
    .unwrap_or_else(|err| {
        eprintln!("{err}");
        ExitCode::FAILURE
    })
}

enum MainError {
    CreateClient(reqwest::Error),
    CreateDirectory(io::Error, PathBuf),
    CreateFile(io::Error, PathBuf),
    GetCache(GetCacheError),
    GetToken,
    WriteFile(io::Error, PathBuf),
    CreateRequest(reqwest::Error),
    ExecuteRequest(reqwest::Error),
    Readline(ReadlineError),
}
impl Display for MainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Self::CreateClient(error) => write!(f, "failed to create https client: {error}"),
            Self::CreateDirectory(error, path) => write!(
                f,
                "failed to create directory at path `{}`: {error}",
                path.display()
            ),
            Self::CreateFile(error, path) => write!(
                f,
                "failed to create file at path `{}`: {error}",
                path.display()
            ),
            Self::WriteFile(error, path) => write!(
                f,
                "failed to write to file at path `{}`: {error}",
                path.display()
            ),
            Self::GetCache(error) => error.fmt(f),
            Self::GetToken => f.write_str("failed to get token, please run `neighborhood_cli auth <EMAIL> send` and `neighborhood_cli auth <EMAIL> login <OTP>` first"),
            Self::CreateRequest(error) => write!(f, "failed to create request: {error}"),
            Self::ExecuteRequest(error) => write!(f, "failed to execute request: {error}"),
            Self::Readline(ReadlineError::Eof | ReadlineError::Interrupted) => Ok(()),
            Self::Readline(error) => write!(f, "failed to read input: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_sync() {
        macro_rules! test_metadata_sync {
            ($val:expr, $from:literal) => {
                if let Some(from) = option_env!($from) {
                    assert_eq!($val, from);
                }
                pp
            };
        }
        test_metadata_sync!(NAME, "CARGO_PKG_NAME");
        test_metadata_sync!(VERSION, "CARGO_PKG_VERSION");
    }
}
