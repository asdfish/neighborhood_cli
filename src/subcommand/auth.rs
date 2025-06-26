use {
    crate::{
        api::{PostApi, Signup, SignupResponse, VerifyOtp, VerifyOtpResponse},
        MainError,
    },
    clap::ArgMatches,
    reqwest::blocking::Client,
    rustyline::{error::ReadlineError, DefaultEditor},
    std::any::Any,
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
                .map(|VerifyOtpResponse { token, .. }| {})
            }
            _ => unreachable!(),
        })
}
