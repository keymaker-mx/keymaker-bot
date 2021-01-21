use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WellKnown {
    pub name: String,
    pub url: String,
    pub server_name: String,
    pub logo_url: Option<String>,
    pub admins: Vec<String>,
    pub categories: Vec<String>,
    pub rules: String,
    pub description: String,
    pub registration_status: ServerRegistrationStatus,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "registration", rename_all = "lowercase")]
pub enum ServerRegistrationStatus {
    Open,
    Invite,
    Closed,
}
