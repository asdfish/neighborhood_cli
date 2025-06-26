use {
    crate::{
        api::{PostApi, Signup, SignupResponse, VerifyOtp, VerifyOtpResponse},
        directories::Directory,
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
                    .create_request(&client)
                    .map_err(MainError::ExecuteRequest)
                    .map(|SignupResponse { message }| {
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
                .create_request(&client)
                .map_err(MainError::ExecuteRequest)
                .inspect(|VerifyOtpResponse { message, .. }| eprintln!("{message}"))
                .and_then(|VerifyOtpResponse { token, .. }| {
                    Directory::Cache
                        .get()
                        .map_err(MainError::GetDirectory)
                        .and_then(
                            |path| match DirBuilder::new().recursive(true).create(&path) {
                                Ok(_) => Ok(path),
                                Err(err) => Err(MainError::CreateDirectory(err, path)),
                            },
                        )
                        .map(|mut dir| {
                            dir.push("token");
                            dir
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
