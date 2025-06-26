pub mod auth;

use {
    crate::{root_command, MainError},
    clap::ArgMatches,
};

pub fn execute((subcommand, mut args): (String, ArgMatches)) -> Result<(), MainError> {
    match subcommand.as_str() {
        "auth" => auth::execute(args),
        _ => unreachable!(),
    }
}
