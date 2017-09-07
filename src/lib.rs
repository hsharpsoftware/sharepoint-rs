extern crate futures;
extern crate hyper;
extern crate tokio_core;


use std::io::{self, Write};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
