#![feature(async_await)]
#![allow(clippy::single_match)]

use {
    encoding::all::UTF_8,
    failure::Fallible,
    futures::{compat::*, prelude::*, task::*},
    std::{env, net::ToSocketAddrs},
    tokio::net::TcpStream,
    yaircc::{Code, IrcStream, Message, Prefix, StreamError, Writer},
};

async fn for_each_message(
    writer: Writer<Compat01As03<TcpStream>>,
    channel: String,
    msg: Result<Message, StreamError>,
) -> Fallible<()> {
    println!("{:?}", msg);
    match msg {
        Ok(msg) => {
            if msg.code == Code::RplWelcome {
                // join channel, no password
                writer.raw(format!("JOIN {}\n", channel)).await?;
            }
            // JOIN is sent when you join a channel.
            if msg.code == Code::Join {
                // If there is a prefix...
                if let Some(prefix) = msg.prefix {
                    match prefix {
                        // And the prefix is a user...
                        Prefix::User(user) => {
                            // And that user's nick is peekaboo, we've joined the channel!
                            if user.nickname == "peekaboo" {
                                writer
                                    .raw(format!("PRIVMSG {} :{}\n", channel, "peekaboo"))
                                    .await?;
                                // Note that if the reconnection settings said to reconnect,
                                // it would. Close would "really" stop it.
                                writer.raw(format!("QUIT :{}\n", "peekaboo")).await?;
                                // writer.close();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}

async fn async_main() -> Fallible<()> {
    let args: Vec<String> = env::args().collect();
    let server = args.get(1);
    let channel = args.get(2);

    if server.is_none() || channel.is_none() {
        eprintln!("Usage: {} <SERVER> <CHANNEL>", args[0]);
        std::process::exit(1)
    }
    let (server, channel) = (server.unwrap(), channel.unwrap());

    let mut addrs = server.to_socket_addrs()?;
    let stream_fut = Compat01As03::new(TcpStream::connect(&addrs.next().unwrap()));
    let stream = Compat01As03::new(stream_fut.await?);
    let irc_stream = IrcStream::new(stream, UTF_8);
    let writer = irc_stream.writer();

    writer
        .raw(format!("USER {} 8 * :{}\n", "peekaboo", "peekaboo"))
        .await?;
    writer.raw(format!("NICK {}\n", "peekaboo")).await?;

    irc_stream
        .then(|msg| for_each_message(writer.clone(), channel.clone(), msg))
        .try_collect::<()>()
        .err_into()
        .await
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut ex = rt.executor().compat();
    let fut = async_main().map_err(|err| eprintln!("{}", err)).map(|_| ());

    let handle_fut = ex.spawn_with_handle(fut).unwrap();
    futures::executor::block_on(handle_fut);
}
