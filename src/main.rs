use rayon::prelude::*;
use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

// The data in transit
const DATA: [u8; 200] = [69u8; 200];

// A reader that can be sent over threads as it is both `Send` and `Sync`
struct Reader<T: Read + Seek>(Arc<Mutex<T>>);

impl<T: Read + Seek> Reader<T> {
    fn fetch_raw(&self, entry: &Entry) -> Vec<u8> {
        let offset = entry.offset as usize;
        let mut raw = Vec::with_capacity(offset);

        // This is ok since we never **read** from the vector
        #[allow(clippy::uninit_vec)]
        unsafe {
            raw.set_len(offset);
        }

        {
            let mut guard = self.0.lock().unwrap();

            guard.seek(SeekFrom::Start(entry.location)).unwrap();
            guard.read_exact(raw.as_mut_slice()).unwrap();
        }

        assert_eq!(&raw, &DATA[0..entry.offset]);

        raw
    }

}

/// An entry from which one can read data from a reader
struct Entry {
    offset: usize,
    location: u64,
}

impl Entry {
    fn new(location: u64) -> Self {
        Entry {
            offset: 20,
            location,
        }
    }
}

fn main() {
    let source = Cursor::new(DATA);
    let reader = Reader(Arc::new(Mutex::new(source)));

    (0..180).into_par_iter().for_each(|mock|{
        let entry = Entry::new(mock);
        reader.fetch_raw(&entry);
    })
}
