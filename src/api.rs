use {
    reqwest::blocking::{Client, Request},
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::fmt::{self, Display, Formatter},
};

pub trait PostApi: Serialize {
    const PATH: &'static str;
    type Response: DeserializeOwned;

    fn create_request(&self, client: &Client) -> Result<Self::Response, reqwest::Error> {
        client
            .post(format!(
                "https://neighborhood.hackclub.com/api/{}",
                Self::PATH
            ))
            .json(self)
            .send()
            .and_then(|response| response.json::<Self::Response>())
    }
}

#[derive(Serialize)]
pub struct Signup<'a> {
    pub email: &'a str,
}
impl PostApi for Signup<'_> {
    const PATH: &'static str = "signup";
    type Response = SignupResponse;
}
#[derive(Deserialize)]
pub struct SignupResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct VerifyOtp<'a> {
    pub email: &'a str,
    pub otp: &'a str,
}
#[derive(Deserialize)]
pub struct VerifyOtpResponse {
    pub message: String,
    pub token: String,
}
impl PostApi for VerifyOtp<'_> {
    const PATH: &'static str = "verifyOTP";
    type Response = VerifyOtpResponse;
}
