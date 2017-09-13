extern crate futures;
extern crate tokio_core;

extern crate serde;
extern crate serde_json;
extern crate serde_xml_rs;

extern crate hyper_tls;

use super::*;

use self::futures::{Future, Stream};
use hyper::{Method, Request};
use hyper::header::{ContentLength, ContentType, SetCookie, Accept, qitem, Cookie};
use self::futures::{future};
use hyper::mime;
use self::serde::de::DeserializeOwned;

static GET_SECURITY_TOKEN_URL: &'static str = "https://login.microsoftonline.com/extSTS.srf";
static GET_ACCESS_TOKEN_URL: &'static str = "https://{host}.sharepoint.com/_forms/default.aspx?wa=wsignin1.0";
static GET_REQUEST_DIGEST_URL: &'static str = "https://{host}.sharepoint.com/_api/contextinfo";
pub static GET_LIST_URL: &'static str = "https://{host}.sharepoint.com/_api/web/lists/GetByTitle('{title}')";

static GET_SECURITY_TOKEN_BODY_PAR: &'static str = r##"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
      xmlns:a="http://www.w3.org/2005/08/addressing"
      xmlns:u="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd">
  <s:Header>
    <a:Action s:mustUnderstand="1">http://schemas.xmlsoap.org/ws/2005/02/trust/RST/Issue</a:Action>
    <a:ReplyTo>
      <a:Address>http://www.w3.org/2005/08/addressing/anonymous</a:Address>
    </a:ReplyTo>
    <a:To s:mustUnderstand="1">https://login.microsoftonline.com/extSTS.srf</a:To>
    <o:Security s:mustUnderstand="1"
       xmlns:o="http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd">
      <o:UsernameToken>
        <o:Username>{user_name}</o:Username>
        <o:Password>{password}</o:Password>
      </o:UsernameToken>
    </o:Security>
  </s:Header>
  <s:Body>
    <t:RequestSecurityToken xmlns:t="http://schemas.xmlsoap.org/ws/2005/02/trust">
      <wsp:AppliesTo xmlns:wsp="http://schemas.xmlsoap.org/ws/2004/09/policy">
        <a:EndpointReference>
          <a:Address>{host}.sharepoint.com</a:Address>
        </a:EndpointReference>
      </wsp:AppliesTo>
      <t:KeyType>http://schemas.xmlsoap.org/ws/2005/05/identity/NoProofKey</t:KeyType>
      <t:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</t:RequestType>
      <t:TokenType>urn:oasis:names:tc:SAML:1.0:assertion</t:TokenType>
    </t:RequestSecurityToken>
  </s:Body>
</s:Envelope>
        "##;

#[derive(Debug, Deserialize, Default)]
struct HeaderItem {
    name: String,
    value: String,
}

#[derive(Clone)]
pub struct AccessTokenCookies {
    rt_fa: Option<String>,
    fed_auth: Option<String>,
}

header! { (XRequestDigest, "X-RequestDigest") => [String] }

