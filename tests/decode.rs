

extern crate opusfile;

use std::mem;


static OPUS_DATA : &'static [u8] = include_bytes!("bach.opus");


#[test]
fn read_all ()
{
    let mut opus = opusfile::OggOpusFile::from_slice(OPUS_DATA).unwrap();

    let mut data : [i16; 11520] = unsafe { mem::uninitialized() };
    let head_total = opus.pcm_total(None).unwrap();

    let mut total = 0;
    while let Ok(count) = opus.read_stereo(&mut data[..]) {
        if count == 0 {
            break;
        }

        total += count as i64;
    }

    assert_eq!(head_total, total);
}


