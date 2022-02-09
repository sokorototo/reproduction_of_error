use rayon::prelude::*;
use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

// The data in transit
const DATA: [u8; 200] = *b"5m4IreHCOsr9jGyKAIP5z1GmH40GKcJY6O0WzelRKpu8r3A8bHKXDxx4ka5cRc92qviKCe9t1E0PHfPb7Bovr0WEjL4BdrIpfSN4Vu1SCA0ivBpdb9j7K9MDH6NE6EwU2wTZTPHegzf2oJ9fk2Jb053H6t5YvgzT2BfsjP2j1KyVTzDwApzTVV7jhAfWPvreX7bHWbkH";

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

        let region = (entry.location) as usize..entry.location as usize + entry.offset;
        assert_eq!(&raw, &DATA[region]);

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

    (0..2).into_par_iter().for_each(|mock| {
        let entry = Entry::new(mock);
        reader.fetch_raw(&entry);
    })
}
