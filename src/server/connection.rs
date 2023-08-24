use std::{net::TcpStream, ptr};
use std::io::Read;

enum StreamReadState {
    None,
    BlockUpdate,
    PlayerPos,
}

const READ_BUF_SIZE: usize = 1024;

pub struct ClientConnection {
    stream: TcpStream,
    state: StreamReadState,
    par_buf: [u8; 256],
    pb_len: usize,
    // todo: add updates mutex thing
}

impl ClientConnection {
    pub fn new(stream: TcpStream) -> ClientConnection {
        ClientConnection {
            stream,
            state: StreamReadState::None,
            par_buf: [0; 256],
            pb_len: 0,
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
        loop {
            let bytes_left = &bytes[amt_read..];
            let len_left = len - amt_read;
            match self.state {
                StreamReadState::None => {
                    let mut byte = [0];
                    if !self.try_read(bytes_left, &mut byte, 1, len_left, &mut pbi) {
                        break;
                    }
                    self.switch_state(byte[0]);
                    amt_read += 1;
                }
                StreamReadState::BlockUpdate => {}
                StreamReadState::PlayerPos => {}
            }
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

    fn switch_state(&mut self, new_state: u8) {
        todo!()
    }
}
