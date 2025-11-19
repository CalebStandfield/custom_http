use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use std::io;
use std::io::Write;
use std::time::Duration;

struct Connection {
    stream: TcpStream,
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
    state: State,
    keep_alive: bool,
}

enum State {
    ReadingHeader,
    ReadingBody,
    WritingHeader,
    WritingBody,
    ReadyToRespond,
    Closed,
}

const LISTENER: Token = Token(0);

struct Reactor {
    poll: Poll,
    events: Events,
    listener: TcpListener,
    conns: slab::Slab<Connection>, // index → Connection
                                   // worker_rx: Receiver<(Token, Vec<u8>)>, // from thread pool back to loop (later)
}

impl Reactor {
    fn new(addr: &str) -> std::io::Result<Self> {
        let poll = Poll::new()?;
        let mut listener = TcpListener::bind(addr.parse().unwrap())?;
        // listener is nonblocking by default in mio
        poll.registry()
            .register(&mut listener, LISTENER, Interest::READABLE)?;
        Ok(Self {
            poll,
            events: Events::with_capacity(1024),
            listener,
            conns: slab::Slab::with_capacity(1024),
        })
    }
}

fn handle_writable(conn: &mut Connection) -> io::Result<()> {
    while !conn.write_buffer.is_empty() {
        let buf = &conn.write_buffer[..];

        match conn.stream.write(buf) {
            Ok(0) => {
                conn.state = State::Closed;
                break;
            }
            Ok(n) => {
                conn.write_buffer.drain(..n);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                eprintln!("write error: {}", e);
                conn.state = State::Closed;
                break;
            }
        }
    }
    Ok(())

}
