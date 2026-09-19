//! Compile a FileDescriptorSet into prost/tonic and pbjson sources.

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use prost::Message;

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1);
    let descriptor = PathBuf::from(args.next().expect("descriptor path"));
    let out_dir = PathBuf::from(args.next().expect("output directory"));
    fs::create_dir_all(&out_dir)?;
    let bytes = fs::read(&descriptor)?;
    let fds = prost_types::FileDescriptorSet::decode(bytes.as_slice())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    tonic_prost_build::configure()
        .build_client(true)
        .build_server(true)
        .out_dir(&out_dir)
        .compile_fds(fds)?;
    pbjson_build::Builder::new()
        .out_dir(&out_dir)
        .register_descriptors(&bytes)?
        .build(&[".hctl2.control.v1"])?;
    let mut manifest = fs::File::create(out_dir.join("mod.rs"))?;
    writeln!(
        manifest,
        "include!(\"hctl2.control.v1.rs\");\ninclude!(\"hctl2.control.v1.serde.rs\");"
    )?;
    Ok(())
}
