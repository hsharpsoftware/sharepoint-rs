extern crate uuid;

extern crate serde;

use self::uuid::Uuid;
use auth::*;
use data::*;

#[allow(unused_imports)]
use super::*;

use self::serde::de::DeserializeOwned;
use self::serde::ser::Serialize;

#[derive(Debug, Deserialize, Default)]
pub struct List {
    #[serde(rename = "Id", default)]
    pub id: Uuid,
}


#[derive(Debug, Deserialize, Default)]
struct ListItemsContainer<T> {
    #[serde(rename = "value", default)]
    results: Vec<T>,
}

#[derive(Debug, Clone)]
pub struct ListItemType {
    pub name: String,
}

static GET_LIST_URL: &'static str = "{site}/_api/web/lists/GetByTitle('{title}')";
static GET_LIST_ITEMS_URL: &'static str = "{site}/_api/web/lists/GetByTitle('{title}')/items";

pub fn get_list_by_title(
    title: String,
    login : LoginContext
) -> Option<List> {
    let access_token_cookies = login.access_token;
    let digest = login.request_digest;
    let site = login.site;
    get_data(
        GET_LIST_URL.replace("{title}", &title).replace(
            "{site}",
            site.parent
                .to_string()
                .as_str(),
        ),
        access_token_cookies,
        digest,
    )
}

pub fn get_list_default_item_type(list_name: String) -> ListItemType {
    let mut v: Vec<char> = list_name.chars().collect();
    v[0] = v[0].to_uppercase().nth(0).unwrap();
    let s2: String = v.into_iter().collect();

    ListItemType { name: format!("{}{}{}", "SP.Data.", s2, "ListItem") }
}


pub fn get_list_items_by_title<T>(
    title: String,
    access_token_cookies: AccessTokenCookies,
    digest: RequestDigest,
    site: Site,
) -> Vec<T>
where
    T: DeserializeOwned + Default,
{
    let res: Option<ListItemsContainer<T>> = get_data(
        GET_LIST_ITEMS_URL.replace("{title}", &title).replace(
            "{site}",
            site.parent
                .to_string()
                .as_str(),
        ),
        access_token_cookies,
        digest,
    );
    //println!("res: '{:?}'", res);
    res.unwrap().results
}

pub fn add_list_item_by_list_title<T,U>(
    title: String,
    access_token_cookies: AccessTokenCookies,
    digest: RequestDigest,
    site: Site,
    data: U,
    list_item_type: ListItemType,
) -> T
where
    T: DeserializeOwned + Default,
    U: Serialize + Default,
{
    let res: Option<T> = post_data(
        GET_LIST_ITEMS_URL.replace("{title}", &title).replace(
            "{site}",
            site.parent
                .to_string()
                .as_str(),
        ),
        access_token_cookies,
        digest,
        data,
        list_item_type.name,
        false,
    );
    //println!("res: '{:?}'", res);
    res.unwrap()
}

pub fn update_list_item_by_list_title<U>(
    title: String,
    access_token_cookies: AccessTokenCookies,
    digest: RequestDigest,
    site: Site,
    data: U,
    list_item_type: ListItemType,
    id : i32,
) -> ()
where
    U: Serialize + Default,
{
    let res: Option<()> = post_data(
        format!("{}({})", GET_LIST_ITEMS_URL.replace("{title}", &title).replace(
            "{site}",
            site.parent
                .to_string()
                .as_str(),
        ), id ),
        access_token_cookies,
        digest,
        data,
        list_item_type.name,
        true,
    );
    //println!("res: '{:?}'", res);
    res.unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn get_list_by_title_works() {
        let (user_name, password, site) = auth::tests::login_params();
        let login = login( site.parent, user_name, password );
        let title = env::var("RUST_TITLE").unwrap().to_string();

        let list = get_list_by_title(title, login).unwrap();
        println!("ID: {}", list.id);
    }

    #[derive(Debug, Deserialize, Default)]
    struct GenericListItem {
        #[serde(rename = "Id", default)]
        id: i32,
    }

    #[derive(Debug, Deserialize, Default, Serialize)]
    struct GenericListItemWithTitle {
        #[serde(rename = "Id", default)]
        id: i32,
        #[serde(rename = "Title", default)]
        title: String,
    }

    #[derive(Debug, Deserialize, Default, Serialize)]
    struct GenericListItemWithTitleForCreate {
        #[serde(rename = "Title", default)]
        title: String,
    }

    #[test]
    fn get_list_items_by_title_works() {
        let (user_name, password, site) = auth::tests::login_params();
        let security_token =
            get_security_token(site.clone(), user_name.to_string(), password.to_string());

        let access_token_cookies = get_access_token_cookies(site.clone(), security_token);
        let digest = get_the_request_digest(site.clone(), access_token_cookies.clone());
        let title = env::var("RUST_TITLE").unwrap().to_string();

        let items: Vec<GenericListItem> =
            get_list_items_by_title(title, access_token_cookies, digest, site);

        println!("items: '{:?}'", items);

        assert!(items.len() > 0);
    }

    pub fn since_the_epoch() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start.duration_since(UNIX_EPOCH).expect(
            "Time went backwards",
        );
        since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000
    }

    #[test]
    fn create_new_list_item_by_title_works() {
        let (user_name, password, site) = auth::tests::login_params();
        let security_token =
            get_security_token(site.clone(), user_name.to_string(), password.to_string());

        let new_item = GenericListItemWithTitleForCreate { title: format!("Test-{}", since_the_epoch()) };

        let access_token_cookies = get_access_token_cookies(site.clone(), security_token);
        let digest = get_the_request_digest(site.clone(), access_token_cookies.clone());
        let title = env::var("RUST_TITLE").unwrap().to_string();

        let item: GenericListItemWithTitle = add_list_item_by_list_title(
            title.to_owned(),
            access_token_cookies.clone(),
            digest.clone(),
            site.to_owned(),
            new_item,
            get_list_default_item_type(title.to_owned()),
        );

        println!("item: '{:?}'", item);
        let item2 = GenericListItemWithTitle{ title: format!("{}-Updated", item.title), .. item };
        let id = item2.id;

        update_list_item_by_list_title(
            title.to_owned(),
            access_token_cookies,
            digest,
            site,
            item2,
            get_list_default_item_type(title),
            id
        );

        //assert!(false);
    }
}
