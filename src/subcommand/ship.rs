use {
    crate::{subcommand::RootConfig, MainError},
    clap::ArgMatches,
};

pub fn execute(args: ArgMatches, config: &RootConfig) -> Result<(), MainError> {
    Ok(())
}
