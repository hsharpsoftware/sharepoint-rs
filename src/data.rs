extern crate serde;
extern crate tokio_core;
extern crate hyper_tls;

extern crate futures;

use auth::*;
use self::futures::future;

use hyper::{Method, Request};

#[allow(unused_imports)]
use super::*;

use self::serde::de::DeserializeOwned;
use self::serde::ser::Serialize;
use hyper::header::{ContentLength, ContentType, SetCookie, Accept, qitem, Cookie};
use hyper::mime;
use self::futures::{Future, Stream};

header! { (XRequestDigest, "X-RequestDigest") => [String] }
header! { (XHttpMethod, "X-Http-Method") => [String] }
header! { (IfMatch, "If-Match") => [String] }

#[derive(Debug, Deserialize, Default)]
pub struct HeaderItem {
    name: String,
    value: String,
}

pub fn process<'a, T>(
    url: String,
    body: String,
    access_token_cookies: Option<AccessTokenCookies>,
    parser: fn(String, Vec<HeaderItem>, Vec<String>) -> Option<T>,
    json: bool,
    x_request_digest: Option<RequestDigest>,
    method: Method,
    use_merge : bool,
) -> Option<T>
where
    T: serde::Deserialize<'a>,
{
    let mut core = self::tokio_core::reactor::Core::new().unwrap();

    let client = ::hyper::Client::configure()
        .connector(
            self::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap(),
        )
        .build(&core.handle());

    let uri = url.parse().unwrap();

    let mut req = Request::new(method, uri);
    req.set_body(body.to_owned());

    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));
    if access_token_cookies.is_some() {
        let atc = access_token_cookies.unwrap();
        let mut cookie = Cookie::new();
        let rt_fa = atc.rt_fa.unwrap();
        let fed_auth = atc.fed_auth.unwrap();
        cookie.append("rtFa", rt_fa.to_owned());
        cookie.append("FedAuth", fed_auth.to_owned());
        println!("rtFa:{}", rt_fa);
        println!("FedAuth:{}", fed_auth);
        req.headers_mut().set(cookie);
    };
    if json {
        req.headers_mut().set(
            Accept(vec![qitem(mime::APPLICATION_JSON)]),
        );
    }
    if x_request_digest.is_some() {
        let digest = x_request_digest.unwrap().content;

        req.headers_mut().set(XRequestDigest(digest.to_owned()));
        println!("digest:{}", digest);
    }

    if use_merge {
        req.headers_mut().set(XHttpMethod("PATCH".to_string()));
        req.headers_mut().set(IfMatch("*".to_string()));
    }

    let mut result: Option<T> = None;
    let mut headers: Vec<HeaderItem> = Vec::new();
    let mut header_cookies: Vec<String> = Vec::new();
    {
        let post = client.request(req).and_then(|res| {
            headers = res.headers()
                .iter()
                .map(|q| {
                    HeaderItem {
                        name: q.name().to_string(),
                        value: q.value_string(),
                    }
                })
                .collect();

            if let Some(&SetCookie(ref cookies)) = res.headers().get() {
                for cookie in cookies.iter() {
                    header_cookies.push(cookie.to_string());
                }
            }

            res.body()
                .fold(Vec::new(), |mut v, chunk| {
                    v.extend(&chunk[..]);
                    future::ok::<_, hyper::Error>(v)
                })
                .and_then(|chunks| {
                    let s = String::from_utf8(chunks).unwrap();
                    result = parser(s.to_owned(), headers, header_cookies);
                    future::ok::<_, hyper::Error>(s)
                })
        });

        core.run(post).unwrap();
    }

    result
}

fn parse_typed_json<T>(body: String, _: Vec<HeaderItem>, _: Vec<String>) -> Option<T>
where
    T: DeserializeOwned,
{
    println!("JSON Parsing '{:?}'", body.to_owned());
    let v: T = serde_json::from_str(&body).unwrap();
    Some(v)
}

fn do_not_parse<T>(_: String, _: Vec<HeaderItem>, _: Vec<String>) -> Option<T>
where
    T: DeserializeOwned + Default,
{
    Some(Default::default())
}

pub fn get_data<T>(
    url: String,
    access_token_cookies: AccessTokenCookies,
    digest: RequestDigest,
) -> Option<T>
where
    T: DeserializeOwned,
{
    process(
        url,
        "".to_string(),
        Some(access_token_cookies),
        parse_typed_json,
        true,
        Some(digest),
        Method::Get,
        false
    )
}

use self::serde_json::Value;

pub fn post_data<T, U>(
    url: String,
    access_token_cookies: AccessTokenCookies,
    digest: RequestDigest,
    data: U,
    use_merge: bool, 
) -> Option<T>
where
    T: DeserializeOwned + Default,
    U: Serialize,
{
    let body = serde_json::to_string(&data).unwrap();

    let v: Value = serde_json::from_str(&body).unwrap();

    println!("Will send '{}' to {}", v.to_string().to_owned(), url);

    process(
        url,
        v.to_string(),
        Some(access_token_cookies),
        if use_merge { do_not_parse } else { parse_typed_json },
        true,
        Some(digest),
        Method::Post,
        use_merge
    )
}
