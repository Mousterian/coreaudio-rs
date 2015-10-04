pub extern crate coreaudio_sys as bindings;

use bindings::core_audio as ca;
use audio_unit::stream_format::StreamFormat;
use error::Error;
use std::mem;
use libc;

pub fn open_audio_file(path: &str) -> Result<ca::AudioFileID, Error> {
    unsafe {
        let url_ref = try!( match ca::CFURLCreateFromFileSystemRepresentation(ca::kCFAllocatorDefault,
                                                                                path.as_ptr(),
                                                                                path.len() as i64,
                                                                                0 as ca::Boolean) {
            url_ref if url_ref.is_null()    => Err(Error::Unspecified),
            url_ref                         => Ok(url_ref),
        } );

        let mut audio_file_id: ca::AudioFileID = mem::uninitialized();
        try!( Error::from_os_status(ca::AudioFileOpenURL(url_ref,
                                    ca::kAudioFileReadPermission as i8,
                                    0,
                                    &mut audio_file_id as *mut ca::AudioFileID)) );

        ca::CFRelease(url_ref as ca::CFTypeRef);
        Ok(audio_file_id)
    }
}

pub fn get_data_format(audio_file_id: ca::AudioFileID) -> Result<StreamFormat, Error> {
    unsafe {
        // get the number of channels of the file
        let mut file_format : ca::AudioStreamBasicDescription = mem::uninitialized();
        let mut property_size = mem::size_of::<ca::AudioStreamBasicDescription>() as u32;
        try!( Error::from_os_status(ca::AudioFileGetProperty(   audio_file_id,
                                                                ca::kAudioFilePropertyDataFormat,
                                                                &mut property_size as *mut ca::UInt32,
                                                                &mut file_format as *mut _ as *mut libc::c_void)  ));
        Ok(StreamFormat::from_asbd(file_format))
    }
}