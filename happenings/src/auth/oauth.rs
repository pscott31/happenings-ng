use leptos::ServerFnError;

#[leptos::server(OAuthLink, "/api", "Url", "generate_oauth_link")]
pub async fn oauth_link() -> Result<String, ServerFnError> { backend::oauth_link().await }
pub async fn oauth_check(ouath_state: String, oauth_code: String) -> Result<String, ServerFnError> {
    backend::oauth_check(ouath_state, oauth_code).await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use crate::auth::session::create_session;
    use crate::person::{DbPerson, NewDbPerson};
    use crate::{config, db, AppState};
    use axum::extract::Host;
    use leptos::use_context;
    use leptos::ServerFnError::{self, ServerError};
    use oauth2::reqwest::async_http_client;
    use oauth2::{basic::BasicClient, CsrfToken, PkceCodeChallenge, RedirectUrl};
    use oauth2::{AuthorizationCode, PkceCodeVerifier, Scope, TokenResponse};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct OAuth2State {
        pkce_code_verifier: String,
        return_url: String,
    }

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct OpenIDUserInfo {
        given_name: String,
        family_name: String,
        picture: String,
        email: String,
        email_verified: bool,
        sub: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum Credentials {
        OAuth,
        Password { hash: String, salt: String },
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct DbUser {
        #[serde(flatten)]
        person: DbPerson,
        credentials: Credentials,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct NewDbUser {
        #[serde(flatten)]
        person: NewDbPerson,
        credentials: Credentials,
    }

    enum Fail {
        NoServerState,
        NoHostname,
        DbError(surrealdb::Error),
        NotCreated,
        NoProviders,
        UnknownCsrfId,
        UserInfoQueryError(reqwest::Error),
        UserInfoParseError(reqwest::Error),
        UserEmailNotVerified(String),
        UserNotCreated,
    }

    impl From<Fail> for ServerFnError {
        fn from(fail: Fail) -> Self {
            let msg = match fail {
                Fail::NoServerState => format!("no server state"),
                Fail::NoHostname => format!("no hostname"),
                Fail::DbError(e) => format!("database error: {:?}", e),
                Fail::NotCreated => format!("oauth state record not created"),
                Fail::NoProviders => format!("no oauth providers configured"),
                Fail::UnknownCsrfId => format!("unknown oauth csrf id"),
                Fail::UserInfoQueryError(e) => format!("failed to query userinfo {:?}", e),
                Fail::UserInfoParseError(e) => format!("failed to query userinfo {:?}", e),
                Fail::UserEmailNotVerified(e) => format!("email address not verified : {:?}", e),
                Fail::UserNotCreated => format!("failed to create new user"),
            };
            ServerError(msg)
        }
    }

    pub async fn oauth_link() -> Result<String, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(Fail::NoServerState)?;
        let hostname = use_context::<Host>().ok_or(Fail::NoHostname)?.0;

        let cfg = first_oauth_config(&app_state)?;
        let client = build_oauth_client(&cfg, hostname);

        let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf_state) = client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid profile email".to_string()))
            .set_pkce_challenge(pkce_code_challenge)
            .url();

        let _record: db::Record = app_state
            .db
            .create(("oauth2_state", csrf_state.secret().clone()))
            .content(OAuth2State {
                pkce_code_verifier: pkce_code_verifier.secret().clone(),
                return_url: "/".to_string(),
            })
            .await
            .map_err(|e| Fail::DbError(e))?
            .ok_or(Fail::NotCreated)?;

        Ok(authorize_url.to_string())
    }

    pub async fn oauth_check(state: String, code: String) -> Result<String, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(Fail::NoServerState)?;
        let hostname = use_context::<Host>().ok_or(Fail::NoHostname)?.0;

        let state = CsrfToken::new(state);
        let code = AuthorizationCode::new(code);

        let oauth_state: OAuth2State = app_state
            .db
            .delete(("oauth2_state", state.secret()))
            .await
            .map_err(|e| Fail::DbError(e))?
            .ok_or(Fail::UnknownCsrfId)?;

        let cfg = first_oauth_config(&app_state)?;
        let client = build_oauth_client(&cfg, hostname);

        let token_response = client
            .exchange_code(code)
            .set_pkce_verifier(PkceCodeVerifier::new(oauth_state.pkce_code_verifier))
            .request_async(async_http_client)
            .await?;

        let access_token = token_response.access_token().secret();

        let url =
            "https://www.googleapis.com/oauth2/v3/userinfo?oauth_token=".to_owned() + access_token;
        let user_info = reqwest::get(url)
            .await
            .map_err(|e| Fail::UserInfoQueryError(e))?
            .json::<OpenIDUserInfo>()
            .await
            .map_err(|e| Fail::UserInfoParseError(e))?;
        dbg!(&user_info);

        if !user_info.email_verified {
            return Err(Fail::UserEmailNotVerified(user_info.email).into());
        }

        let mut result = app_state
            .db
            .query("SELECT * FROM person where email=$email;")
            .bind(("email", &user_info.email))
            .await?;

        let mut people: Vec<DbPerson> = result.take(0)?;

        let person = match people.pop() {
            Some(person) => person,
            None => app_state
                .db
                .create("person")
                .content(NewDbUser {
                    person: NewDbPerson {
                        given_name: user_info.given_name,
                        family_name: user_info.family_name,
                        picture: Some(user_info.picture),
                        phone: None,
                        email: user_info.email,
                    },
                    credentials: Credentials::OAuth,
                })
                .await?
                .pop()
                .ok_or(Fail::UserNotCreated)?,
        };

        // // Create the session
        let session = create_session(app_state.db, person.id.id.to_string()).await?;
        Ok(session)
    }

    fn build_oauth_client(cfg: &config::OAuthProvider, hostname: String) -> BasicClient {
        let scheme = if hostname.starts_with("localhost") {
            "http"
        } else {
            "https"
        };
        let redirect_url = format!("{}://{}/oauth_return", scheme, hostname);
        BasicClient::new(
            cfg.client_id.clone(),
            Some(cfg.client_secret.clone()),
            cfg.auth_url.clone(),
            Some(cfg.token_url.clone()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_url).unwrap())
    }

    fn first_oauth_config(state: &AppState) -> Result<config::OAuthProvider, Fail> {
        state
            .config
            .login
            .oauth_providers
            .first()
            .cloned()
            .ok_or(Fail::NoProviders)
    }
}

