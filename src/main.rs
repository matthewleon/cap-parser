#[macro_use]
extern crate derivative;

use std::fs;
use fs::File;
use std::io::{Read, Write};

use image::ImageResult;
use nom::error::VerboseError;

use crate::parser::parse::packet;
use crate::parser::renderer::{Handler, Screen};
use threadpool::ThreadPool;

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod parser;

fn timeit<Ret, F: FnOnce() -> Ret>(f: F) -> Ret {
    let before = std::time::Instant::now();
    let result = f();
    let after = std::time::Instant::now();
    println!("took {:?}", after - before);

    result
}

fn main() -> std::io::Result<()> {
    timeit(|| {
        let mut f = File::open("subs.sup")?;
        let mut buffer = Vec::with_capacity(f.metadata()?.len() as usize);
        f.read_to_end(&mut buffer)?;

        let mut fout = File::create("subs.srt")?;
        let text = do_parse(&buffer);
        fout.write(text.as_bytes())?;

        Ok(())
    })
}

fn do_parse(i: &[u8]) -> String {
    let mut handler = Handler::new();
    let mut frame = 0;

    let mut rest = i;

    let pool = ThreadPool::new(num_cpus::get());
    while !rest.is_empty() {
        match packet::<VerboseError<&[u8]>>(&rest) {
            Ok((remains, packet)) => {
                rest = remains;
                match handler.handle(packet) {
                    Ok(image) => match image {
                        Some(img) => {
                            pool.execute(move || match display_to_text(frame, &img) {
                                Ok(()) => {}
                                Err(error) => eprintln!("error {:#?}\n", error),
                            });
                            frame = frame + 1;
                        }
                        None => {}
                    },
                    Err(error) => {
                        eprintln!("error! {:#?}\n", error);
                        return "error".to_string();
                    }
                }
            }
            Err(error) => {
                eprintln!("error! {:#?}\n", error);
                return "error".to_string();
            }
        }
    }
    pool.join();
    return String::new();
}

fn display_to_text(frame: u32, d: &Screen) -> ImageResult<()> {
    let fname = format!("tmp/sub-{}.tiff", frame);
    d.image.save(&fname)

    // TODO: finally output JSON with this info
    // Ok(text.map(|data| format!(
    //     "{}\n{} --> {}\n{}\n\n",
    //     frame + 1,
    //     format_timestamp_microsec(d.begin_mis),
    //     format_timestamp_microsec(d.begin_mis + d.dur_mis),
    //     data
    // )))
}
