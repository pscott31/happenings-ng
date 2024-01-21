use leptos::ServerFnError;

#[leptos::server(SignUpPassword, "/api", "Url", "signup_password")]
pub async fn signup_password(
    email: String,
    password: String,
    given_name: String,
    family_name: String,
    phone: Option<String>,
) -> Result<(), ServerFnError> {
    backend::signup(email, password, given_name, family_name, phone).await
}

#[leptos::server(SignInPassword, "/api", "Url", "signin_password")]
pub async fn signin(email: String, password: String) -> Result<String, ServerFnError> {
    backend::signin(email, password).await
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use crate::auth::session::create_session;
    use leptos::{use_context, ServerFnError};
    use rand::distributions::{Alphanumeric, DistString};
    use sha256::Sha256Digest;

    use tracing::*;

    use crate::{db, AppState};

    enum Fail {
        NoServerState,
        DbError(surrealdb::Error),
        NoUser(String),
        WrongCreds(String),
        UserCreateFailed,
        IncorrectPassword(String),
    }

    // TODO: status codes for unauthorized..
    impl From<Fail> for ServerFnError {
        fn from(fail: Fail) -> Self {
            let msg = match fail {
                Fail::NoServerState => format!("no server state"),
                Fail::DbError(e) => format!("database error: {:?}", e),
                Fail::NoUser(email) => {
                    format!("user with email {} does not exist in database", email)
                }
                Fail::WrongCreds(email) => format!(
                    "user with email {} exists, but associated with oauth login",
                    email
                ),
                Fail::UserCreateFailed => format!("user not created in db"),
                Fail::IncorrectPassword(email) => {
                    format!("incorrect password for account with email {}", email)
                }
            };
            ServerFnError::ServerError(msg)
        }
    }

    pub async fn signup(
        email: String,
        password: String,
        given_name: String,
        family_name: String,
        phone: Option<String>,
    ) -> Result<(), ServerFnError> {
        info!("new user: {:?}", email);
        let app_state = use_context::<AppState>().ok_or(Fail::NoServerState)?;

        let salt = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
        let hash = make_hash(&salt, password);

        let _foo: db::Record = app_state
            .db
            .create("person")
            .content(db::NewPerson {
                given_name: given_name,
                family_name: family_name,
                picture: None,
                email: email,
                phone: phone,
                credentials: db::Credentials::Password { hash, salt },
            })
            .await?
            .pop()
            .ok_or(Fail::UserCreateFailed)?;
        Ok(())
    }

    pub async fn signin(email: String, password: String) -> Result<String, ServerFnError> {
        let app_state = use_context::<AppState>().ok_or(Fail::NoServerState)?;

        let mut people: Vec<db::Person> = app_state
            .db
            .query("select * from person where email=$email")
            .bind(("email", &email))
            .await?
            .take(0)?;

        let person = people.pop().ok_or(Fail::NoUser(email.clone()))?;

        match person.credentials {
            db::Credentials::OAuth => return Err(Fail::WrongCreds(email.clone()).into()),
            db::Credentials::Password { hash, salt } => {
                make_hash(salt, password)
                    .eq(&hash)
                    .then_some(())
                    .ok_or(Fail::IncorrectPassword(email))?;
            }
        }

        Ok(create_session(person.id.into()).await?)
    }

    fn make_hash<T, S>(salt: T, pw: S) -> String
    where
        T: AsRef<str>,
        S: AsRef<str>,
    {
        let salted = format!("{}{}", salt.as_ref(), pw.as_ref());
        Sha256Digest::digest(salted)
    }
}

