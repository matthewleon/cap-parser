#[macro_use]
extern crate derivative;

use std::fs;
use fs::File;
use std::io::Read;

use image::ImageResult;
use nom::error::VerboseError;

use crate::parser::parse::packet;
use crate::parser::renderer::{Handler, Screen};

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

        let text = do_parse(&buffer);
        println!("{}", text.as_str());

        Ok(())
    })
}

fn do_parse(i: &[u8]) -> String {
    let mut handler = Handler::new();
    let mut frame = 0;

    let mut rest = i;
    let mut fnames = String::new();

    while !rest.is_empty() {
        match packet::<VerboseError<&[u8]>>(&rest) {
            Ok((remains, packet)) => {
                rest = remains;
                match handler.handle(packet) {
                    Ok(image) => match image {
                        Some(img) => {
                            match display_to_text(frame, &img) {
                                Ok(fname) => {
                                    fnames.push_str(format!("{}\n", fname).as_str());
                                }
                                Err(error) => eprintln!("error {:#?}\n", error),
                            };
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
    return fnames;
}

fn display_to_text(frame: u32, d: &Screen) -> ImageResult<String> {
    let fname = format!("tmp/sub-{}.tiff", frame);
    d.image.save(&fname)?;
    return Ok(fname);

    // TODO: finally output JSON with this info
    // Ok(text.map(|data| format!(
    //     "{}\n{} --> {}\n{}\n\n",
    //     frame + 1,
    //     format_timestamp_microsec(d.begin_mis),
    //     format_timestamp_microsec(d.begin_mis + d.dur_mis),
    //     data
    // )))
}
