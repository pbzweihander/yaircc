#![allow(clippy::single_match)]

use {
    encoding::all::UTF_8,
    failure::Fallible,
    std::{env, net::TcpStream},
    yaircc::{Code, IrcStream, Prefix},
};

fn main() -> Fallible<()> {
    let args: Vec<String> = env::args().collect();
    let server = args.get(1);
    let channel = args.get(2);

    if server.is_none() || channel.is_none() {
        eprintln!("Usage: {} <SERVER> <CHANNEL>", args[0]);
        std::process::exit(1)
    }
    let (server, channel) = (server.unwrap(), channel.unwrap());

    let stream = TcpStream::connect(server)?;
    let irc_stream = IrcStream::from_std(stream, UTF_8);
    let writer = irc_stream.writer();

    writer.raw_wait(format!("USER {} 8 * :{}\n", "peekaboo", "peekaboo"))?;
    writer.raw_wait(format!("NICK {}\n", "peekaboo"))?;

    for msg in irc_stream {
        println!("{:?}", msg);
        match msg {
            Ok(msg) => {
                if msg.code == Code::RplWelcome {
                    // join channel, no password
                    writer.raw_wait(format!("JOIN {}\n", channel))?;
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
                                    writer.raw_wait(format!(
                                        "PRIVMSG {} :{}\n",
                                        channel, "peekaboo"
                                    ))?;
                                    // Note that if the reconnection settings said to reconnect,
                                    // it would. Close would "really" stop it.
                                    writer.raw_wait(format!("QUIT :{}\n", "peekaboo"))?;
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
    }

    Ok(())
}
