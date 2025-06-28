//! Shared things between all the apis

use serde::Deserialize;

#[derive(Deserialize)]
pub struct MessageResponse {
    pub message: String,
}
