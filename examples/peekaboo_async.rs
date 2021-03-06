use {
    async_std::{net::TcpStream, task},
    encoding::all::UTF_8,
    failure::Fallible,
    futures::prelude::*,
    std::env,
    yaircc::{Code, IrcStream, Message, Prefix, StreamError, Writer},
};

macro_rules! write_irc {
    ($writer:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $writer.raw(msg).await?;
    }
}

async fn for_each_message(
    writer: &Writer<TcpStream>,
    channel: &str,
    msg: Result<Message, StreamError>,
) -> Fallible<()> {
    match msg {
        Ok(msg) => {
            println!("{:?}", msg);
            match msg.code {
                Code::RplWelcome => {
                    // join channel, no password
                    write_irc!(writer, "JOIN {}\n", channel);
                }
                // JOIN is sent when you join a channel.
                Code::Join => {
                    // If there is a prefix and the prefix is a user...
                    if let Some(Prefix::User(user)) = msg.prefix {
                        // And that user's nick is peekaboo, we've joined the channel!
                        if user.nickname == "peekaboo" {
                            write_irc!(writer, "PRIVMSG {} :{}\n", channel, "peekaboo");
                            // Note that if the reconnection settings said to reconnect,
                            // it would. Close would "really" stop it.
                            write_irc!(writer, "QUIT :{}\n", "peekaboo");
                        }
                    }
                }
                Code::Ping => {
                    write_irc!(writer, "PONG {}\n", msg.args.join(" "));
                }
                _ => {}
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    Ok(())
}

fn get_args() -> (String, String) {
    let args: Vec<String> = env::args().collect();
    let server = args.get(1);
    let channel = args.get(2);

    if server.is_none() || channel.is_none() {
        eprintln!("Usage: {} <SERVER> <CHANNEL>", args[0]);
        std::process::exit(1)
    }

    (server.unwrap().clone(), channel.unwrap().clone())
}

async fn async_main() -> Fallible<()> {
    let (server, channel) = get_args();

    let stream = TcpStream::connect(server).await?;
    let irc_stream = IrcStream::new(stream, UTF_8);
    let writer = irc_stream.writer();

    write_irc!(writer, "USER {} 8 * :{}\n", "peekaboo", "peekaboo");
    write_irc!(writer, "NICK {}\n", "peekaboo");

    irc_stream
        .then(|msg| for_each_message(&writer, &channel, msg))
        .try_collect::<()>()
        .err_into()
        .await
}

fn main() -> Fallible<()> {
    let fut = async_main();
    task::block_on(fut)?;
    Ok(())
}
