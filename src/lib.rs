

extern crate libc;
#[macro_use]
extern crate enum_primitive;
extern crate opusfile_sys;

use std::borrow::ToOwned;
use std::vec::Vec;
use std::{mem, slice, io, ptr};
use std::io::{Read, Seek};
use std::ffi::CStr;
use std::fmt::{self, Display};
use std::error::Error as StdError;
use enum_primitive::FromPrimitive;

use opusfile_sys as ffi;


static CB : ffi::OpusFileCallbacks = ffi::OpusFileCallbacks {
    read:  Some(read_cb),
    seek:  Some(seek_cb),
    tell:  Some(tell_cb),
    close: None,
};


enum_from_primitive! {
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum OpusFileError
    {
        False         = -1,
        Eof           = -2,
        Hole          = -3,
        ERead         = -128,
        EFault        = -129,
        EImpl         = -130,
        EInval        = -131,
        ENotFormat    = -132,
        EBadHeader    = -133,
        EVersion      = -134,
        ENotAudio     = -135,
        EBadPacket    = -136,
        EBadLink      = -137,
        ENoSeek       = -138,
        EBadTimeStamp = -139,
    }
}
impl Display for OpusFileError
{
    fn fmt (&self, fmt : &mut fmt::Formatter) -> Result<(), fmt::Error>
    {
        write!(fmt, "{}", self.description())
    }
}
impl StdError for OpusFileError
{
    fn description (&self) -> &str
    {
        match *self {
            OpusFileError::False         => "False",
            OpusFileError::Eof           => "Eof",
            OpusFileError::Hole          => "Hole",
            OpusFileError::ERead         => "ERead",
            OpusFileError::EFault        => "EFault",
            OpusFileError::EImpl         => "EImpl",
            OpusFileError::EInval        => "EInval",
            OpusFileError::ENotFormat    => "ENotFormat",
            OpusFileError::EBadHeader    => "EBadHeader",
            OpusFileError::EVersion      => "EVersion",
            OpusFileError::ENotAudio     => "ENotAudio",
            OpusFileError::EBadPacket    => "EBadPacket",
            OpusFileError::EBadLink      => "EBadLink",
            OpusFileError::ENoSeek       => "ENoSeek",
            OpusFileError::EBadTimeStamp => "EBadTimeStamp",
        }
    }
}


pub type OpusFileResult<T> = Result<T, OpusFileError>;


pub trait ReadSeek: Read + Seek { }
impl <T> ReadSeek for T where T : Read + Seek { }


