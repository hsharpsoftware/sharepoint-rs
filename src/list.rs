extern crate uuid;

use super::*;
use self::uuid::Uuid;
use auth::*;

#[derive(Debug, Deserialize, Default)]
pub struct List {
    #[serde(rename = "Id", default)]
    pub id: Uuid,
}

pub fn get_list_by_title(
    title: String,
    access_token_cookies: AccessTokenCookies,
    digest: String,
    host: String,
) -> Option<List> {
    get_data(
        GET_LIST_URL.replace("{title}", &title).replace(
            "{host}",
            &host,
        ),
        access_token_cookies,
        digest,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn get_list_by_title_works() {
        let (user_name, password, host) = auth::tests::login_params();
        let security_token = get_security_token(
            host.to_string(),
            user_name.to_string(),
            password.to_string(),
        );

        let access_token_cookies = get_access_token_cookies(host.to_string(), security_token);
        let digest = get_the_request_digest(host.to_string(), access_token_cookies.clone());
        let title = env::var("RUST_TITLE").unwrap().to_string();

        let list = get_list_by_title(title, access_token_cookies, digest, host).unwrap();
        println!("ID: {}", list.id);
    }
}
