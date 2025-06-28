mod auth;
mod devlog;
mod ship;

use {crate::MainError, clap::ArgMatches};

pub struct RootConfig {
    async_upload: bool,
}

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let config = RootConfig {
        async_upload: args.remove_one("async-upload").unwrap_or_default(),
    };
    let (subcommand, args) = args.remove_subcommand().unwrap();

    match subcommand.as_str() {
        "auth" => auth::execute(args),
        "devlog" => devlog::execute(args, &config),
        "ship" => ship::execute(args, &config),
        _ => unreachable!(),
    }
}
