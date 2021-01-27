#[derive(sqlx::Type, Debug, Clone)]
#[sqlx(rename = "registration", rename_all = "lowercase")]
pub enum Registration {
    Open,
    Invite,
    Closed,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Server {
    pub name: String,
    pub url: String,
    pub server_name: String,
    pub logo_url: Option<String>,
    pub admins: Vec<String>,
    pub categories: Vec<String>,
    pub rules: String,
    pub description: String,
    pub registration_status: Registration,
    pub verified: bool,
}
