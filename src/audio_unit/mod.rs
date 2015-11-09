//! This module is an attempt to provide a friendly, rust-esque interface to Apple's Audio Unit API.
//!
//! Learn more about the Audio Unit API [here](https://developer.apple.com/library/mac/documentation/MusicAudio/Conceptual/AudioUnitProgrammingGuide/Introduction/Introduction.html#//apple_ref/doc/uid/TP40003278-CH1-SW2)
//! and [here](https://developer.apple.com/library/mac/documentation/MusicAudio/Conceptual/AudioUnitProgrammingGuide/TheAudioUnit/TheAudioUnit.html).

use bindings::core_audio as au;
use error::{Error, AudioUnitError};
use libc;
use self::stream_format::StreamFormat;
use std::mem;
use std::ptr;

pub mod audio_format;
pub mod stream_format;
pub mod graph;

/// The input and output **Scope**s.
///
/// More info [here](https://developer.apple.com/library/ios/documentation/AudioUnit/Reference/AudioUnitPropertiesReference/index.html#//apple_ref/doc/constant_group/Audio_Unit_Scopes)
/// and [here](https://developer.apple.com/library/mac/documentation/MusicAudio/Conceptual/AudioUnitProgrammingGuide/TheAudioUnit/TheAudioUnit.html).
#[derive(Copy, Clone, Debug)]
pub enum Scope {
    Global		= 0,
    Input		= 1,
    Output		= 2,
    Group		= 3,
    Part		= 4,
    Note		= 5,
    Layer		= 6,
    LayerItem	= 7
}

/// Represents the **Input** and **Output** **Element**s.
///
/// These are used when specifying which **Element** we're setting the properties of.
#[derive(Copy, Clone, Debug)]
pub enum Element {
    Output = 0,
    Input  = 1,
}

/// Represents the different kinds of Audio Units that are available.
#[derive(Copy, Clone, Debug)]
pub enum Type {
    Output          = 1635086197,
    MusicDevice     = 1635085685,
    MusicEffect     = 1635085670,
    FormatConverter = 1635083875,
    Effect          = 1635083896,
    Mixer           = 1635085688,
    Panner          = 1635086446,
    Generator       = 1635084142,
    OfflineEffect   = 1635086188,
    MidiProcessor   = 1635085673,
}

/// Represents the different audio unit sub types.
#[derive(Copy, Clone, Debug)]
pub enum SubType {
    GenericOutput        = 1734700658,
    HalOutput            = 1634230636,
    DefaultOutput        = 1684366880,
    SystemOutput         = 1937339168,
    VoiceProcessingIO    = 1987078511,
    DLSSynth             = 1684828960,
    Sampler              = 1935764848,
    MIDISynth            = 1836284270,
    AUConverter          = 1668247158,
    Varispeed            = 1986097769,
    DeferredRenderer     = 1684366962,
    Splitter             = 1936747636,
    Merger               = 1835364967,
    NewTimePitch         = 1853191280,
    AUiPodTimeOther      = 1768977519,
    RoundTripAAC         = 1918984547,
    PeakLimiter          = 1819112562,
    DynamicsProcessor    = 1684237680,
    LowPassFilter        = 1819304307,
    HighPassFilter       = 1752195443,
    BandPassFilter       = 1651532147,
    HighShelfFilter      = 1752393830,
    LowShelfFilter       = 1819502694,
    ParametricEQ         = 1886217585,
    Distortion           = 1684632436,
    Delay                = 1684368505,
    SampleDelay          = 1935961209,
    GraphicEQ            = 1735550321,
    MultiBandCompressor  = 1835232624,
    MatrixReverb         = 1836213622,
    Pitch                = 1953329268,
    AUFilter             = 1718185076,
    NetSend              = 1853058660,
    RogerBeep            = 1919903602,
    NBandEQ              = 1851942257,
    MultiChannelMixer    = 1835232632,
    MatrixMixer          = 1836608888,
    SpatialMixer         = 862217581,
    StereoMixer          = 1936554098,
    Mixer3D              = 862219640,
    SphericalHeadPanner  = 1936746610,
    VectorPanner         = 1986158963,
    SoundFieldPanner     = 1634558569,
    HRTFPanner           = 1752331366,
    NetReceive           = 1852990326,
    ScheduledSoundPlayer = 1936945260,
    AudioFilePlayer      = 1634103404,
}

/// The number of frames available in some buffer.
pub type NumFrames = usize;

/// A type representing a render callback (aka "Input Procedure")
/// If set on an AudioUnit, this will be called every time the AudioUnit requests audio.
/// The first arg is [frames[channels]]; the second is the number of frames to render.
pub type RenderCallback = FnMut(&mut[&mut[f32]], NumFrames) -> Result<(), String>;

