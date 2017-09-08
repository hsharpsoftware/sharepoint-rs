extern crate futures;
extern crate hyper;
extern crate tokio_core;

extern crate serde;
extern crate serde_json;
extern crate serde_xml_rs;

#[macro_use]
extern crate serde_derive;

extern crate hyper_tls;

use std::io::{self, Write};
use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::{Client, Chunk, Method, Request, Headers};
use hyper::header::{ContentLength, ContentType, SetCookie};
use self::futures::{future, Async, Poll};
use self::futures::task::{self, Task};


static GET_SECURITY_TOKEN_URL: &'static str = "https://login.microsoftonline.com/extSTS.srf";
static GET_ACCESS_TOKEN_URL: &'static str = "https://{host}.sharepoint.com/_forms/default.aspx?wa=wsignin1.0";

static GET_SECURITY_TOKEN_BODY_PAR: &'static str =
r##"<s:Envelope xmlns:s="http://www.w3.org/2003/05/soap-envelope"
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
    name : String,
    value : String
}

fn post<'a, T>(url: String, body: String, parser: fn(String, Vec<HeaderItem>, Vec<String>) -> Option<T> )  -> Option<T>
    where T: serde::Deserialize<'a>
{
    let mut core = ::tokio_core::reactor::Core::new().unwrap();

    let client = ::hyper::Client::configure()
        .connector(::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap())
        .build(&core.handle());

    let uri = url.parse().unwrap();

    let mut req = Request::new(Method::Post, uri);
    req.set_body(body.to_owned());
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(body.len() as u64));

    let mut result : Option<T> = None;
    let mut headers : Vec<HeaderItem> = Vec::new();
    let mut header_cookies : Vec<String> = Vec::new();
    {
        let post = client.request(req).and_then(|res| {
            headers = res.headers().iter().map( |q| HeaderItem{name:q.name().to_string(), value:q.value_string()} ).collect();

            if let Some (&SetCookie (ref cookies)) = res.headers().get() {
                for cookie in cookies.iter() {
                    header_cookies.push(cookie.to_string());
                }
            }

            res.body().fold(Vec::new(), |mut v, chunk| {
                v.extend(&chunk[..]);
                future::ok::<_, hyper::Error>(v)
            }).and_then(|chunks| {
                let s = String::from_utf8(chunks).unwrap();                
                result = parser(s.to_owned(), headers, header_cookies );
                future::ok::<_, hyper::Error>(s)
            })
        });    

        core.run(post).unwrap();
    }

    result
}

use serde_xml_rs::deserialize;

#[derive(Debug, Deserialize, Default)]
struct Header {
}

#[derive(Debug, Deserialize, Default)]
struct BinarySecurityToken {
    #[serde(rename="$value")] //see https://stackoverflow.com/a/37972585/2013924
    content : String
}

#[derive(Debug, Deserialize, Default)]
struct RequestedSecurityToken {
    #[serde(rename = "BinarySecurityToken", default)]
    binary_security_token : BinarySecurityToken
}

#[derive(Debug, Deserialize, Default)]
struct RequestSecurityTokenResponse {
    #[serde(rename = "RequestedSecurityToken", default)]
    requested_security_token : RequestedSecurityToken
}

#[derive(Debug, Deserialize, Default)]
struct Body {
    #[serde(rename = "RequestSecurityTokenResponse", default)]
    request_security_token_response : RequestSecurityTokenResponse
}

#[derive(Debug, Deserialize)]
struct Envelope {
    #[serde(rename = "Header", default)]
    pub header: Header,
    #[serde(rename = "Body", default)]
    pub body: Body,
}

use serde_json::Value;

fn parse_json( body : String, _ : Vec<HeaderItem>, _ : Vec<String> ) -> Option<Value> {
    println!("Parsing '{:?}'", body);        
    let v: Value = serde_json::from_str(&body).unwrap();
    Some(v)
}

fn parse_xml( body : String, _ : Vec<HeaderItem>, _ : Vec<String> ) -> Option<Envelope> {
    println!("Parsing '{:?}'", body);
    let v: Envelope = deserialize(body.as_bytes()).unwrap();
    Some(v)
}

pub fn get_security_token( host : String, user_name : String, password : String ) -> String {
    let s = GET_SECURITY_TOKEN_BODY_PAR.replace("{user_name}", &user_name).replace("{password}", &password).replace("{host}", &host);
    let res : Envelope = post(GET_SECURITY_TOKEN_URL.to_string(), s.to_string(), parse_xml).unwrap();
    res.body.request_security_token_response.requested_security_token.binary_security_token.content
}

fn parse_cookies( _ : String, _ : Vec<HeaderItem>, cookies : Vec<String> ) -> Option<(Vec<String>)> {
    let res : Vec<String> = cookies.iter().map(|x| x.to_owned() ).filter(|x| x.starts_with("rtFa=") || x.starts_with("FedAuth=")).collect();
    Some(res)
}

pub fn get_access_token_cookies( host : String, security_token : String ) -> Vec<String> {
    let res = post(GET_ACCESS_TOKEN_URL.replace("{host}", &host), security_token, parse_cookies).unwrap();
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_works() {
        let res = post("https://httpbin.org/post".to_string(), "".to_string(), parse_json);
        println!("Got '{:?}'", res);
    }
    #[test]
    fn xml_works() {
        let user_name = "";
        let password = "";
        let host = "";
        let res = get_security_token(host.to_string(), user_name.to_string(),password.to_string() );
        println!("Got '{:?}'", res);
    }
    #[test]
    fn get_access_token_cookies_works() {
        let user_name = "";
        let password = "";
        let host = "";
        let security_token = get_security_token(host.to_string(), user_name.to_string(),password.to_string() );
        assert_eq!(get_access_token_cookies( host.to_string(), security_token ).len(), 2);
    }
}
