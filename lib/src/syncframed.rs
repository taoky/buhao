use tokio_util::bytes::BytesMut;

#[derive(Debug)]
pub struct SyncFramed<T, U, EncoderItem> {
    inner: T,
    codec: U,
    _encoder_item: std::marker::PhantomData<EncoderItem>,
}

impl<T, U, EncoderItem> SyncFramed<T, U, EncoderItem>
where
    T: std::io::Read + std::io::Write,
    U: tokio_util::codec::Decoder + tokio_util::codec::Encoder<EncoderItem>,
{
    pub fn new(inner: T, codec: U) -> Self {
        Self {
            inner,
            codec,
            _encoder_item: std::marker::PhantomData,
        }
    }

    pub fn send(&mut self, item: EncoderItem) -> Result<(), std::io::Error> {
        let mut buf = BytesMut::new();
        self.codec
            .encode(item, &mut buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to encode"))?;
        self.inner.write_all(&buf)?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<U::Item, std::io::Error> {
        let mut buf = BytesMut::new();
        loop {
            let mut inner_buf = [0; 1024];
            let n = self.inner.read(&mut inner_buf)?;
            // can it be decoded?
            buf.extend_from_slice(&inner_buf[..n]);
            if let Some(item) = self
                .codec
                .decode(&mut buf)
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to decode"))?
            {
                return Ok(item);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::TcpStream;

    use super::*;
    use tokio_util::codec::length_delimited::LengthDelimitedCodec;

    #[test]
    fn it_works() {
        let codec = LengthDelimitedCodec::new();
        let stream = TcpStream::connect("localhost:8000").unwrap();
        let mut _framed = SyncFramed::new(stream, codec);
        _framed.send("hello".into()).unwrap();
        let _item = _framed.recv().unwrap();
        println!("{:?}", _item);
    }
}
