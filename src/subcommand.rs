pub mod auth;
pub mod tag;

use {crate::MainError, clap::ArgMatches};

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let (subcommand, args) = args.remove_subcommand().unwrap();

    match subcommand.as_str() {
        "auth" => auth::execute(args),
        "tag" => tag::execute(args),
        _ => unreachable!(),
    }
}
