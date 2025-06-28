mod update;

use {crate::MainError, clap::ArgMatches};

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    let name = args.remove_one::<String>("name").unwrap();

    let (subcommand, args) = args.remove_subcommand().unwrap();
    match subcommand.as_str() {
        "update" => update::execute(args, &name),
        _ => unreachable!(),
    }
}
