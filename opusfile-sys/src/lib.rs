

extern crate libc;

use libc::*;


pub const OP_FALSE         : c_int = -1;
pub const OP_EOF           : c_int = -2;
pub const OP_HOLE          : c_int = -3;
pub const OP_EREAD         : c_int = -128;
pub const OP_EFAULT        : c_int = -129;
pub const OP_EIMPL         : c_int = -130;
pub const OP_EINVAL        : c_int = -131;
pub const OP_ENOTFORMAT    : c_int = -132;
pub const OP_EBADHEADER    : c_int = -133;
pub const OP_EVERSION      : c_int = -134;
pub const OP_ENOTAUDIO     : c_int = -135;
pub const OP_EBADPACKET    : c_int = -136;
pub const OP_EBADLINK      : c_int = -137;
pub const OP_ENOSEEK       : c_int = -138;
pub const OP_EBADTIMESTAMP : c_int = -139;


#[repr(C)]
pub struct OpusHead
{
	pub version           : c_int,
	pub channel_count     : c_int,
	pub pre_skip          : c_uint,
	pub input_sample_rate : u32,
	pub output_gain       : c_int,
	pub mapping_family    : c_int,
	pub stream_count      : c_int,
	pub coupled_count     : c_int,
	pub mapping           : [c_uchar; 255],
}


#[repr(C)]
pub struct OpusTags
{
  pub user_comments   : *const *const c_char,
  pub comment_lengths : *const c_int,
  pub comments        : c_int,
  pub vendor          : *const c_char,
}


#[allow(non_camel_case_types)]
pub type op_read_func  = unsafe extern "C" fn (*mut c_void, *mut c_uchar, c_int) -> c_int;
#[allow(non_camel_case_types)]
pub type op_seek_func  = unsafe extern "C" fn (*mut c_void, i64, c_int) -> c_int;
#[allow(non_camel_case_types)]
pub type op_tell_func  = unsafe extern "C" fn (*mut c_void) -> i64;
#[allow(non_camel_case_types)]
pub type op_close_func = unsafe extern "C" fn (*mut c_void) -> c_int;


#[repr(C)]
pub struct OpusFileCallbacks
{
	pub read  : op_read_func,
	pub seek  : Option<op_seek_func>,
	pub tell  : Option<op_tell_func>,
	pub close : Option<op_close_func>,
}


pub type OggOpusFile = c_void;


#[link(name = "opusfile")]
extern "C"
{
	pub fn op_open_memory (_data: *const c_uchar, _size: size_t, _error: *mut c_int) -> *mut OggOpusFile;
	pub fn op_open_callbacks (
		_source: *mut c_void,
		_cb: *const OpusFileCallbacks,
		_initial_data: *const c_uchar,
		_initial_bytes: size_t,
		_error: *mut c_int
	) -> *mut OggOpusFile;
	pub fn op_free        (_of: *mut OggOpusFile);
	
	pub fn op_seekable        (_of: *const OggOpusFile)             -> c_int;
	pub fn op_link_count      (_of: *const OggOpusFile)             -> c_int;
	pub fn op_serialno        (_of: *const OggOpusFile, _li: c_int) -> u32;
	pub fn op_channel_count   (_of: *const OggOpusFile, _li: c_int) -> c_int;
	pub fn op_raw_total       (_of: *const OggOpusFile, _li: c_int) -> i64;
	pub fn op_pcm_total       (_of: *const OggOpusFile, _li: c_int) -> i64;
	pub fn op_head            (_of: *const OggOpusFile, _li: c_int) -> *const OpusHead;
	pub fn op_tags            (_of: *const OggOpusFile, _li: c_int) -> *const OpusTags;
	pub fn op_current_link    (_of: *const OggOpusFile)             -> c_int;
	pub fn op_bitrate         (_of: *const OggOpusFile, _li: c_int) -> i32;
	pub fn op_bitrate_instant (_of: *const OggOpusFile)             -> i32;
	
	pub fn op_raw_tell (_of: *const OggOpusFile)                  -> i64;
	pub fn op_pcm_tell (_of: *const OggOpusFile)                  -> i64;
	pub fn op_raw_seek (_of: *mut OggOpusFile, _byte_offset: i64) -> c_int;
	pub fn op_pcm_seek (_of: *mut OggOpusFile, _pcm_offset: i64)  -> c_int;
	
	pub fn op_read              (_of: *mut OggOpusFile, _pcm: *mut i16, _buf_size: c_int, _li: *mut c_int)     -> c_int;
	pub fn op_read_float        (_of: *mut OggOpusFile, _pcm: *mut c_float, _buf_size: c_int, _li: *mut c_int) -> c_int;
	pub fn op_read_stereo       (_of: *mut OggOpusFile, _pcm: *mut i16, _buf_size: c_int)                      -> c_int;
	pub fn op_read_float_stereo (_of: *mut OggOpusFile, _pcm: *mut c_float, _buf_size: c_int)                  -> c_int;
}