fn process<'a, T>(
    url: String,
    body: String,
    access_token_cookies: Option<AccessTokenCookies>,
    parser: fn(String, Vec<HeaderItem>, Vec<String>) -> Option<T>,
    json: bool,
    x_request_digest: Option<String>,
    method: Method,
) -> Option<T>
where
    T: serde::Deserialize<'a>,
{
    let mut core = auth::tokio_core::reactor::Core::new().unwrap();

    let client = ::hyper::Client::configure()
        .connector(
            auth::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap(),
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
        cookie.append("rtFa", rt_fa.to_owned());
        cookie.append("FedAuth", atc.fed_auth.unwrap());
        //println!("rtFa:{}", rt_fa);
        req.headers_mut().set(cookie);
    };
    if json {
        req.headers_mut().set(
            Accept(vec![qitem(mime::APPLICATION_JSON)]),
        );
    }
    if x_request_digest.is_some() {
        req.headers_mut().set(XRequestDigest(
            x_request_digest.unwrap().to_owned(),
        ));
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

use self::serde_xml_rs::deserialize;

#[derive(Debug, Deserialize, Default)]
struct Header {}

#[derive(Debug, Deserialize, Default)]
struct BinarySecurityToken {
    #[serde(rename = "$value")]
    content: String,
}

#[derive(Debug, Deserialize, Default)]
struct RequestedSecurityToken {
    #[serde(rename = "BinarySecurityToken", default)]
    binary_security_token: BinarySecurityToken,
}

#[derive(Debug, Deserialize, Default)]
struct RequestSecurityTokenResponse {
    #[serde(rename = "RequestedSecurityToken", default)]
    requested_security_token: RequestedSecurityToken,
}

#[derive(Debug, Deserialize, Default)]
struct Body {
    #[serde(rename = "RequestSecurityTokenResponse", default)]
    request_security_token_response: RequestSecurityTokenResponse,
}

#[derive(Debug, Deserialize)]
struct Envelope {
    #[serde(rename = "Header", default)]
    pub header: Header,
    #[serde(rename = "Body", default)]
    pub body: Body,
}

#[derive(Debug, Deserialize, Default)]
struct FormDigestValue {
    #[serde(rename = "$value")]
    content: String,
}

#[derive(Debug, Deserialize, Default)]
struct GetContextWebInformation {
    #[serde(rename = "FormDigestValue", default)]
    pub form_digest_value: FormDigestValue,
}

fn parse_typed_json<T>(body: String, _: Vec<HeaderItem>, _: Vec<String>) -> Option<T>
where
    T: DeserializeOwned,
{
    //println!("JSON Parsing '{:?}'", body.to_owned());
    let v: T = serde_json::from_str(&body).unwrap();
    Some(v)
}

fn parse_xml_envelope(body: String, _: Vec<HeaderItem>, _: Vec<String>) -> Option<Envelope> {
    //println!("XML Parsing '{:?}'", body);
    let v: Envelope = deserialize(body.as_bytes()).unwrap();
    Some(v)
}

pub fn get_security_token(host: String, user_name: String, password: String) -> String {
    let s = GET_SECURITY_TOKEN_BODY_PAR
        .replace("{user_name}", &user_name)
        .replace("{password}", &password)
        .replace("{host}", &host);
    let res: Envelope = process(
        GET_SECURITY_TOKEN_URL.to_string(),
        s.to_string(),
        None,
        parse_xml_envelope,
        false,
        None,
        Method::Post,
    ).unwrap();
    res.body
        .request_security_token_response
        .requested_security_token
        .binary_security_token
        .content
}

fn parse_cookies(_: String, _: Vec<HeaderItem>, cookies: Vec<String>) -> Option<(Vec<String>)> {
    let res: Vec<String> = cookies
        .iter()
        .map(|x| x.to_owned().split(";").next().unwrap().to_string())
        .filter(|x| x.starts_with("rtFa=") || x.starts_with("FedAuth="))
        .collect();
    Some(res)
}

pub fn get_access_token_cookies(host: String, security_token: String) -> AccessTokenCookies {
    let data = process(
        GET_ACCESS_TOKEN_URL.replace("{host}", &host),
        security_token,
        None,
        parse_cookies,
        false,
        None,
        Method::Post,
    ).unwrap();
    let mut res = AccessTokenCookies {
        rt_fa: None,
        fed_auth: None,
    };
    for i in data.clone() {
        //println!("Cookie:{}", i);
        if i.starts_with("rtFa=") {
            let right = i.split("rtFa=").nth(1).unwrap().to_string();
            res = AccessTokenCookies {
                rt_fa: Some(right),
                fed_auth: res.fed_auth,
            };
        }
        if i.starts_with("FedAuth=") {
            let right = i.split("FedAuth=").nth(1).unwrap().to_string();
            res = AccessTokenCookies {
                rt_fa: res.rt_fa,
                fed_auth: Some(right),
            };
        }
    }
    res
}

fn parse_digest(
    body: String,
    _: Vec<HeaderItem>,
    _: Vec<String>,
) -> Option<GetContextWebInformation> {
    //println!("Parsing '{:?}'", body);
    let v: GetContextWebInformation = deserialize(body.as_bytes()).unwrap();
    Some(v)
}

pub fn get_the_request_digest(host: String, access_token_cookies: AccessTokenCookies) -> String {
    let res: GetContextWebInformation = process(
        GET_REQUEST_DIGEST_URL.replace("{host}", &host),
        "".to_string(),
        Some(access_token_cookies),
        parse_digest,
        false,
        None,
        Method::Post,
    ).unwrap();
    res.form_digest_value.content
}

pub fn get_data<T>(
    url: String,
    access_token_cookies: AccessTokenCookies,
    digest: String,
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
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::env;

    pub fn login_params() -> (String, String, String) {
        let login = env::var("RUST_USERNAME").unwrap();
        let password = env::var("RUST_PASSWORD").unwrap();
        let host = env::var("RUST_HOST").unwrap();
        (login, password, host)
    }

    use self::serde_json::Value;

    fn parse_json(body: String, _: Vec<HeaderItem>, _: Vec<String>) -> Option<Value> {
        println!("JSON Parsing '{:?}'", body);
        let v: Value = serde_json::from_str(&body).unwrap();
        Some(v)
    }

    #[test]
    fn json_works() {
        let _res = process(
            "https://httpbin.org/post".to_string(),
            "".to_string(),
            None,
            parse_json,
            true,
            None,
            Method::Post,
        );
        //println!("Got '{:?}'", _res);
    }
    #[test]
    fn xml_works() {
        let (user_name, password, host) = login_params();
        let _res = get_security_token(
            host.to_string(),
            user_name.to_string(),
            password.to_string(),
        );
        //println!("Got '{:?}'", _res);
    }
    #[test]
    fn get_access_token_cookies_works() {
        let (user_name, password, host) = login_params();
        let security_token = get_security_token(
            host.to_string(),
            user_name.to_string(),
            password.to_string(),
        );
        let access_token = get_access_token_cookies(host.to_string(), security_token);
        assert!(access_token.rt_fa.is_some());
        assert!(access_token.fed_auth.is_some());
    }
    #[test]
    fn get_the_request_digest_works() {
        let (user_name, password, host) = login_params();
        let security_token = get_security_token(
            host.to_string(),
            user_name.to_string(),
            password.to_string(),
        );
        let digest = get_the_request_digest(
            host.to_string(),
            get_access_token_cookies(host.to_string(), security_token),
        );
        //println!("Digest '{:?}'", digest);
        assert!(digest.len() > 0);
    }
    #[test]
    fn get_the_list() {
        let (user_name, password, host) = login_params();
        let security_token = get_security_token(
            host.to_string(),
            user_name.to_string(),
            password.to_string(),
        );

        let access_token_cookies = get_access_token_cookies(host.to_string(), security_token);
        let digest = get_the_request_digest(host.to_string(), access_token_cookies.clone());

        println!(
            "Trying to get to '{}'",
            env::var("RUST_LIST_GET_URL").unwrap().to_string()
        );

        process(
            env::var("RUST_LIST_GET_URL").unwrap().to_string(),
            "".to_string(),
            Some(access_token_cookies),
            parse_json,
            true,
            Some(digest),
            Method::Get,
        );
    }
}
