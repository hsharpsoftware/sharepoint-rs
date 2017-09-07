extern crate futures;
extern crate hyper;
extern crate tokio_core;

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::io::{self, Write};
use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::{Client, Chunk, Method, Request};
use hyper::header::{ContentLength, ContentType};

fn post<T>(body: String, parser: fn(Chunk) -> Option<T> )  -> Option<T>
    where T: serde::Serialize
{
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let uri = "http://httpbin.org/post".parse().unwrap();

    let mut req = Request::new(Method::Post, uri);
    req.set_body(body);

    let mut result : Option<T> = None;
    {
        let post = client.request(req).and_then(|res| {
            res.body().concat2().and_then(|body| {
                //let v: T = serde_json::from_slice(&body).unwrap();
                result = parser(body);
                Ok(())
            })
        });    

        core.run(post).unwrap();
    }

    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
