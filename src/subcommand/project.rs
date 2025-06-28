mod post;

use {crate::MainError, clap::ArgMatches};

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let (subcommand, args) = args.remove_subcommand().unwrap();
    match subcommand.as_str() {
        "post" => post::execute(args),
        _ => unreachable!(),
    }
}
