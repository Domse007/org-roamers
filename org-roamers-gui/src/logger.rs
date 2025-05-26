use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone)]
pub struct LogBuffer<const N: usize> {
    inner: Arc<Mutex<([String; N], usize)>>,
}

impl<const N: usize> LogBuffer<N> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(([const { String::new() }; N], 0))),
        }
    }

    pub fn get_logs(&self) -> String {
        let (ref buf, head) = *self.inner.lock().unwrap();

        let before = &buf[0..N];
        let after = &buf[head..N - 1];

        after.join("") + &before.join("")
    }
}

impl<const N: usize> std::io::Write for LogBuffer<N> {
    fn write(&mut self, s: &[u8]) -> Result<usize, std::io::Error> {
        let (ref mut buf, ref mut head) = *self.inner.lock().unwrap();
        let s = str::from_utf8(s).unwrap();
        buf[*head] = s.to_string();
        *head = (*head + 1) % N;
        Ok(s.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a, const N: usize> MakeWriter<'a> for LogBuffer<N> {
    type Writer = LogBuffer<N>;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}
