use {
    clap::{builder::TypedValueParser, Arg, Command},
    std::{
        ffi::OsStr,
        path::{Path, PathBuf},
    },
};

#[derive(Clone, Copy)]
pub struct CommandValueParser;
impl TypedValueParser for CommandValueParser {
    type Value = PathBuf;

    fn parse_ref(
        &self,
        command: &Command,
        arg: Option<&Arg>,
        path: &OsStr,
    ) -> Result<PathBuf, clap::Error> {
        let path = Path::new(path);

        if path.is_file() {
            Ok(PathBuf::from(path))
        } else {
            let mut error =
                clap::Error::new(clap::error::ErrorKind::ValueValidation).with_cmd(command);
            if let Some(arg) = arg {
                error.insert(
                    clap::error::ContextKind::InvalidArg,
                    clap::error::ContextValue::String(arg.to_string()),
                );
            }
            error.insert(
                clap::error::ContextKind::InvalidValue,
                clap::error::ContextValue::String(path.display().to_string()),
            );

            Err(error)
        }
    }
}