#[derive(Copy, Clone, Debug)]
/// Apple audio unit manufacturer ID.
pub enum Manufacturer {
    Apple = 1634758764
}

/// A rust representation of the au::AudioUnit, including a pointer to the current rendering callback.
///
/// Find the original Audio Unit Programming Guide [here](https://developer.apple.com/library/mac/documentation/MusicAudio/Conceptual/AudioUnitProgrammingGuide/TheAudioUnit/TheAudioUnit.html).
pub struct AudioUnit {
    instance: au::AudioUnit,
    callback: Option<*mut libc::c_void>,
    owned: bool
}

macro_rules! try_os_status {
    ($expr:expr) => (try!(Error::from_os_status($expr)))
}

impl AudioUnit {

    /// Construct a new AudioUnit.
    pub fn new(au_type: Type, sub_type: SubType) -> Result<AudioUnit, Error> {

        // A description of the audio unit we desire.
        let desc = au::AudioComponentDescription {
            componentType         : au_type as libc::c_uint,
            componentSubType      : sub_type as libc::c_uint,
            componentManufacturer : au::kAudioUnitManufacturer_Apple,
            componentFlags        : 0,
            componentFlagsMask    : 0,
        };

        unsafe {
            // Find the default audio unit for the description.
            let component = match au::AudioComponentFindNext(ptr::null_mut(), &desc as *const _) {
                component if component.is_null() => return Err(Error::NoMatchingDefaultAudioUnitFound),
                component                        => component,
            };

            // Get an instance of the default audio unit using the component.
            let mut instance: au::AudioUnit = mem::uninitialized();

            try_os_status!(au::AudioComponentInstanceNew(component, &mut instance as *mut au::AudioUnit));
            // Initialise the audio unit!
            try_os_status!(au::AudioUnitInitialize(instance));
            Ok(AudioUnit {
                instance: instance,
                callback: None,
                owned: true
            })
        }
    }

    fn free_render_callback(&self) {
        if let Some(callback) = self.callback {
            // Here, we transfer ownership of the callback back to the current scope so that it
            // is dropped and cleaned up. Without this line, we would leak the Boxed callback.
            let _: Box<Box<RenderCallback>> = unsafe { Box::from_raw(callback as *mut Box<RenderCallback>) };
        }
    }

    /// Pass a render callback (aka "Input Procedure") to the audio unit.
    pub fn render_callback(&mut self, f: Option<Box<RenderCallback>>) -> Result<(), Error>
    {
        // Setup render callback. Notice that we relinquish ownership of the Callback
        // here so that it can be used as the C render callback via a void pointer.
        // We do however store the *mut so that we can convert back to a
        // Box<Box<RenderCallback>> within our AudioUnit's Drop implementation
        // (otherwise it would leak). The double-boxing is due to incompleteness with
        // Rust's FnMut implemetation and is necessary to be able to convert to the
        // correct pointer size.
        let callback_ptr = match f {
            Some(x) => Box::into_raw(Box::new(x)) as *mut libc::c_void,
            _ => ptr::null_mut()
        };
        let render_callback = au::AURenderCallbackStruct {
            inputProc: Some(input_proc),
            inputProcRefCon: callback_ptr
        };

        unsafe {
            try_os_status!(au::AudioUnitSetProperty(
                self.instance,
                au::kAudioUnitProperty_SetRenderCallback,
                Scope::Input as libc::c_uint,
                Element::Output as libc::c_uint,
                &render_callback as *const _ as *const libc::c_void,
                mem::size_of::<au::AURenderCallbackStruct>() as u32));
        }

        self.free_render_callback();
        self.callback = if !callback_ptr.is_null() { Some(callback_ptr) } else { None };
        Ok(())
    }

    /// Start the audio unit.
    pub fn start(&self) -> Result<(), Error> {
        unsafe { try_os_status!(au::AudioOutputUnitStart(self.instance)); }
        Ok(())
    }

    /// Stop the audio unit.
    pub fn stop(&self) -> Result<(), Error> {
        unsafe { try_os_status!(au::AudioOutputUnitStop(self.instance)); }
        Ok(())
    }

    /// Set the audio unit's sample rate.
    pub fn set_sample_rate(&self, sample_rate: f64) -> Result<(), Error> {
        unsafe {
            try_os_status!(au::AudioUnitSetProperty(
                self.instance,
                au::kAudioUnitProperty_SampleRate,
                au::kAudioUnitScope_Input,
                0,
                &sample_rate as *const _ as *const libc::c_void,
                mem::size_of::<f64>() as u32));
            Ok(())
        }
    }

