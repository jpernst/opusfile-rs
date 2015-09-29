

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

use opusfile_sys as opus;


static CB : opus::OpusFileCallbacks = opus::OpusFileCallbacks {
	read:  read_cb,
	seek:  seek_cb,
	tell:  tell_cb,
	close: ptr::null(),
};


enum_from_primitive! {
    pub enum OpusFileError
    {
        False         = opus::OP_FALSE as i32,
        Eof           = opus::OP_EOF as i32,
        Hole          = opus::OP_HOLE as i32,
        ERead         = opus::OP_EREAD as i32,
        EFault        = opus::OP_EFAULT as i32,
        EImpl         = opus::OP_EIMPL as i32,
        EInval        = opus::OP_EINVAL as i32,
        ENotFormat    = opus::OP_ENOTFORMAT as i32,
        EBadHeader    = opus::OP_EBADHEADER as i32,
        EVersion      = opus::OP_EVERSION as i32,
        ENotAudio     = opus::OP_ENOTAUDIO as i32,
        EBadPacket    = opus::OP_EBADPACKET as i32,
        EBadLink      = opus::OP_EBADLINK as i32,
        ENoSeek       = opus::OP_ENOSEEK as i32,
        EBadTimeStamp = opus::OP_EBADTIMESTAMP as i32,
    }
}


pub type OpusFileResult<T> = Result<T, OpusFileError>;


trait ReadSeek: Read + Seek { }
impl <T> ReadSeek for T where T : Read + Seek { }


