use {
    crate::{MainError, api::MessageResponse, cache::PathCache},
    clap::ArgMatches,
    reqwest::blocking::{Client, Response},
    serde::{Deserialize, Serialize},
    std::{
        fs::{DirBuilder, File},
        io::Write,
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
                    })
            }
            _ => unreachable!(),
        })
}
