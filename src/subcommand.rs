pub mod auth;
pub mod devlog;

use {crate::MainError, clap::ArgMatches};

pub struct RootConfig {
    async_upload: bool,
}

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let (subcommand, mut args) = args.remove_subcommand().unwrap();
    let config = RootConfig {
        async_upload: args.remove_one("async-upload").unwrap_or_default(),
    };

    match subcommand.as_str() {
        "auth" => auth::execute(args),
        "devlog" => devlog::execute(args, &config),
        _ => unreachable!(),
    }
}
