extern crate serde_xml_rs;

#[allow(unused_imports)]
use super::*;

use data::*;

use hyper::Method;

static GET_SECURITY_TOKEN_URL: &'static str = "https://login.microsoftonline.com/extSTS.srf";
static GET_ACCESS_TOKEN_URL: &'static str = "https://{host}/_forms/default.aspx?wa=wsignin1.0";
static GET_REQUEST_DIGEST_URL: &'static str = "https://{host}/{path}/_api/contextinfo";

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
          <a:Address>{host}</a:Address>
        </a:EndpointReference>
      </wsp:AppliesTo>
      <t:KeyType>http://schemas.xmlsoap.org/ws/2005/05/identity/NoProofKey</t:KeyType>
      <t:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</t:RequestType>
      <t:TokenType>urn:oasis:names:tc:SAML:1.0:assertion</t:TokenType>
    </t:RequestSecurityToken>
  </s:Body>
</s:Envelope>
        "##;


#[derive(Clone)]
pub struct AccessTokenCookies {
    pub rt_fa: Option<String>,
    pub fed_auth: Option<String>,
}

#[derive(Clone)]
pub struct RequestDigest {
    pub content: String,
}

#[derive(Clone)]
pub struct LoginContext {
    pub access_token : AccessTokenCookies,
    pub request_digest : RequestDigest,
    pub site : Site,
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

fn parse_xml_envelope(body: String, _: Vec<HeaderItem>, _: Vec<String>) -> Option<Envelope> {
    //println!("XML Parsing '{:?}'", body);
    let v: Envelope = deserialize(body.as_bytes()).unwrap();
    Some(v)
}

pub fn login( site : String, user_name : String, password : String ) -> LoginContext {
    let site_url: &str = &site;
    let site_parsed: hyper::Uri = site_url.parse().unwrap();
    let site = Site { parent: site_parsed.to_string() };
    let security_token =
        get_security_token(site.clone(), user_name, password);
    let access_token_cookies = get_access_token_cookies(site.clone(), security_token);
    let digest = get_the_request_digest(site.clone(), access_token_cookies.clone());    
    LoginContext{  
        access_token : access_token_cookies,
        request_digest : digest,
        site : site,
    }
}

fn host(site: Site) -> String {
    println!("Parsing '{:?}'", site.parent);
    let site_parsed: hyper::Uri = site.parent.parse().unwrap();
    let result = site_parsed.host().unwrap().to_string();
    println!("Returning '{:?}'", result);
    result
}

fn host_and_path(site: Site) -> (String, String) {
    println!("Parsing '{:?}'", site.parent);
    let site_parsed: hyper::Uri = site.parent.parse().unwrap();
    let result = site_parsed.host().unwrap().to_string();
    let path = site_parsed.path().to_string();
    println!("Returning '{:?}' & '{:?}'", result, path);
    (result, path)
}

pub fn get_security_token(site: Site, user_name: String, password: String) -> String {
    let host = host(site);
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
        false
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

pub fn get_access_token_cookies(site: Site, security_token: String) -> AccessTokenCookies {
    let host = host(site);
    let data = process(
        GET_ACCESS_TOKEN_URL.replace("{host}", &host),
        security_token,
        None,
        parse_cookies,
        false,
        None,
        Method::Post,
        false
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

pub fn get_the_request_digest(
    site: Site,
    access_token_cookies: AccessTokenCookies,
) -> RequestDigest {
    let (host, path) = host_and_path(site);
    let res: GetContextWebInformation = process(
        GET_REQUEST_DIGEST_URL.replace("{host}", &host).replace(
            "{path}",
            &path,
        ),
        "".to_string(),
        Some(access_token_cookies),
        parse_digest,
        false,
        None,
        Method::Post,
        false,
    ).unwrap();
    RequestDigest { content: res.form_digest_value.content }
}


#[cfg(test)]
pub mod tests {
    use super::*;
    use std::env;

    pub fn login_params() -> (String, String, Site) {
        let login = env::var("RUST_USERNAME").unwrap();
        let password = env::var("RUST_PASSWORD").unwrap();
        let config_site = env::var("RUST_SITE").unwrap();
        let site_url: &str = &config_site;
        let site_parsed: hyper::Uri = site_url.parse().unwrap();
        let site = Site { parent: site_parsed.to_string() };
        (login, password, site)
    }

    use self::serde_json::Value;

    fn parse_json(body: String, _: Vec<HeaderItem>, _: Vec<String>) -> Option<Value> {
        println!("JSON Parsing '{:?}'", body);
        let v: Value = serde_json::from_str(&body).unwrap();
        Some(v)
    }

    #[test]
    fn xml_works() {
        let (user_name, password, site) = login_params();
        let _res = get_security_token(site, user_name.to_string(), password.to_string());
        //println!("Got '{:?}'", _res);
    }
    #[test]
    fn get_access_token_cookies_works() {
        let (user_name, password, site) = login_params();
        let security_token =
            get_security_token(site.clone(), user_name.to_string(), password.to_string());
        let access_token = get_access_token_cookies(site, security_token);
        assert!(access_token.rt_fa.is_some());
        assert!(access_token.fed_auth.is_some());
    }
    #[test]
    fn get_the_request_digest_works() {
        let (user_name, password, site) = login_params();
        let security_token =
            get_security_token(site.clone(), user_name.to_string(), password.to_string());
        let digest =
            get_the_request_digest(site.clone(), get_access_token_cookies(site, security_token));
        //println!("Digest '{:?}'", digest);
        assert!(digest.content.len() > 0);
    }
    #[test]
    fn get_the_list() {
        let (user_name, password, site) = login_params();
        let security_token =
            get_security_token(site.clone(), user_name.to_string(), password.to_string());

        let access_token_cookies = get_access_token_cookies(site.clone(), security_token);
        let digest = get_the_request_digest(site, access_token_cookies.clone());

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
            false,
        );
    }
}
