#![feature(async_await)]

mod code;
mod message;
mod stream;

pub use {
    code::Code,
    message::{Message, ParseError, Prefix, PrefixUser},
    stream::{IrcStream, StreamError, Writer},
};