enum DataSource <'s> {
    Slice (&'s mut [u8]),
	Read (&mut Read + 's),
	ReadSeek (&mut ReadSeek + 's),
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
				of if of != ptr::null() => Ok(OggOpusFile { of: of, data: DataSource::Slice(data), }),
				_ => Err(OpusFileError::from_isize(e as isize).unwrap()),
			}
		}
	}
	
	pub fn from_read (data : &Read + 'static) -> OpusFileResult<OggOpusFile>
	{
		unsafe {
			let mut e: libc::c_int = 0;
			match opus::op_open_callbacks(mem::transmute_copy(&data), &CB, ptr::null(), 0, &mut e as *mut libc::c_int) {
				of if of != ptr::null() => Ok(OggOpusFile { of: of, data: DataSource::Read(data), }),
				_ => Err(OpusFileError::from_isize(e as isize).unwrap()),
			}
		}
	}
	
	pub fn from_read_seek (data : &ReadSeek + 'static) -> OpusFileResult<OggOpusFile>
	{
		unsafe {
			let mut e: libc::c_int = 0;
			match opus::op_open_callbacks(mem::transmute_copy(&data), &CB, ptr::null(), 0, &mut e as *mut libc::c_int) {
				of if of != ptr::null() => Ok(OggOpusFile { of: of, data: DataSource::ReadSeek(data), }),
				_ => Err(OpusFileError::from_isize(e as isize).unwrap()),
			}
		}
	}
	
	
	pub fn seekable (self: &OggOpusFile) -> bool
	{
		unsafe {
			opus::op_seekable(self._of as *const opus::OggOpusFile) != 0
		}
	}
	
	
	pub fn link_count (self: &OggOpusFile) -> i32
	{
		unsafe {
			opus::op_link_count(self._of as *const opus::OggOpusFile) as i32
		}
	}
	
	
	pub fn serial_no (self: &OggOpusFile, li: i32 ) -> u32
	{
		unsafe {
			opus::op_serialno(self._of as *const opus::OggOpusFile, li as libc::c_int) as u32
		}
	}
	
	
	pub fn channel_count (self: &OggOpusFile, li: i32 ) -> i32
	{
		unsafe {
			opus::op_channel_count(self._of as *const opus::OggOpusFile, li as libc::c_int) as i32
		}
	}
	
	
	pub fn raw_total (self: &OggOpusFile, li: i32 ) -> i64
	{
		unsafe {
			opus::op_raw_total(self._of as *const opus::OggOpusFile, li as libc::c_int) as i64
		}
	}
	
	
	pub fn pcm_total (self: &OggOpusFile, li: i32 ) -> i64
	{
		unsafe {
			opus::op_pcm_total(self._of as *const opus::OggOpusFile, li as libc::c_int) as i64
		}
	}
	
	
	pub fn head (self: &OggOpusFile, li: i32) -> Option<OpusHead>
	{
		unsafe {
			match opus::op_head(self._of as *const opus::OggOpusFile, li as libc::c_int) {
				oh if oh != ptr::null() => Some(OpusHead {
					version:           (*oh).version as i32,
					channel_count:     (*oh).channel_count as u8,
					pre_skip:          (*oh).pre_skip as i32,
					input_sample_rate: (*oh).input_sample_rate as u32,
					output_gain:       (*oh).output_gain as i32,
					mapping_family:    (*oh).mapping_family as i32,
					stream_count:      (*oh).stream_count as u8,
					coupled_count:     (*oh).coupled_count as u8,
					mapping:           Vec::from_slice((*oh).mapping.slice(0, (2 * (*oh).coupled_count + ((*oh).stream_count - (*oh).coupled_count)) as usize)),
				}),
				_ => None,
			}
		}
	}
	
	
	pub fn tags (self: &OggOpusFile, li: i32) -> Option<OpusTags>
	{
		unsafe {
			match opus::op_tags(self._of as *const opus::OggOpusFile, li as libc::c_int) {
				ot if ot != ptr::null() => Some(OpusTags {
					user_comments: Vec::from_fn((*ot).comments as usize, |ci| {
						String::from_raw_parts(*((*ot).user_comments as *const *const u8).offset(ci as isize), *((*ot).comment_lengths as *const i32).offset(ci as isize) as usize)
					}),
					vendor: CStr::from_ptr((*ot).vendor as *const u8).to_string_lossy().to_owned(),
				}),
				_ => None,
			}
		}
	}
	
	
	pub fn current_link (self: &OggOpusFile) -> i32
	{
		unsafe {
			opus::op_current_link(self._of as *const opus::OggOpusFile) as i32
		}
	}
	
	pub fn bitrate (self: &OggOpusFile, li: i32) -> i32
	{
		unsafe {
			opus::op_bitrate(self._of as *const opus::OggOpusFile, li as libc::c_int) as i32
		}
	}
	
	pub fn bitrate_instant (self: &OggOpusFile) -> i32
	{
		unsafe {
			opus::op_bitrate_instant(self._of as *const opus::OggOpusFile) as i32
		}
	}
	
	pub fn raw_tell (self: &OggOpusFile) -> i64
	{
		unsafe {
			opus::op_raw_tell(self._of as *const opus::OggOpusFile) as i64
		}
	}
	
	pub fn pcm_tell (self: &OggOpusFile) -> i64
	{
		unsafe {
			opus::op_pcm_tell(self._of as *const opus::OggOpusFile) as i64
		}
	}
	
	pub fn raw_seek (self: &mut OggOpusFile, offset: i64) -> OpusFileResult<()> {
		unsafe {
			match opus::op_raw_seek(self._of, offset) {
				0 => Ok(()),
				e => Err(OpusFileError::from_i32(e as i32).unwrap()),
			}
		}
	}
	
	pub fn pcm_seek (self: &mut OggOpusFile, offset: i64) -> OpusFileResult<()> {
		unsafe {
			match opus::op_raw_seek(self._of, offset) {
				0 => Ok(()),
				e => Err(OpusFileError::from_i32(e as i32).unwrap()),
			}
		}
	}
	
	pub fn read (self: &mut OggOpusFile, pcm: &mut [i16], li: &mut i32) -> OpusFileResult<i32>
	{
		unsafe {
			match opus::op_read(self._of, pcm.as_mut_ptr(), pcm.len() as libc::c_int, li as *mut i32 as *mut libc::c_int) {
				e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
				n          => Ok(n as i32),
			}
		}
	}
	
	pub fn read_float (self: &mut OggOpusFile, pcm: &mut [libc::c_float], li: &mut i32) -> OpusFileResult<i32>
	{
		unsafe {
			match opus::op_read_float(self._of, pcm.as_mut_ptr(), pcm.len() as libc::c_int, li as *mut i32 as *mut libc::c_int) {
				e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
				n          => Ok(n as i32),
			}
		}
	}
	
	pub fn read_stereo (self: &mut OggOpusFile, pcm: &mut [i16]) -> OpusFileResult<i32>
	{
		unsafe {
			match opus::op_read_stereo(self._of, pcm.as_mut_ptr(), pcm.len() as libc::c_int) {
				e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
				n          => Ok(n as i32),
			}
		}
	}
	
	pub fn read_float_stereo (self: &mut OggOpusFile, pcm: &mut [f32]) -> OpusFileResult<i32>
	{
		unsafe {
			match opus::op_read_float_stereo(self._of, pcm.as_mut_ptr(), pcm.len() as libc::c_int) {
				e if e < 0 => Err(OpusFileError::from_i32(e as i32).unwrap()),
				n          => Ok(n as i32),
			}
		}
	}
}


unsafe extern "C" fn read_cb (src: *mut libc::c_void, buf: *mut libc::c_uchar, size: libc::c_int) -> libc::c_int {
	match *(src as *mut DataSource) {
		DataSource::Read(r) => {
            slice::raw::mut_buf_as_slice(buf, size as usize, |s| { r.read(s).unwrap_or(-1) as libc::c_int })
        },
		DataSource::ReadSeek(r) => {
            slice::raw::mut_buf_as_slice(buf, size as usize, |s| { r.read(s).unwrap_or(-1) as libc::c_int })
        },
		_ => -1,
	}
}


unsafe extern "C" fn seek_cb (src: *mut libc::c_void, pos: i64, style: libc::c_int) -> libc::c_int {
	let style = match style {
		libc::SEEK_SET => io::SeekSet,
		libc::SEEK_CUR => io::SeekCur,
		libc::SEEK_END => io::SeekEnd,
		_ => return -1,
	};
	
	match *(src as *mut DataSource) {
		OwnedReadSeeker(ref mut s) => if s.seek(pos, style).is_ok() { 0 } else { -1 },
		_ => -1,
	}
}


unsafe extern "C" fn tell_cb (src: *mut libc::c_void) -> i64 {
	match *(src as *mut DataSource) {
		OwnedReadSeeker(ref mut s) => s.tell().unwrap_or(-1) as i64,
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


