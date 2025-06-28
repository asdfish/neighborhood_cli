pub mod auth;
pub mod devlog;

use {crate::MainError, clap::ArgMatches};

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let (subcommand, args) = args.remove_subcommand().unwrap();

    match subcommand.as_str() {
        "auth" => auth::execute(args),
        "devlog" => devlog::execute(args),
        _ => unreachable!(),
    }
}
