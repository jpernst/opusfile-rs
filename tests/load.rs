


extern crate opusfile;

use std::io;


static OPUS_DATA : &'static [u8] = include_bytes!("bach.opus");


#[test]
fn slice ()
{
    let _ = opusfile::OggOpusFile::from_slice(OPUS_DATA).unwrap();
}

#[test]
fn reader ()
{
    let mut cursor = io::Cursor::new(OPUS_DATA);
    let _ = opusfile::OggOpusFile::from_read(&mut cursor).unwrap();
}


