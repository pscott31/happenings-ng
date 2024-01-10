use oauth2::{AuthUrl, ClientId, ClientSecret, TokenUrl};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Login {
    pub admin_email: String,
    pub admin_password: String,
    pub oauth_providers: Vec<OAuthProvider>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OAuthProvider {
    pub name: String,
    pub icon_url: String,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub auth_url: AuthUrl,
    pub token_url: TokenUrl,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DB {
    pub endpoint: String,
    pub credentials: Option<Credentials>,
    pub namespace: String,
    pub database: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Square {
    pub endpoint: String,
    pub api_key: String,
    pub location_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Credentials {
    Root { username: String, password: String },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub login: Login,
    pub db: DB,
    pub square: Square,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            login: Login {
                admin_email: "admin@admin.com".to_string(),
                admin_password: "admin".to_string(),
                oauth_providers: Vec::new(),
            },
            db: DB {
                endpoint: "file:/happenings.db".to_string(),
                credentials: None,
                namespace: "happenings".to_string(),
                database: "happenings".to_string(),
            },
            // TODO: make an option?
            square: Square {
                endpoint: "https://connect.squareupsandbox.com/v2".to_string(),
                api_key: "".to_string(),
                location_id: "".to_string(),
            },
        }
    }
}

