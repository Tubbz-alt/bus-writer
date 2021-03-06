extern crate bus_writer;

use bus_writer::*;
use std::io::{BufReader, Cursor, Read};
use std::fs::{self, File};
use std::process::exit;

fn main() {
    let data: Vec<u8> = [0u8; 1024 * 1024 * 5].into_iter()
        .zip([1u8; 1024 * 1024 * 5].into_iter())
        .cycle()
        .take(50 * 1024 * 1024)
        .fold(Vec::with_capacity(100 * 1024 * 1024), |mut acc, (&x, &y)| {
            acc.push(x);
            acc.push(y);
            acc
        });

    let mut source = Cursor::new(&data);

    let files = ["a", "b", "c", "d", "e", "f", "g", "h"];
    let mut temp_files = [
        fs::OpenOptions::new().read(true).write(true).create(true).open(files[0]).unwrap(),
        fs::OpenOptions::new().read(true).write(true).create(true).open(files[1]).unwrap(),
        fs::OpenOptions::new().read(true).write(true).create(true).open(files[2]).unwrap(),
        fs::OpenOptions::new().read(true).write(true).create(true).open(files[3]).unwrap(),
        fs::OpenOptions::new().read(true).write(true).create(true).open(files[4]).unwrap(),
        fs::OpenOptions::new().read(true).write(true).create(true).open(files[5]).unwrap(),
        fs::OpenOptions::new().read(true).write(true).create(true).open(files[6]).unwrap(),
        fs::OpenOptions::new().read(true).write(true).create(true).open(files[7]).unwrap(),
    ];

    let mut errored = false;
    let result = BusWriter::new(
        &mut source,
        &mut temp_files,
        // Reports progress of each device so that callers may create their own progress bars
        // for each destination being written to, as seen in System76's Popsicle GTK UI.
        |event| match event {
            BusWriterMessage::Written { id, bytes_written } => {
                println!("{}: {} total bytes written", files[id], bytes_written);
            }
            BusWriterMessage::Completed { id } => {
                println!("{}: Completed", files[id]);
            }
            BusWriterMessage::Errored { id, why } => {
                println!("{} errored: {}", files[id], why);
                errored = true;
            }
        },
        // Executed at certain points while writing to check if the process needs to be cancelled
        || false
    ).write();

    if let Err(why) = result {
        eprintln!("writing failed: {}", why);
        exit(1);
    } else if errored {
        eprintln!("an error occurred");
        exit(1);
    }

    eprintln!("finished writing; validating files");

    let result = BusVerifier::new(
        source,
        &mut temp_files,
        |event| match event {
            BusVerifierMessage::Read { id, bytes_read } => {
                println!("{}: {} bytes verified", files[id], bytes_read);
            }
            BusVerifierMessage::Valid { id } => {
                println!("{}: Validated", files[id]);
            }
            BusVerifierMessage::Invalid { id } => {
                println!("{}: Invalid", id);
                errored = true;
            }
            BusVerifierMessage::Errored { id, why } => {
                println!("{} errored while verifying: {}", files[id], why);
                errored = true;
            }
        },
        || false
    ).verify();

    if let Err(why) = result {
        eprintln!("writing failed: {}", why);
        exit(1);
    } else if errored {
        eprintln!("Error occurred");
        exit(1);
    }

    eprintln!("All files validated!");
}