//! Ipc library.

use ::core::pin::pin;
use ::std::net::Shutdown;

use ::futures::{FutureExt, select};
use ::smol::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::unix::UnixStream,
};

/// Sender used to send ipc messages to a channel.
#[derive(Debug, Clone)]
pub struct Sender {
    /// Wrapped sender.
    tx: ::flume::Sender<Box<[u8]>>,
}

impl Sender {
    /// Send a message.
    ///
    /// # Errors
    /// If th message cannot be sent it is returned
    /// as the error variant.
    pub fn send(&self, msg: Box<[u8]>) -> Result<(), Box<[u8]>> {
        self.tx.send(msg).map_err(::flume::SendError::into_inner)
    }
}

/// Receiver used to receive ipc messages from a channel.
#[derive(Debug, Clone)]
pub struct Receiver {
    /// Wrapped receiver.
    rx: ::flume::Receiver<Box<[u8]>>,
}

impl Receiver {
    /// Receive a message.
    ///
    /// Returns None if channel is closed.
    pub fn recv(&self) -> Option<Box<[u8]>> {
        self.rx.recv().ok()
    }
}

/// Ipc channel connection.
#[derive(Debug)]
pub struct Connection {
    /// Where to send incoming messages.
    incoming: ::flume::Sender<Box<[u8]>>,
    /// Where to receive outgoing messages from.
    outgoing: ::flume::Receiver<Box<[u8]>>,
    /// Unix stream in use.
    stream: UnixStream,
}

/// Collection of reader and writer pair.
#[derive(Debug, Clone)]
struct Stream {
    /// Reader socket.
    reader: UnixStream,
    /// Writer socket.
    writer: UnixStream,
}
/// Write a message to a unix stream.
async fn write_message(writer: &mut UnixStream, message: &[u8]) -> Result<(), ::smol::io::Error> {
    let payload_size = message.len().to_le_bytes();
    writer.write_all(&payload_size).await?;
    writer.write_all(message).await?;
    writer.flush().await
}

/// Read the size of a payload.
async fn read_payload_size(reader: &mut UnixStream) -> Result<u64, ::smol::io::Error> {
    let mut payload_size = [0u8; size_of::<u64>()];
    reader.read_exact(&mut payload_size).await?;
    Ok(u64::from_le_bytes(payload_size))
}

impl Stream {
    /// Run event loop, returning parsed read message length.
    async fn event_loop(
        &mut self,
        state: &mut State,
        outgoing: &::flume::Receiver<Box<[u8]>>,
    ) -> Result<u64, ::smol::io::Error> {
        let Self { reader, writer } = self;
        let mut read = pin!(read_payload_size(reader).fuse());
        loop {
            let mut forward = outgoing.recv_async();
            let payload = select! {
                result = read => return result,
                payload = forward => payload,
            };

            let Ok(payload) = payload else {
                ::log::info!(
                    "no senders for outgoing ipc requests remain, closing write end of channel"
                );
                break;
            };

            if let Err(err) = write_message(writer, &payload).await {
                ::log::info!("write to ipc channel failed, {err}");
                break;
            }
        }
        if let Err(err) = self.writer.shutdown(Shutdown::Write) {
            ::log::warn!("failed to shut down write of ipc channel, {err}")
        }
        state.stop_writing();
        read.await
    }

    /// Read payload given size.
    async fn read_payload(
        &mut self,
        payload_size: u64,
        state: &mut State,
        incoming: &::flume::Sender<Box<[u8]>>,
    ) {
        let additional = match usize::try_from(payload_size) {
            Err(err) => {
                ::log::error!("could not convert u64 to usize, {err}");
                state.stop_reading();
                return;
            }
            Ok(additional) => additional,
        };

        let mut buf = vec![0; additional];
        if let Err(err) = self.reader.read_exact(&mut buf).await {
            ::log::info!("failed to receive message content, {err}");
            state.stop_reading();
            return;
        }

        if let Err(err) = incoming.send_async(buf.into_boxed_slice()).await {
            ::log::error!("could not forward received message, {err}");
            state.stop_reading();
        }
    }
}

impl From<UnixStream> for Stream {
    fn from(value: UnixStream) -> Self {
        Self {
            reader: value.clone(),
            writer: value,
        }
    }
}

/// Process loop state.
#[derive(Debug)]
enum State {
    /// Reading and writing.
    ReadWrite,
    /// Only reading.
    Read,
    /// Neither reading nor writing.
    Closed,
}

impl State {
    /// Remove reading from state.
    const fn stop_reading(&mut self) {
        *self = Self::Closed;
    }
    /// Remove reading from state.
    const fn stop_writing(&mut self) {
        if let Self::ReadWrite = self {
            *self = Self::Read
        }
    }
}

impl Connection {
    /// Create an unbounded connection.
    pub fn unbounded(stream: UnixStream) -> (Self, Sender, Receiver) {
        Self::builder().bind(stream)
    }

    /// Create a connection builder.
    pub const fn builder() -> ConnectionBuilder {
        ConnectionBuilder {
            in_cap: None,
            out_cap: None,
        }
    }

    /// Process incoming and outgoing messages.
    pub async fn process(self) {
        let Self {
            incoming,
            outgoing,
            stream,
        } = self;

        let mut stream = Stream::from(stream);
        let mut state = State::ReadWrite;

        loop {
            let payload_size = match state {
                State::Closed => break,
                State::ReadWrite => match stream.event_loop(&mut state, &outgoing).await {
                    Ok(payload_size) => payload_size,
                    Err(err) => {
                        ::log::error!("Failed to receive messages, {err}");
                        state.stop_reading();
                        continue;
                    }
                },
                State::Read => match read_payload_size(&mut stream.reader).await {
                    Ok(payload_size) => payload_size,
                    Err(err) => {
                        ::log::error!("Failed to receive payload size, {err}");
                        state.stop_reading();
                        continue;
                    }
                },
            };

            if payload_size == u64::MAX {
                ::log::info!("stream kill signal received...");
                state.stop_reading();
                continue;
            }

            stream
                .read_payload(payload_size, &mut state, &incoming)
                .await;
        }
    }
}

/// Builder used to configure connection before creation.
#[derive(Debug, Default)]
pub struct ConnectionBuilder {
    /// Bound of incoming channel (messages received by connection).
    in_cap: Option<usize>,
    /// Bound of outgoing channel (messages sent by channel).
    out_cap: Option<usize>,
}

impl ConnectionBuilder {
    /// Set capacity of intra process channel used to send outgoing messages.
    pub const fn outgoing_capacity(self, cap: usize) -> Self {
        Self {
            out_cap: Some(cap),
            ..self
        }
    }

    /// Set capacity of intra process channel used to receive incoming messages.
    pub const fn incoming_capacity(self, cap: usize) -> Self {
        Self {
            in_cap: Some(cap),
            ..self
        }
    }

    /// Build connection.
    pub fn bind(self, stream: UnixStream) -> (Connection, Sender, Receiver) {
        let Self { in_cap, out_cap } = self;

        let (incoming, rx) = if let Some(cap) = in_cap {
            ::flume::bounded(cap)
        } else {
            ::flume::unbounded()
        };
        let (tx, outgoing) = if let Some(cap) = out_cap {
            ::flume::bounded(cap)
        } else {
            ::flume::unbounded()
        };

        (
            Connection {
                incoming,
                outgoing,
                stream,
            },
            Sender { tx },
            Receiver { rx },
        )
    }
}
