use {
    encoding::all::UTF_8,
    failure::Fallible,
    futures::io::AllowStdIo,
    std::{env, net::TcpStream},
    yaircc::{Code, IrcStream, Message, Prefix, StreamError, Writer},
};

macro_rules! write_irc {
    ($writer:expr, $($arg:tt)*) => {
        let msg = format!($($arg)*);
        $writer.raw_wait(msg)?;
    }
}

fn for_each_message(
    writer: &Writer<AllowStdIo<TcpStream>>,
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

fn main() -> Fallible<()> {
    let (server, channel) = get_args();

    let stream = TcpStream::connect(server)?;
    let irc_stream = IrcStream::from_std(stream, UTF_8);
    let writer = irc_stream.writer();

    write_irc!(writer, "USER {} 8 * :{}\n", "peekaboo", "peekaboo");
    write_irc!(writer, "NICK {}\n", "peekaboo");

    for msg in irc_stream {
        for_each_message(&writer, &channel, msg)?;
    }

    Ok(())
}
