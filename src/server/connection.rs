use std::io::Read;
use std::sync::{Arc, Mutex};
use std::{net::TcpStream, ptr};

use crate::world::update::WorldUpdate;

#[derive(PartialEq, Eq)]
enum StreamReadState {
    /// this state occurs be in two cases:
    /// 1. there has been no data sent yet
    /// 2. the parser just finished reading data for one type and
    /// will set the next state in the next pass
    Unset,
    /// 0
    BlockUpdate,
    /// 1
    PreMessage,
    /// 2
    Message,
    /// 3
    PlayerPos,
}

impl StreamReadState {
    fn needs_size(&self) -> Option<usize> {
        match *self {
            StreamReadState::Unset => Some(1),
            StreamReadState::BlockUpdate => Some(4), // may change. Who knows
            StreamReadState::PreMessage => Some(1),
            StreamReadState::Message => None,
            StreamReadState::PlayerPos => Some(12),
        }
    }
}

impl From<u8> for StreamReadState {
    fn from(value: u8) -> Self {
        use StreamReadState::*;
        match value {
            0 => BlockUpdate,
            1 => PreMessage,
            2 => Message,
            3 => PlayerPos,
            _ => unreachable!(),
        }
    }
}

const READ_BUF_SIZE: usize = 1024;

pub struct ClientConnection {
    stream: TcpStream,
    state: StreamReadState,
    par_buf: [u8; 256],
    pb_len: usize,
    updates: Arc<Mutex<Vec<WorldUpdate>>>,
}

impl ClientConnection {
    pub fn new(stream: TcpStream, updates: Arc<Mutex<Vec<WorldUpdate>>>) -> ClientConnection {
        ClientConnection {
            stream,
            state: StreamReadState::Unset,
            par_buf: [0; 256],
            pb_len: 0,
            updates,
        }
    }

    pub fn read_from_stream(&mut self) {
        let mut buf = [0; READ_BUF_SIZE];

        // todo: proper error handling here
        self.stream.read(&mut buf).expect("couldn't read bytes");

        self.read_bytes(&buf, READ_BUF_SIZE);
    }

    pub fn read_bytes(&mut self, bytes: &[u8], len: usize) {
        let mut amt_read = 0;
        let mut pbi = 0;
        let mut dynamic_size = 0;
        loop {
            use StreamReadState::*;
            let bytes_left = &bytes[amt_read..];
            let len_left = len - amt_read;
            let n = self.state.needs_size();
            let n = if let Some(n) = n { n } else { dynamic_size };
            let mut data = Vec::with_capacity(n);

            if !self.try_read(bytes_left, &mut data[..], n, len_left, &mut pbi) {
                break;
            }

            match self.state {
                Unset => self.switch_state(data[0]),
                BlockUpdate => todo!(), // extract block update from it
                PreMessage => dynamic_size = data[0] as usize,
                Message => todo!(), // add to chat log or something
                PlayerPos => todo!(), // update recorded position
            }

            amt_read += n;
        }

        let pb_left = self.pb_len - pbi;
        unsafe {
            ptr::copy(
                self.par_buf.as_ptr().add(pbi),
                self.par_buf.as_mut_ptr(),
                pb_left,
            );
        }
    }

    fn switch_state(&mut self, new_state: u8) {
        self.state = new_state.into();
    }

    fn try_read(
        &mut self,
        in_buf: &[u8],
        out_buf: &mut [u8],
        read_amt: usize,
        in_len: usize,
        pbi: &mut usize,
    ) -> bool {
        if self.pb_len + in_len < read_amt {
            return false;
        }

        // todo: testing all of this behavior (very important!)
        // also probably fuzzy testing
        let pb_amt = read_amt.min(self.pb_len);
        let in_amt = read_amt - pb_amt;

        out_buf[0..pb_amt].copy_from_slice(&self.par_buf[*pbi..(*pbi + pb_amt)]);

        out_buf[pb_amt..(pb_amt + in_amt)].copy_from_slice(&in_buf[0..in_amt]);

        *pbi += pb_amt;

        self.pb_len -= pb_amt;

        return true;
    }
}

#[cfg(test)]
mod tests {
    use std::{
        net::{TcpListener, TcpStream},
        sync::{Arc, Mutex},
    };

    use rand::{distributions::Uniform, Rng};

    use super::ClientConnection;

    #[test]
    fn test_try_read_fuzzy() {
        let _dummy_listener = TcpListener::bind("127.0.0.1:8421").unwrap();
        let dummy_stream = TcpStream::connect("127.0.0.1:8421").unwrap();
        let cc = ClientConnection::new(dummy_stream, Arc::new(Mutex::new(Vec::new())));

        let mut r = rand::thread_rng();
        let range = Uniform::new(u8::MIN, u8::MAX);

        let test_data = (0..4096).map(|_| r.sample(&range)).collect::<Vec<_>>();

        let mut res = [0; 1024];

        // todo: finish after parsing is finished
    }
}
