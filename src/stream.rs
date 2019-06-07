use {
    crate::message::{Message, ParseError},
    encoding::{DecoderTrap, EncoderTrap, EncodingRef},
    futures::{
        executor::{block_on, block_on_stream, BlockingStream},
        io::{AllowStdIo, BufReader, Error as AsyncIoError, ReadHalf, WriteHalf},
        lock::Mutex,
        prelude::*,
        ready,
        task::{Context, Poll},
    },
    std::{
        fmt,
        io::{Error as IoError, Read, Write},
        mem,
        pin::Pin,
        sync::Arc,
    },
};

#[derive(Debug)]
pub enum StreamError {
    ParseError(ParseError),
    AsyncIoError(AsyncIoError),
}

impl From<ParseError> for StreamError {
    fn from(err: ParseError) -> Self {
        StreamError::ParseError(err)
    }
}

impl From<AsyncIoError> for StreamError {
    fn from(err: AsyncIoError) -> Self {
        StreamError::AsyncIoError(err)
    }
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            StreamError::ParseError(ref e) => write!(f, "ParseError: {}", e),
            StreamError::AsyncIoError(ref e) => write!(f, "AsyncIoError: {}", e),
        }
    }
}

impl std::error::Error for StreamError {}

pub struct Writer<S> {
    pub encoding: EncodingRef,
    inner: Arc<Mutex<WriteHalf<S>>>,
}

impl<S> Writer<S>
where
    S: AsyncWrite + Unpin,
{
    #[allow(clippy::needless_lifetimes)]
    pub async fn raw(&self, msg: impl AsRef<str>) -> Result<(), IoError> {
        let bytes = self
            .encoding
            .encode(msg.as_ref(), EncoderTrap::Ignore)
            .unwrap();

        let mut writer = self.inner.lock().await;
        writer.write_all(&bytes).await
    }

    pub fn raw_wait(&self, msg: impl AsRef<str>) -> Result<(), IoError> {
        let fut = self.raw(msg);
        block_on(fut)
    }
}

impl<S> Clone for Writer<S> {
    fn clone(&self) -> Self {
        Writer {
            encoding: self.encoding,
            inner: self.inner.clone(),
        }
    }
}

pub struct IrcStream<S> {
    pub encoding: EncodingRef,
    reader: BufReader<ReadHalf<S>>,
    writer: Writer<S>,
    async_buf: Vec<u8>,
    async_read: usize,
}

impl<S> IrcStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub fn new(stream: S, encoding: EncodingRef) -> Self {
        let (read_half, write_half) = stream.split();
        let writer = Writer {
            encoding,
            inner: Arc::new(Mutex::new(write_half)),
        };

        IrcStream {
            encoding,
            reader: BufReader::new(read_half),
            writer,
            async_buf: Vec::new(),
            async_read: 0,
        }
    }

    pub fn writer(&self) -> Writer<S> {
        self.writer.clone()
    }
}

impl<S> IrcStream<AllowStdIo<S>>
where
    S: Read + Write + Send,
{
    pub fn from_std(stream: S, encoding: EncodingRef) -> Self {
        IrcStream::new(AllowStdIo::new(stream), encoding)
    }
}

fn read_until_internal<R: AsyncBufRead + ?Sized>(
    mut reader: Pin<&mut R>,
    byte: u8,
    buf: &mut Vec<u8>,
    read: &mut usize,
    cx: &mut Context<'_>,
) -> Poll<Result<usize, AsyncIoError>> {
    loop {
        let (done, used) = {
            let available = ready!(reader.as_mut().poll_fill_buf(cx))?;
            if let Some(i) = memchr::memchr(byte, available) {
                buf.extend_from_slice(&available[..=i]);
                (true, i + 1)
            } else {
                buf.extend_from_slice(available);
                (false, available.len())
            }
        };
        reader.as_mut().consume(used);
        *read += used;
        if done || used == 0 {
            return Poll::Ready(Ok(mem::replace(read, 0)));
        }
    }
}

impl<S> Stream for IrcStream<S>
where
    S: AsyncRead + Unpin,
{
    type Item = Result<Message, StreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self {
            ref mut reader,
            ref mut async_buf,
            ref mut async_read,
            encoding,
            ..
        } = *self;

        let read = ready!(read_until_internal(
            Pin::new(reader),
            b'\n',
            async_buf,
            async_read,
            cx
        ))?;

        if read > 0 {
            let line = encoding.decode(async_buf, DecoderTrap::Ignore).unwrap();
            *async_read = 0;
            async_buf.clear();
            Poll::Ready(Some(Message::parse(&line).map_err(Into::into)))
        } else {
            Poll::Ready(None)
        }
    }
}

impl<S> IntoIterator for IrcStream<S>
where
    S: AsyncRead + Unpin,
{
    type Item = Result<Message, StreamError>;
    type IntoIter = BlockingStream<Self>;

    fn into_iter(self) -> Self::IntoIter {
        block_on_stream(self)
    }
}
