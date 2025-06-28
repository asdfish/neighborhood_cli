use {
    crate::{subcommand::RootConfig, MainError},
    clap::ArgMatches,
};

mod edit;

pub fn execute(mut args: ArgMatches, config: &RootConfig) -> Result<(), MainError> {
    let key = args.remove_one::<String>("key").unwrap();

    let (subcommand, args) = args.remove_subcommand().unwrap();
    match subcommand.as_str() {
        "edit" => edit::execute(args, &key),
        "post" => todo!(),
        _ => unreachable!(),
    }
}
