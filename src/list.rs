extern crate uuid;

extern crate serde;

use self::uuid::Uuid;
use auth::*;
use data::*;

#[allow(unused_imports)]
use super::*;

use self::serde::de::DeserializeOwned;

#[derive(Debug, Deserialize, Default)]
pub struct List {
    #[serde(rename = "Id", default)]
    pub id: Uuid,
}


#[derive(Debug, Deserialize, Default)]
struct ListItemsContainer<T> 
{
    #[serde(rename = "value", default)]
    results: Vec<T>,
}

static GET_LIST_URL: &'static str = "https://{host}.sharepoint.com/_api/web/lists/GetByTitle('{title}')";
static GET_LIST_ITEMS_URL: &'static str = "https://{host}.sharepoint.com/_api/web/lists/GetByTitle('{title}')/items";

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

pub fn get_list_items_by_title<T>(
    title: String,
    access_token_cookies: AccessTokenCookies,
    digest: String,
    host: String,
) -> Vec<T> 
where
    T: DeserializeOwned + Default + std::fmt::Debug,
{
    let res : Option<ListItemsContainer<T>> = get_data(
        GET_LIST_ITEMS_URL.replace("{title}", &title).replace(
            "{host}",
            &host,
        ),
        access_token_cookies,
        digest,
    );
    println!("res: '{:?}'", res);    
    res.unwrap().results
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

    #[derive(Debug, Deserialize, Default)]
    struct Generic_Item {
        #[serde(rename = "Id", default)]        
        id : i32,
    }

    #[test]
    fn get_list_items_by_title_works() {
        let (user_name, password, host) = auth::tests::login_params();
        let security_token = get_security_token(
            host.to_string(),
            user_name.to_string(),
            password.to_string(),
        );

        let access_token_cookies = get_access_token_cookies(host.to_string(), security_token);
        let digest = get_the_request_digest(host.to_string(), access_token_cookies.clone());
        let title = env::var("RUST_TITLE").unwrap().to_string();

        let items : Vec<Generic_Item> = get_list_items_by_title(title, access_token_cookies, digest, host);

        println!("items: '{:?}'", items);

        assert!( items.len() > 0 );
    }
}
