use {
    crate::{env, MainError},
    clap::ArgMatches,
    std::{
        borrow::Cow,
        ffi::OsString,
        process::Command,
    },
};

pub(super) fn execute(mut args: ArgMatches, key: &str) -> Result<(), MainError> {
    let command = args
        .remove_one("editor")
        .map(Cow::Owned)
        .or_else(|| {
            // SAFETY: this is single threaded
            unsafe {
                env::var(c"VISUAL")
            }
                .or_else(|| unsafe { env::var(c"EDITOR") })
                .map(Cow::Borrowed)
        })
        .expect("this argument should be required if `VISUAL` and `EDITOR` are unset");
    let args = args.remove_many::<String>("arg").unwrap_or_default();

    let mut command = Command::new(command);
    command.args(args);
    // println!("{:?}", command);
    Ok(())
}