    /// Get the audio unit's sample rate.
    pub fn sample_rate(&self) -> Result<f64, Error> {
        unsafe {
            let mut sample_rate: f64 = 0.0;
            let mut size: u32 = mem::size_of::<f64>() as u32;
            try_os_status!(au::AudioUnitGetProperty(
                self.instance,
                au::kAudioUnitProperty_SampleRate,
                au::kAudioUnitScope_Input,
                0,
                &mut sample_rate as *mut _ as *mut libc::c_void,
                &mut size as *mut _));
            Ok(sample_rate)
        }
    }

    /// Sets the current Stream Format for the AudioUnit.
    pub fn set_stream_format(&self,
                            scope: Scope,
                            element: Element,
                            stream_format: StreamFormat) -> Result<(), Error> {
        unsafe {
            let mut asbd = stream_format.to_asbd();
            let size = mem::size_of::<au::AudioStreamBasicDescription>() as au::UInt32;
            try_os_status!(au::AudioUnitSetProperty(
                self.instance,
                au::kAudioUnitProperty_StreamFormat,
                scope as libc::c_uint,
                element as libc::c_uint,
                &mut asbd as *mut _ as *mut libc::c_void,
                size));
            Ok(())
        }
    }

    pub fn from_graph(instance: au::AudioUnit) -> AudioUnit {
        AudioUnit { instance: instance, callback: None, owned: false }
    }

    /// Return the current Stream Format for the AudioUnit.
    pub fn stream_format(&self, scope: Scope, element: Element) -> Result<StreamFormat, Error> {
        unsafe {
            let mut asbd: au::AudioStreamBasicDescription = mem::uninitialized();
            let mut size = ::std::mem::size_of::<au::AudioStreamBasicDescription>() as u32;
            try_os_status!(au::AudioUnitGetProperty(
                self.instance,
                au::kAudioUnitProperty_StreamFormat,
                scope as libc::c_uint,
                element as libc::c_uint,
                &mut asbd as *mut _ as *mut libc::c_void,
                &mut size as *mut au::UInt32));
            Ok(StreamFormat::from_asbd(asbd))
        }
    }

}

impl Drop for AudioUnit {
    fn drop(&mut self) {
        unsafe {
            use error;
            use std::error::Error;
            // if this AudioUnit was returned from an AUGraph, the graph will take responsibility for calling these when
            // the graph is stopped.
            if self.owned {
                if let Err(err) = self .stop() {
                    panic!("{:?}", err.description());
                }
                if let Err(err) = error::Error::from_os_status(au::AudioUnitUninitialize(self .instance)) {
                    panic!("{:?}", err.description());
                }
            }
            self.free_render_callback();
        }
    }
}

/// Callback procedure that will be called each time our audio_unit requests audio.
extern "C" fn input_proc(in_ref_con: *mut libc::c_void,
                         _io_action_flags: *mut au::AudioUnitRenderActionFlags,
                         _in_time_stamp: *const au::AudioTimeStamp,
                         _in_bus_number: au::UInt32,
                         in_number_frames: au::UInt32,
                         io_data: *mut au::AudioBufferList) -> au::OSStatus {
    let callback: *mut Box<RenderCallback> = in_ref_con as *mut _;
    unsafe {
        let num_channels = (*io_data).mNumberBuffers as usize;

        // FIXME: We shouldn't need a Vec for this, it should probably be something like
        // `&[&mut [f32]]` instead.
        let mut channels: Vec<&mut [f32]> =
            (0..num_channels)
                .map(|i| {
                    let slice_ptr = (*io_data).mBuffers[i].mData as *mut libc::c_float;
                    // TODO: the size of this buffer needs to be calculated properly based on the stream format.
                    // Currently this won't be correct in at least this case:
                    /*
                    stream_format::StreamFormat {
                        sample_rate: 44100.0,
                        audio_format: audio_format::AudioFormat::LinearPCM(Some(audio_format::LinearPCMFlag::IsFloat)),
                        bytes_per_packet: 2 * 4,
                        frames_per_packet: 1,
                        bytes_per_frame: 2 * 4,
                        channels_per_frame: 2,
                        bits_per_channel: 32
                    }
                     */
                    ::std::slice::from_raw_parts_mut(slice_ptr, in_number_frames as usize)
                })
                .collect();

        match (*callback)(&mut channels[..], in_number_frames as usize) {
            Ok(()) => 0 as au::OSStatus,
            Err(description) => {
                use std::io::Write;
                writeln!(::std::io::stderr(), "{:?}", description).unwrap();
                AudioUnitError::NoConnection as au::OSStatus
            },
        }
    }
}
