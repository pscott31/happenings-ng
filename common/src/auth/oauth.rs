use leptos::ServerFnError;

// This guy generates an OAuth link by making a PkceCodeChallenge and storing it in the database
// then generating a link to your OAuth provider (currently only one is supported, it would be
// easy to expand it to multiple providers)
//
// Make this server function GetJSON so that it listens to GET requests, not POST as we directly
// visit this endpoint in the browser.
#[leptos::server(OAuthRedirect, "/api", "GetJson", "oauth_redirect")]
pub async fn redirect() -> Result<String, ServerFnError> {
    let res = backend::redirect().await;
    res
}

// Once the login is completed the OAuth provider will navigate us to this return page.
// with a csrf token in the 'state' and an authorization code. We use the authorization code
// to query the oauth provider to get OpenID identity information like email address and name.
//
// We create 'person' if one doesn't exist, then generate a session token and pop it in the
// database.
#[leptos::server(OAuthCheck, "/api", "Url", "oauth_check")]
pub async fn check(ouath_state: String, oauth_code: String) -> Result<String, ServerFnError> {
    backend::check(ouath_state, oauth_code).await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use crate::auth::session::create_session;
    use crate::auth::{Credentials, NewDbUser};
    use crate::person::{DbPerson, NewDbPerson};
    use crate::{config, db, AppState};
    use axum::extract::Host;
    use leptos::logging::warn;
    use leptos::use_context;
    use leptos::ServerFnError::{self, ServerError};
    use leptos_axum_hack;
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

    #[derive(Debug)]
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
                Fail::NoServerState => "no server state".to_string(),
                Fail::NoHostname => "no hostname".to_string(),
                Fail::DbError(e) => format!("database error: {:?}", e),
                Fail::NotCreated => "oauth state record not created".to_string(),
                Fail::NoProviders => "no oauth providers configured".to_string(),
                Fail::UnknownCsrfId => "unknown oauth csrf id".to_string(),
                Fail::UserInfoQueryError(e) => format!("failed to query userinfo {:?}", e),
                Fail::UserInfoParseError(e) => format!("failed to query userinfo {:?}", e),
                Fail::UserEmailNotVerified(e) => format!("email address not verified : {:?}", e),
                Fail::UserNotCreated => "failed to create new user".to_string(),
            };
            ServerError(msg)
        }
    }

    pub async fn redirect() -> Result<String, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(Fail::NoServerState)?;
        let hostname = use_context::<Host>().ok_or(Fail::NoHostname)?.0;

        let cfg = first_config(&app_state)?;
        let client = client(&cfg, hostname);

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
            .map_err(Fail::DbError)?
            .ok_or(Fail::NotCreated)?;

        leptos_axum_hack::redirect(authorize_url.to_string().as_ref());
        Ok("redirecting!".to_string())
    }

    pub async fn check(state: String, code: String) -> Result<String, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(Fail::NoServerState)?;
        let hostname = use_context::<Host>().ok_or(Fail::NoHostname)?.0;

        let state = CsrfToken::new(state);
        let code = AuthorizationCode::new(code);

        let oauth_state: Result<Option<OAuth2State>, _> = app_state
            .db
            .delete(("oauth2_state", state.secret()))
            .await
            .map_err(Fail::DbError);
        warn!("oauth state: {:?}", &oauth_state);
        let oauth_state = oauth_state?;

        let oauth_state = oauth_state.ok_or(Fail::UnknownCsrfId)?;

        let cfg = first_config(&app_state)?;
        let client = client(&cfg, hostname);

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
            .map_err(Fail::UserInfoQueryError)?
            .json::<OpenIDUserInfo>()
            .await
            .map_err(Fail::UserInfoParseError)?;
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

        // Create the session
        let session = create_session(person.id.into()).await?;
        Ok(session)
    }

    fn client(cfg: &config::OAuthProvider, hostname: String) -> BasicClient {
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

    fn first_config(state: &AppState) -> Result<config::OAuthProvider, Fail> {
        state
            .config
            .login
            .oauth_providers
            .first()
            .cloned()
            .ok_or(Fail::NoProviders)
    }
}

