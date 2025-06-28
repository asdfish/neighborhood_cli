mod api;
mod cache;
mod env;
mod subcommand;

use {
    crate::cache::GetCacheError,
    clap::{
        builder::{Arg, Command, NonEmptyStringValueParser, Resettable},
        ArgAction,
    },
    std::{
        fmt::{self, Display, Formatter},
        io,
        path::PathBuf,
        process::ExitCode,
    },
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
                    Command::new("post")
                        .about("Post things related to this project")
                        .subcommand_required(true)
                        .arg(
                            Arg::new("async")
                                .help("Enable asynchronous uploads. WARNING: This may not work with large files.")
                                .short('a')
                                .action(ArgAction::SetTrue)
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
                                .arg(
                                    Arg::new("message")
                                        .help("A message describing what you did")
                                        .short('m')
                                        .long("message")
                                        .value_parser(NonEmptyStringValueParser::default())
                                        .required(true),
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
enum MainError {
    CreateClient(reqwest::Error),
    CreateDirectory(io::Error, PathBuf),
    CreateFile(io::Error, PathBuf),
    CreateRuntime(io::Error),
    DecodeResponse(serde_json::Error, String),
    GetCache(GetCacheError),
    GetToken,
    ReadFile(io::Error, PathBuf),
    WriteFile(io::Error, PathBuf),
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
            Self::DecodeResponse(error, response) => write!(f, "failed to decode response `{response}`: {error}"),
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
            Self::GetCache(error) => error.fmt(f),
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
                pp
            };
        }
        test_metadata_sync!(NAME, "CARGO_PKG_NAME");
        test_metadata_sync!(VERSION, "CARGO_PKG_VERSION");
    }
}
