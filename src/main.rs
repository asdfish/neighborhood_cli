mod api;
mod cache;
mod env;
mod subcommand;

use {
    cfg_if::cfg_if,
    clap::{
        ArgAction,
        builder::{Arg, Command, NonEmptyStringValueParser},
    },
    std::{
        borrow::Cow,
        fmt::{self, Display, Formatter, Write},
        io,
        path::{Path, PathBuf},
        process::ExitCode,
    },
    toml_edit::TomlError,
};

const NAME: &str = "neighborhood_cli";
const VERSION: &str = "1.0.1";

fn root_command() -> Command {
    Command::new(NAME)
        .about("Cli for the hackclub's neighborhood event")
        .version(VERSION)
        .subcommand_required(true)
        .subcommand(
            Command::new("auth")
                .about("Login/signup into neighborhood")
                .arg(
                    Arg::new("email")
                        .help("The email that will be used for authentication")
                        .value_parser(NonEmptyStringValueParser::default())
                        .required(true),
                )
                .subcommand_required(true)
                .subcommand(
                    Command::new("send")
                        .visible_alias("signup")
                        .about("Send an otp to this email address or sign up for neighborhood if this email address is new"),
                )
                .subcommand(
                    Command::new("login")
                        .about("Authenticate using an otp")
                        .arg(
                            Arg::new("otp")
                                .help("The received otp")
                                .value_parser(NonEmptyStringValueParser::default())
                                .required(true),
                        ),
                ),
        )
        .subcommand(
            Command::new("project")
                .about("Manipulate projects")
                .arg(
                    Arg::new("name")
                        .help("The name of this project")
                        .value_parser(NonEmptyStringValueParser::default())
                        .required(true)
                )
                .subcommand_required(true)
                .subcommand(
                    Command::new("update")
                        .about("Update things related to this project")
                        .subcommand_required(true)
                        .arg(
                            Arg::new("async")
                                .help("Enable asynchronous uploads. WARNING: This may not work with large files")
                                .short('a')
                                .action(ArgAction::SetTrue)
                        )
                        .arg(
                            Arg::new("message")
                                .help("What changed between this and the last version")
                                .value_parser(NonEmptyStringValueParser::default())
                        )
                        .subcommand(
                            Command::new("devlog")
                                .about("Post a devlog")
                                .arg(
                                    Arg::new("photobooth")
                                        .help("The path to a video explaining what you did")
                                        .short('p')
                                        .long("photobooth")
                                        .value_name("path")
                                        .value_parser(NonEmptyStringValueParser::default())
                                        .required(true),
                                )
                                .arg(
                                    Arg::new("demo")
                                        .help("The path to a video showcasing your product")
                                        .short('d')
                                        .long("demo")
                                        .value_name("path")
                                        .value_parser(NonEmptyStringValueParser::default())
                                        .required(true),
                                )
                        )
                        .subcommand(
                            Command::new("release")
                                .visible_alias("ship")
                                .about("Post a new release")
                                .arg(
                                    Arg::new("editor")
                                        .short('E')
                                        .long("editor")
                                        .help("The editor used for editing the form. If this is unset, `VISUAL` and `EDITOR` will bve used instead and will raise an error if those are also unset")
                                        .value_parser(NonEmptyStringValueParser::default())
                                )
                                .arg(
                                    Arg::new("arg")
                                        .short('a')
                                        .long("arg")
                                        .help("Arguments passed to the editor")
                                        .value_parser(NonEmptyStringValueParser::default())
                                )
                                .arg(
                                    Arg::new("edit")
                                        .short('e')
                                        .long("edit")
                                        .help("If set, this will use <editor> to edit the form. Using this will also enable more things to be edited")
                                        .action(ArgAction::SetTrue)
                                )
                                .arg(
                                    Arg::new("reset")
                                        .short('r')
                                        .long("reset")
                                        .help("If set, this will reset your cached form")
                                        .action(ArgAction::SetTrue)
                                        .requires("edit")
                                )
                                .arg(
                                    Arg::new("no-confirm")
                                        .short('y')
                                        .long("no-confirm")
                                        .help("Disable prompts, answering yes to them all")
                                        .action(ArgAction::SetTrue)
                                )
                        )
                )
        )
}

