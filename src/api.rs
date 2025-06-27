use {
    reqwest::blocking::{Client, Request, RequestBuilder},
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    std::fmt::{self, Display, Formatter},
};

pub trait PostApi {
    const PATH: &'static str;
    type Response: DeserializeOwned;

    fn configure_request(&self, _: RequestBuilder) -> RequestBuilder;
    fn execute_request(&self, client: &Client) -> Result<Self::Response, reqwest::Error> {
        self.configure_request(client.post(&format!(
            "https://neighborhood.hackclub.com/api/{}",
            Self::PATH
        )))
        .send()
        .and_then(|response| response.json::<Self::Response>())
    }
}

pub trait JsonPostApi: Serialize {
    const PATH: &'static str;
    type Response: DeserializeOwned;
}
impl<T> PostApi for T
where
    T: JsonPostApi,
{
    const PATH: &'static str = T::PATH;
    type Response = <T as JsonPostApi>::Response;

    fn configure_request(&self, request: RequestBuilder) -> RequestBuilder {
        request.json(self)
    }
}

#[derive(Deserialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signup<'a> {
    pub email: &'a str,
}
impl JsonPostApi for Signup<'_> {
    const PATH: &'static str = "signup";
    type Response = MessageResponse;
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyOtp<'a> {
    pub email: &'a str,
    pub otp: &'a str,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyOtpResponse {
    pub message: String,
    pub token: String,
}
impl JsonPostApi for VerifyOtp<'_> {
    const PATH: &'static str = "verifyOTP";
    type Response = VerifyOtpResponse;
}
