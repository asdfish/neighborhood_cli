use {
    crate::{
        api::{JsonPostApi, MessageResponse, PostApi, Signup, VerifyOtp, VerifyOtpResponse},
        cache::PathCache,
        MainError,
    },
    clap::ArgMatches,
    reqwest::blocking::Client,
    rustyline::{error::ReadlineError, DefaultEditor},
    std::{
        any::Any,
        fs::{DirBuilder, File},
        io::Write,
    },
};

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    Client::builder()
        .build()
        .map_err(MainError::CreateClient)
        .and_then(|client| match args.subcommand().unwrap() {
            ("send", _) => {
                let email = args.remove_one::<String>("email").unwrap();
                Signup { email: &email }
                    .execute_request(&client)
                    .map_err(MainError::ExecuteRequest)
                    .map(|MessageResponse { message }| {
                        eprintln!("{message}");
                    })
            }
            ("login", login_args) => {
                let email = args.get_one::<String>("email").unwrap();
                let otp = login_args.get_one::<String>("otp").unwrap();
                VerifyOtp {
                    email: &email,
                    otp: &otp,
                }
                .execute_request(&client)
                .map_err(MainError::ExecuteRequest)
                .inspect(|VerifyOtpResponse { message, .. }| eprintln!("{message}"))
                .and_then(|VerifyOtpResponse { token, .. }| {
                    PathCache::default()
                        .into_token()
                        .map_err(MainError::GetCache)
                        .and_then(|path| {
                            match DirBuilder::new()
                                .recursive(true)
                                .create(&path.parent().unwrap_or(&path))
                            {
                                Ok(_) => Ok(path),
                                Err(err) => Err(MainError::CreateDirectory(err, path)),
                            }
                        })
                        .and_then(|path| match File::create(&path) {
                            Ok(mut file) => file
                                .write_all(token.as_bytes())
                                .and_then(|_| file.flush())
                                .map_err(|err| MainError::WriteFile(err, path)),
                            Err(err) => Err(MainError::CreateFile(err, path)),
                        })
                })
            }
            _ => unreachable!(),
        })
}
