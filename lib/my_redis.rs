use tokio::net::TcpStream;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt, BufWriter};
use mini_redis::{Frame, Result};
use mini_redis::frame::Error::Incomplete;
use bytes::{BytesMut, Buf};
use std::io::Cursor;

struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        // 实现读取帧的逻辑
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                }
                else {
                    return Err("connection reset by peer.".into());
                }
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        // 实现写入帧的逻辑
        match frame {
            Frame::Simple(s) => {
                self.stream.write_u8(b"+").await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(e) => {
                self.stream.write_u8(b"-").await?;
                self.stream.write_all(e.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(i) => {
                self.stream.write_u8(b":").await?;
                self.stream.write_decimal(*i).await?;
            }
            Frame::Null => {
                self.stream.write_all(b"$-1\r\n").await?;
            }
            Frame::Bulk(val) => {
                let len = val.len();
                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_arr) => unimplemented!(),
        }
        self.stream.flush().await?;
        Ok(())
    }
    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        // 实现帧解析的逻辑
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buf) {
            Ok(_) => {
                let frame_len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?;

                self.buffer.advance(frame_len);
                Ok(Some(frame))
            }
            Err(Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}