fn main() -> ExitCode {
    (|| subcommand::execute(root_command().get_matches()))()
        .map(|_| ExitCode::SUCCESS)
        .unwrap_or_else(|err| {
            eprintln!("{err}");
            ExitCode::FAILURE
        })
}

#[derive(Debug)]
pub enum MainError {
    CreateClient(reqwest::Error),
    CreateDirectory(io::Error, Cow<'static, Path>),
    CreateFile(io::Error, PathBuf),
    CreateRuntime(io::Error),
    CreateTempDir(io::Error),
    DecodeResponse(serde_json::Error, String),
    ExecuteCommand(io::Error, String),
    ReadLine(io::Error),
    GetCache,
    GetToken,
    NoEditor,
    NonExistantProject(String),
    ParseReleaseConfig(TomlError),
    ReadFile(io::Error, Cow<'static, Path>),
    WriteFile(io::Error, Cow<'static, Path>),
    ExecuteRequest(reqwest::Error),
    Server(Option<String>),
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
            Self::CreateRuntime(error) => write!(f, "failed to create runtime: {error}"),
            Self::CreateTempDir(error) => write!(f, "failed to create temporary directory: {error}"),
            Self::DecodeResponse(error, response) => write!(f, "failed to decode response `{response}`: {error}"),
            Self::ExecuteCommand(error, command) => write!(f, "failed to execute command `{command}`: {error}"),
            Self::ReadLine(error) => write!(f, "failed to read input: {error}"),
            Self::NoEditor => f.write_str("failed to get editor: flag `--editor` was not specified and both environment variables `VISUAL` and `EDITOR` were not set"),
            Self::NonExistantProject(project) => write!(f, "project `{project}` does not exist"),
            Self::ParseReleaseConfig(error) => write!(f, "failed to read release config:\n{error}\nRun `neighborhood_cli project <project> post ship -m <message> -e` to edit"),
            Self::ReadFile(error, path) => write!(
                f,
                "failed to read file at path `{}`: {error}",
                path.display()
            ),
            Self::WriteFile(error, path) => write!(
                f,
                "failed to write to file at path `{}`: {error}",
                path.display()
            ),
            Self::GetCache =>
{
        f.write_str("failed to get the cache directory, please ensure that you have the following environment variables set:")
                .and_then(|_| {
                    cfg_if! {
                        if #[cfg(target_os = "macos")] {
                            const ENV_VARS: &[&str] = &["HOME"];
                        } else if #[cfg(unix)] {
                            const ENV_VARS: &[&str] = &["XDG_CACHE_HOME", "HOME"];
                        } else if #[cfg(windows)] {
                            const ENV_VARS: &[&str] = &["LOCALAPPDATA"];
                        } else {
                            const ENV_VARS: &[&str] = &[];
                        }
                    }

                    ENV_VARS
                        .iter()
                        .try_for_each(|env_var| {
                            f
                                .write_str(env_var)
                                .and_then(|_| f.write_char('\n'))
                        })
                })
            }
            Self::GetToken => f.write_str("failed to get token, please run `neighborhood_cli auth <EMAIL> send` and `neighborhood_cli auth <EMAIL> login <OTP>` first"),
            Self::ExecuteRequest(error) => write!(f, "failed to execute request: {error}"),
            Self::Server(Some(error)) => write!(f, "the backend responded with an error: {error}"),
            Self::Server(None) => f.write_str("the backend responded with an unknown error"),
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
            };
        }
        test_metadata_sync!(NAME, "CARGO_PKG_NAME");
        test_metadata_sync!(VERSION, "CARGO_PKG_VERSION");
    }
}
