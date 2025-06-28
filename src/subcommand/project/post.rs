mod devlog;

use {crate::MainError, clap::ArgMatches};

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let async_upload = args.remove_one("async").unwrap_or_default();
    let (subcommand, args) = args.remove_subcommand().unwrap();

    match subcommand.as_str() {
        "devlog" => devlog::execute(args, async_upload),
        _ => unreachable!(),
    }
}