enum DataSource <'d> {
    Read (&'d mut Read),
    ReadSeek (&'d mut ReadSeek),
}


pub struct OggOpusFile <'d>
{
    of    : *mut ffi::OggOpusFile,
    _data : *mut DataSource<'d>,
}
impl <'d> OggOpusFile<'d>
{
    pub fn from_slice (data : &[u8]) -> OpusFileResult<OggOpusFile>
    {
        unsafe {
            let mut e: libc::c_int = 0;
            match ffi::op_open_memory(data.as_ptr() as *const libc::c_uchar, data.len() as libc::size_t, &mut e as *mut libc::c_int) {
                of if of != ptr::null_mut() => Ok(OggOpusFile { of: of, _data: ptr::null_mut(), }),
                _ => Err(OpusFileError::from_isize(e as isize).unwrap()),
            }
        }
    }
    
    pub fn from_read (data : &mut Read) -> OpusFileResult<OggOpusFile>
    {
        unsafe {
            let mut e: libc::c_int = 0;
            let data = mem::transmute(Box::new(DataSource::Read(data)));
            match ffi::op_open_callbacks(data, &CB, ptr::null(), 0, &mut e as *mut libc::c_int) {
                of if of != ptr::null_mut() => Ok(OggOpusFile { of: of, _data: data as *mut _, }),
                _ => Err(OpusFileError::from_isize(e as isize).unwrap()),
            }
        }
    }
    
    pub fn from_read_seek (data : &mut ReadSeek) -> OpusFileResult<OggOpusFile>
    {
        unsafe {
            let mut e: libc::c_int = 0;
            let data = mem::transmute(Box::new(DataSource::ReadSeek(data)));
            match ffi::op_open_callbacks(data, &CB, ptr::null(), 0, &mut e as *mut libc::c_int) {
                of if of != ptr::null_mut() => Ok(OggOpusFile { of: of, _data: data as *mut _, }),
                _ => Err(OpusFileError::from_isize(e as isize).unwrap()),
            }
        }
    }
    
    
    pub fn seekable (&self) -> bool
    {
        unsafe { ffi::op_seekable(self.of as *const ffi::OggOpusFile) != 0 }
    }
    
    
    pub fn link_count (&self) -> i32
    {
        unsafe { ffi::op_link_count(self.of as *const ffi::OggOpusFile) as i32 }
    }
    
    
    pub fn serial_no (&self, li : Option<i32> ) -> u32
    {
        unsafe { ffi::op_serialno(self.of as *const ffi::OggOpusFile, li.unwrap_or(-1) as libc::c_int) as u32 }
    }
    
    
    pub fn channel_count (&self, li : Option<i32> ) -> i32
    {
        unsafe { ffi::op_channel_count(self.of as *const ffi::OggOpusFile, li.unwrap_or(-1) as libc::c_int) as i32 }
    }
    
    
    pub fn raw_total (&self, li : Option<i32> ) -> OpusFileResult<i64>
    {
        unsafe {
            match ffi::op_raw_total(self.of as *const ffi::OggOpusFile, li.unwrap_or(-1) as libc::c_int) as i64 {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n => Ok(n),
            }
        }
    }
    
    
    pub fn pcm_total (&self, li : Option<i32> ) -> OpusFileResult<i64>
    {
        unsafe {
            match ffi::op_pcm_total(self.of as *const ffi::OggOpusFile, li.unwrap_or(-1) as libc::c_int) as i64 {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n => Ok(n),
            }
        }
    }
    
    
    pub fn head (&self, li : Option<i32>) -> Option<OpusHead>
    {
        unsafe {
            match ffi::op_head(self.of as *const ffi::OggOpusFile, li.unwrap_or(-1) as libc::c_int) {
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
    
    
    pub fn tags (&self, li : Option<i32>) -> Option<OpusTags>
    {
        unsafe {
            match ffi::op_tags(self.of as *const ffi::OggOpusFile, li.unwrap_or(-1) as libc::c_int) {
                ot if ot != ptr::null() => Some(OpusTags {
                    user_comments: (0 .. (*ot).comments as usize).filter_map(|ci| {
                        let len = *((*ot).comment_lengths as *const i32).offset(ci as isize) as usize;
                        let comment = String::from_utf8_lossy(slice::from_raw_parts(*((*ot).user_comments as *const *const u8).offset(ci as isize), len));
                        let mut split = comment.splitn(2, '=').fuse();
                        if let (Some(tag), Some(data)) = (split.next(), split.next()) {
                            Some((tag.to_uppercase(), data.to_owned()))
                        } else {
                            None
                        }
                    }).collect(),
                    vendor: String::from_utf8_lossy(CStr::from_ptr((*ot).vendor as *const i8).to_bytes()).into_owned(),
                }),
                _ => None,
            }
        }
    }
    
    
    pub fn current_link (&self) -> OpusFileResult<i32>
    {
        unsafe {
            match ffi::op_current_link(self.of as *const ffi::OggOpusFile) as i32 {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n => Ok(n),
            }
        }
    }
    
    pub fn bitrate (&self, li : Option<i32>) -> OpusFileResult<i32>
    {
        unsafe {
            match ffi::op_bitrate(self.of as *const ffi::OggOpusFile, li.unwrap_or(-1) as libc::c_int) as i32 {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n => Ok(n),
            }
        }
    }
    
    pub fn bitrate_instant (&self) -> OpusFileResult<i32>
    {
        unsafe {
            match ffi::op_bitrate_instant(self.of as *mut ffi::OggOpusFile) as i32 {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n => Ok(n),
            }
        }
    }
    
    pub fn raw_tell (&self) -> OpusFileResult<i64>
    {
        unsafe {
            match ffi::op_raw_tell(self.of as *const ffi::OggOpusFile) as i64 {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n => Ok(n),
            }
        }
    }
    
    pub fn pcm_tell (&self) -> OpusFileResult<i64>
    {
        unsafe {
            match ffi::op_pcm_tell(self.of as *const ffi::OggOpusFile) as i64 {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n => Ok(n),
            }
        }
    }
    
    pub fn raw_seek (&mut self, offset : i64) -> OpusFileResult<()> {
        unsafe {
            match ffi::op_raw_seek(self.of, offset) {
                0 => Ok(()),
                e => Err(OpusFileError::from_i32(e as i32).unwrap()),
            }
        }
    }
    
    pub fn pcm_seek (&mut self, offset : i64) -> OpusFileResult<()> {
        unsafe {
            match ffi::op_pcm_seek(self.of, offset) {
                0 => Ok(()),
                e => Err(OpusFileError::from_i32(e as i32).unwrap()),
            }
        }
    }
    
    pub fn read (&mut self, pcm : &mut [i16], li : Option<&mut i32>) -> OpusFileResult<i32>
    {
        unsafe {
            let li = li.map(|li| li as *mut i32 as *mut libc::c_int).unwrap_or(ptr::null_mut());
            match ffi::op_read(self.of, pcm.as_mut_ptr(), pcm.len() as libc::c_int, li) {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n          => Ok(n as i32),
            }
        }
    }
    
    pub fn read_float (&mut self, pcm : &mut [libc::c_float], li : Option<&mut i32>) -> OpusFileResult<i32>
    {
        unsafe {
            let li = li.map(|li| li as *mut i32 as *mut libc::c_int).unwrap_or(ptr::null_mut());
            match ffi::op_read_float(self.of, pcm.as_mut_ptr(), pcm.len() as libc::c_int, li) {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n          => Ok(n as i32),
            }
        }
    }
    
    pub fn read_stereo (&mut self, pcm : &mut [i16]) -> OpusFileResult<i32>
    {
        unsafe {
            match ffi::op_read_stereo(self.of, pcm.as_mut_ptr(), pcm.len() as libc::c_int) {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n          => Ok(n as i32),
            }
        }
    }
    
    pub fn read_float_stereo (&mut self, pcm : &mut [f32]) -> OpusFileResult<i32>
    {
        unsafe {
            match ffi::op_read_float_stereo(self.of, pcm.as_mut_ptr(), pcm.len() as libc::c_int) {
                e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
                n          => Ok(n as i32),
            }
        }
    }
}
impl <'d> Drop for OggOpusFile<'d>
{
    fn drop (&mut self)
    {
        unsafe {
            ffi::op_free(self.of);
            if self._data != ptr::null_mut() {
                let _ : Box<DataSource<'d>> = mem::transmute(self._data);
            }
        }
    }
}


extern "C" fn read_cb (src : *mut libc::c_void, buf : *mut libc::c_uchar, size : libc::c_int) -> libc::c_int {
    fn fill_buffer <R> (mut read : R, mut buf : &mut [u8]) -> usize
        where R : Read
    {
        let mut i = 0;
        while i < buf.len() {
            let buf = &mut buf[i ..];
            match read.read(buf) {
                Ok(0) => break,
                Ok(n) => i += n,
                Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }

        i
    }

    unsafe {
        let buf = slice::from_raw_parts_mut(buf, size as usize);
        match *(src as *mut DataSource) {
            DataSource::Read(ref mut r) => fill_buffer(r, buf) as i32,
            DataSource::ReadSeek(ref mut r) => fill_buffer(r, buf) as i32,
        }
    }
}


extern "C" fn seek_cb (src : *mut libc::c_void, pos: i64, style : libc::c_int) -> libc::c_int {
    let pos = match style {
        libc::SEEK_SET => io::SeekFrom::Start(pos as u64),
        libc::SEEK_CUR => io::SeekFrom::Current(pos),
        libc::SEEK_END => io::SeekFrom::End(pos),
        _ => return -1,
    };
    
    unsafe {
        match *(src as *mut DataSource) {
            DataSource::ReadSeek(ref mut s) => {
                loop {
                    match s.seek(pos) {
                        Ok(_) => return 0,
                        Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(_) => return -1,
                    }
                }
            }
            _ => -1,
        }
    }
}


extern "C" fn tell_cb (src : *mut libc::c_void) -> i64 {
    unsafe {
        match *(src as *mut DataSource) {
            DataSource::ReadSeek(ref mut s) => {
                loop {
                    match s.seek(io::SeekFrom::Current(0)) {
                        Ok(pos) => return pos as i64,
                        Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
                        Err(_) => return -1,
                    }
                }
            }
            _ => -1,
        }
    }
}


#[derive(Clone, PartialEq, Eq, Debug)]
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


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OpusTags
{
    pub user_comments : Vec<(String, String)>,
    pub vendor        : String,
}


