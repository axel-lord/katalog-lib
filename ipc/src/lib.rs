//! Ipc library.

use ::std::net::Shutdown;

use ::futures::{FutureExt, select};
use ::smol::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::unix::UnixStream,
};

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

/// Write a message to a unix stream.
async fn write_message(writer: &mut UnixStream, message: &[u8]) -> Result<(), ::smol::io::Error> {
    let payload_size = message.len().to_le_bytes();
    writer.write_all(&payload_size).await?;
    writer.write_all(message).await?;
    writer.flush().await
}

impl Connection {
    /// Process incoming and outgoing messages.
    pub async fn process(self) -> ! {
        let Self {
            incoming,
            outgoing,
            stream,
        } = self;

        let mut writer = stream.clone();
        let mut reader = stream;
        let mut payload_size = [0u8; size_of::<u64>()];
        loop {
            let mut read = reader.read_exact(&mut payload_size).fuse();
            let result = loop {
                let mut forward = outgoing.recv_async();
                let to_send = select! {
                    result = read => break result,
                    to_send = forward => to_send,
                };
                let Ok(to_send) = to_send else {
                    if let Err(err) = writer.shutdown(Shutdown::Write) {
                        ::log::warn!("could not shutdown write of ipc channel, {err}");
                    }
                    continue;
                };
                if let Err(err) = write_message(&mut writer, &to_send).await {
                    ::log::error!("could not write message to stream, {err}");
                    if let Err(err) = writer.shutdown(Shutdown::Write) {
                        ::log::warn!("could not shutdown write of ipc channel, {err}");
                    }
                }
            };

            if let Err(err) = result {
                panic!("Failed to receive messages, {err}")
            }

            let payload_size = u64::from_le_bytes(payload_size);
        }
    }
}
