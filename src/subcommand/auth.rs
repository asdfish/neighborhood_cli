use {
    crate::{api::MessageResponse, cache::TOKEN, MainError},
    clap::ArgMatches,
    reqwest::blocking::{Client, Response},
    serde::{Deserialize, Serialize},
    std::{
        borrow::Cow,
        fs::{self, DirBuilder},
    },
};

pub fn execute(mut args: ArgMatches) -> Result<(), MainError> {
    Client::builder()
        .build()
        .map_err(MainError::CreateClient)
        .and_then(|client| match args.subcommand().unwrap() {
            ("send" | "signup", _) => {
                #[derive(Serialize)]
                struct Signup<'a> {
                    email: &'a str,
                }

                let email = args.remove_one::<String>("email").unwrap();
                client
                    .post("https://neighborhood.hackclub.com/api/signup")
                    .json(&Signup { email: &email })
                    .send()
                    .and_then(Response::error_for_status)
                    .map_err(MainError::ExecuteRequest)
                    .and_then(|response| {
                        response
                            .text()
                            .map_err(MainError::ExecuteRequest)
                            .and_then(|response| {
                                serde_json::from_str(&response)
                                    .map_err(|error| MainError::DecodeResponse(error, response))
                            })
                            .map(|MessageResponse { message }| {
                                eprintln!("{message}");
                            })
                    })
            }
            ("login", login_args) => {
                #[derive(Serialize)]
                struct VerifyOtp<'a> {
                    email: &'a str,
                    otp: &'a str,
                }
                #[derive(Deserialize)]
                struct VerifyOtpResponse {
                    message: String,
                    token: String,
                }

                let email = args.get_one::<String>("email").unwrap();
                let otp = login_args.get_one::<String>("otp").unwrap();
                client
                    .post("https://neighborhood.hackclub.com/api/verifyOTP")
                    .json(&VerifyOtp {
                        email: &email,
                        otp: &otp,
                    })
                    .send()
                    .and_then(Response::error_for_status)
                    .map_err(MainError::ExecuteRequest)
                    .and_then(|response| {
                        response
                            .text()
                            .map_err(MainError::ExecuteRequest)
                            .and_then(|response| {
                                serde_json::from_str(&response)
                                    .map_err(|error| MainError::DecodeResponse(error, response))
                            })
                            .inspect(|VerifyOtpResponse { message, .. }| eprintln!("{message}"))
                            .and_then(|VerifyOtpResponse { token, .. }| {
                                let path = TOKEN.as_ref().ok_or(MainError::GetCache)?;

                                if let Some(parent) = path.parent() {
                                    if !parent.is_dir() {
                                        DirBuilder::new().recursive(true).create(parent).map_err(
                                            |error| {
                                                MainError::CreateDirectory(
                                                    error,
                                                    Cow::Borrowed(path),
                                                )
                                            },
                                        )?;
                                    }
                                }

                                fs::write(path, token).map_err(|error| {
                                    MainError::WriteFile(error, Cow::Borrowed(path))
                                })
                            })
                    })
            }
            _ => unreachable!(),
        })
}
