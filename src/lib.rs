

extern crate libc;
#[macro_use]
extern crate enum_primitive;
extern crate opusfile_sys;

use std::borrow::ToOwned;
use std::vec::Vec;
use std::rc::Rc;
use std::{mem, slice, string, io, ptr};
use std::io::{Read, Seek};
use std::ffi::CStr;
use enum_primitive::FromPrimitive;

use opusfile_sys as opus;


static CB : opus::OpusFileCallbacks = opus::OpusFileCallbacks {
    read:  read_cb,
    seek:  Some(seek_cb),
    tell:  Some(tell_cb),
    close: None,
};


enum_from_primitive! {
    pub enum OpusFileError
    {
        False         = opus::OP_FALSE as isize,
        Eof           = opus::OP_EOF as isize,
        Hole          = opus::OP_HOLE as isize,
        ERead         = opus::OP_EREAD as isize,
        EFault        = opus::OP_EFAULT as isize,
        EImpl         = opus::OP_EIMPL as isize,
        EInval        = opus::OP_EINVAL as isize,
        ENotFormat    = opus::OP_ENOTFORMAT as isize,
        EBadHeader    = opus::OP_EBADHEADER as isize,
        EVersion      = opus::OP_EVERSION as isize,
        ENotAudio     = opus::OP_ENOTAUDIO as isize,
        EBadPacket    = opus::OP_EBADPACKET as isize,
        EBadLink      = opus::OP_EBADLINK as isize,
        ENoSeek       = opus::OP_ENOSEEK as isize,
        EBadTimeStamp = opus::OP_EBADTIMESTAMP as isize,
    }
}


pub type OpusFileResult<T> = Result<T, OpusFileError>;


trait ReadSeek: Read + Seek { }
impl <T> ReadSeek for T where T : Read + Seek { }


