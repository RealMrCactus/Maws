use std::fs::{File, OpenOptions};
use std::io::{self, Error, Read, Write, SeekFrom, Seek};
use rand::*;

macro_rules! read_sysfs {
    ($dev:expr, $file:expr) => {{
        let path = format!("/sys/block/{}/{}", $dev, $file);
        let content = std::fs::read_to_string(&path)
            .expect(&format!("Failed to read sysfs path: {}", path));

        content.trim().parse()
            .expect("Failed to parse data")
    }};
}

#[derive(Clone)]
struct drive {
    location: String,
    size: u64,
    rotational: bool
}

fn get_drive_info(dev: &str) -> drive {
    let mut dev = drive {
        location: String::from(dev),
        size: 0,
        rotational: true
    };

    let sectors: u64 = read_sysfs!(dev.location, "size");
    dev.size = sectors * 512;

    match read_sysfs!(dev.location, "queue/rotational") {
        1 => dev.rotational = true,
        0 => dev.rotational = false,
        _ => panic!("Unexpected value in rotational sysfs!")
    }

    dev
}

fn payload(chunk_size: usize, buffers: usize) -> Vec<Vec<u8>> {
    let mut buffer_pool: Vec<Vec<u8>> = Vec::with_capacity(buffers);
    let mut rng = rand::rng();

    for _ in 0..buffers {
        let mut buffer = vec![0u8; chunk_size * 1024 * 1024];

        rng.fill_bytes(&mut buffer);
        buffer_pool.push(buffer);
    }

    buffer_pool
}

fn decapitate(target: drive, buffer_pool: Vec<Vec<u8>>, chunk_size: usize) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(&target.location).expect("Failed to open drive for decapitation.");
    
    let mut rng = rand::rng();
    let pool_len = buffer_pool.len();
    
    let index_start = rng.random_range(0..pool_len);
    let payload_start = &buffer_pool[index_start];

    file.seek(SeekFrom::Start(0)).expect("Failed to seek to position 0");
    file.write_all(payload_start).expect("Failed to decapitate start.");    

    if target.size > (chunk_size as u64) {
        let end_offset = target.size - (chunk_size as u64);

        let index_end = rng.random_range(0..pool_len);
        let payload_end = &buffer_pool[index_end];
        
        file.seek(SeekFrom::Start(end_offset)).expect("Failed to seek to end offset.");
        file.write_all(&payload_end).expect("Failed to decapitate end");
    }

    file.sync_all().expect("Failed to sync decapitations.");

    Ok(())
}

fn main() {
    let buffers = 8;
    let chunk_size = 16;

    let buffer_pool = payload(chunk_size, buffers);
    let mut dev = get_drive_info("sdc");
    
    decapitate(dev.clone(), buffer_pool, chunk_size).expect("Failed to decapitate");

    println!("\n\nSize (in bytes): {}\nRotational: {}", dev.size, dev.rotational);
}