enum DataSource <'s> {
    Slice (&'s [u8]),
    Read (&'s mut Read),
    ReadSeek (&'s mut ReadSeek),
}


pub struct OggOpusFile <'s>
{
    of   : *mut opus::OggOpusFile,
    data : DataSource<'s>,
}
impl <'s> OggOpusFile<'s>
{
    pub fn from_slice (data : &[u8]) -> OpusFileResult<OggOpusFile>
    {
        unsafe {
            let mut e: libc::c_int = 0;
            match opus::op_open_memory(data.as_ptr() as *const libc::c_uchar, data.len() as libc::size_t, &mut e as *mut libc::c_int) {
                of if of != ptr::null_mut() => Ok(OggOpusFile { of: of, data: DataSource::Slice(data), }),
                _ => Err(OpusFileError::from_isize(e as isize).unwrap()),
            }
        }
    }
    
    pub fn from_read (data : &mut Read) -> OpusFileResult<OggOpusFile>
    {
        unsafe {
            let mut e: libc::c_int = 0;
            match opus::op_open_callbacks(mem::transmute_copy(&data), &CB, ptr::null(), 0, &mut e as *mut libc::c_int) {
                of if of != ptr::null_mut() => Ok(OggOpusFile { of: of, data: DataSource::Read(data), }),
                _ => Err(OpusFileError::from_isize(e as isize).unwrap()),
            }
        }
    }
    
    pub fn from_read_seek (data : &mut ReadSeek) -> OpusFileResult<OggOpusFile>
    {
        unsafe {
            let mut e: libc::c_int = 0;
            match opus::op_open_callbacks(mem::transmute_copy(&data), &CB, ptr::null(), 0, &mut e as *mut libc::c_int) {
                of if of != ptr::null_mut() => Ok(OggOpusFile { of: of, data: DataSource::ReadSeek(data), }),
                _ => Err(OpusFileError::from_isize(e as isize).unwrap()),
            }
        }
    }
    
    
    pub fn seekable (&self) -> bool
    {
        unsafe {
            opus::op_seekable(self.of as *const opus::OggOpusFile) != 0
        }
    }
    
    
    pub fn link_count (&self) -> i32
    {
        unsafe {
            opus::op_link_count(self.of as *const opus::OggOpusFile) as i32
        }
    }
    
    
    pub fn serial_no (&self, li: i32 ) -> u32
    {
        unsafe {
            opus::op_serialno(self.of as *const opus::OggOpusFile, li as libc::c_int) as u32
        }
    }
    
    
    pub fn channel_count (&self, li: i32 ) -> i32
    {
        unsafe {
            opus::op_channel_count(self.of as *const opus::OggOpusFile, li as libc::c_int) as i32
        }
    }
    
    
    pub fn raw_total (&self, li: i32 ) -> i64
    {
        unsafe {
            opus::op_raw_total(self.of as *const opus::OggOpusFile, li as libc::c_int) as i64
        }
    }
    
    
    pub fn pcm_total (&self, li: i32 ) -> i64
    {
        unsafe {
            opus::op_pcm_total(self.of as *const opus::OggOpusFile, li as libc::c_int) as i64
        }
    }
    
    
    pub fn head (&self, li: i32) -> Option<OpusHead>
    {
        unsafe {
            match opus::op_head(self.of as *const opus::OggOpusFile, li as libc::c_int) {
                oh if oh != ptr::null() => Some(OpusHead{
                    version:           (*oh).version as i32,
                    channel_count:     (*oh).channel_count as u8,
                    pre_skip:          (*oh).pre_skip as i32,
                    input_sample_rate: (*oh).input_sample_rate as u32,
                    output_gain:       (*oh).output_gain as i32,
                    mapping_family:    (*oh).mapping_family as i32,
                    stream_count:      (*oh).stream_count as u8,
                    coupled_count:     (*oh).coupled_count as u8,
                    mapping:           (*oh).mapping[0 .. (2 * (*oh).coupled_count + ((*oh).stream_count - (*oh).coupled_count)) as usize].to_owned(),
                }),
                _ => None,
            }
        }
    }
    
    
    pub fn tags (&self, li: i32) -> Option<OpusTags>
    {
        unsafe {
            match opus::op_tags(self.of as *const opus::OggOpusFile, li as libc::c_int) {
                ot if ot != ptr::null() => Some(OpusTags {
                    user_comments: [0 .. (*ot).comments as usize].map(|ci| {
                        let len = *((*ot).comment_lengths as *const i32).offset(ci as isize) as usize;
                        String::from_utf8_lossy(slice::from_raw_parts(*((*ot).user_comments as *const *const u8).offset(ci as isize), len)).into_owned()
                    }).collect(),
                    vendor: CStr::from_ptr((*ot).vendor as *const i8).to_string_lossy().into_owned(),
                }),
                _ => None,
            }
        }
    }
    
    
    pub fn current_link (&self) -> i32
    {
        unsafe {
            opus::op_current_link(self.of as *const opus::OggOpusFile) as i32
        }
    }
    
    pub fn bitrate (&self, li: i32) -> i32
    {
        unsafe {
            opus::op_bitrate(self.of as *const opus::OggOpusFile, li as libc::c_int) as i32
        }
    }
    
    pub fn bitrate_instant (&self) -> i32
    {
        unsafe {
            opus::op_bitrate_instant(self.of as *const opus::OggOpusFile) as i32
        }
    }
    
    pub fn raw_tell (&self) -> i64
    {
        unsafe {
            opus::op_raw_tell(self.of as *const opus::OggOpusFile) as i64
        }
    }
    
    pub fn pcm_tell (&self) -> i64
    {
        unsafe {
            opus::op_pcm_tell(self.of as *const opus::OggOpusFile) as i64
        }
    }
    
    pub fn raw_seek (&mut self, offset: i64) -> OpusFileResult<()> {
        unsafe {
            match opus::op_raw_seek(self.of, offset) {
                0 => Ok(()),
                e => Err(OpusFileError::from_i32(e as i32).unwrap()),
            }
        }
    }
    
    pub fn pcm_seek (&mut self, offset: i64) -> OpusFileResult<()> {
        unsafe {
            match opus::op_raw_seek(self.of, offset) {
                0 => Ok(()),
                e => Err(OpusFileError::from_i32(e as i32).unwrap()),
            }
        }
    }
    
    pub fn read (&mut self, pcm: &mut [i16], li: &mut i32) -> OpusFileResult<i32>
    {
        unsafe {
            match opus::op_read(self.of, pcm.as_mut_ptr(), pcm.len() as libc::c_int, li as *mut i32 as *mut libc::c_int) {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n          => Ok(n as i32),
            }
        }
    }
    
    pub fn read_float (&mut self, pcm: &mut [libc::c_float], li: &mut i32) -> OpusFileResult<i32>
    {
        unsafe {
            match opus::op_read_float(self.of, pcm.as_mut_ptr(), pcm.len() as libc::c_int, li as *mut i32 as *mut libc::c_int) {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n          => Ok(n as i32),
            }
        }
    }
    
    pub fn read_stereo (&mut self, pcm: &mut [i16]) -> OpusFileResult<i32>
    {
        unsafe {
            match opus::op_read_stereo(self.of, pcm.as_mut_ptr(), pcm.len() as libc::c_int) {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n          => Ok(n as i32),
            }
        }
    }
    
    pub fn read_float_stereo (&mut self, pcm: &mut [f32]) -> OpusFileResult<i32>
    {
        unsafe {
            match opus::op_read_float_stereo(self.of, pcm.as_mut_ptr(), pcm.len() as libc::c_int) {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n          => Ok(n as i32),
            }
        }
    }
}


unsafe extern "C" fn read_cb (src: *mut libc::c_void, buf: *mut libc::c_uchar, size: libc::c_int) -> libc::c_int {
    fn fill_buffer (read : &mut Read, buf : &[u8]) -> usize
    {
        let mut i = 0;

        0
    }

    match *(src as *mut DataSource) {
        DataSource::Read(r) => fill_buffer(r, slice::from_raw_parts_mut(buf, size)),
        DataSource::ReadSeek(r) => fill_buffer(r),
        _ => -1,
    }
}


unsafe extern "C" fn seek_cb (src : *mut libc::c_void, pos: i64, style : libc::c_int) -> libc::c_int {
    let pos = match style {
        libc::SEEK_SET => io::SeekFrom::Start(pos as u64),
        libc::SEEK_CUR => io::SeekFrom::Current(pos),
        libc::SEEK_END => io::SeekFrom::End(pos),
        _ => return -1,
    };
    
    match *(src as *mut DataSource) {
        DataSource::ReadSeek(s) => {
            while let Err(e) = s.seek(pos) { match e.kind() {
                io::ErrorKind::Interrupted => continue,
                _ => return -1,
            }}
        }
        _ => -1,
    }
}


unsafe extern "C" fn tell_cb (src: *mut libc::c_void) -> i64 {
    match *(src as *mut DataSource) {
        DataSource::ReadSeek(s) => {
            while let Err(e) = s.seek(io::SeekFrom::Current(0)) { match e.kind() {
                io::ErrorKind::Interrupted => continue,
                _ => return -1,
            }}
        }
        _ => -1,
    }
}


pub struct OpusHead
{
    pub mapping           : Vec<u8>,
    pub version           : i32,
    pub pre_skip          : i32,
    pub input_sample_rate : u32,
    pub output_gain       : i32,
    pub mapping_family    : i32,
    pub channel_count     : u8,
    pub stream_count      : u8,
    pub coupled_count     : u8,
}


pub struct OpusTags
{
    pub user_comments : Vec<String>,
    pub vendor        : String,
}


