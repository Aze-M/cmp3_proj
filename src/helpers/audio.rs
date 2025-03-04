use anyhow::Result;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, Host, StreamConfig, StreamError,
};
use std::{
    fs::File,
    path::Path,
    sync::{Arc, LazyLock as Lazy, Mutex, OnceLock}
};
use symphonia::{
    core::{
        audio::{AudioBufferRef, Signal},
        codecs::DecoderOptions,
        formats::{FormatOptions, FormatReader},
        io::MediaSourceStream,
        meta::MetadataOptions,
    },
    default,
};

static AUDIO_BUFFER: Lazy<Arc<Mutex<Vec<f32>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));
static mut VOLUME: f32 = 1.0;

pub struct AudioEngine {
    pub host: Host,
    output: Option<Device>,
    config: Option<StreamConfig>
}

//less yellow lines :)
#[allow(unused)]
impl AudioEngine {
    fn new() -> Self {
        AudioEngine {
            host: cpal::default_host(),
            output: None,
            config: None,
        }
    }

    pub fn get_instance() -> &'static Mutex<AudioEngine> {
        static INSTANCE: OnceLock<Mutex<AudioEngine>> = OnceLock::new();

        println!("Audio instance grabbed.");

        return INSTANCE.get_or_init(|| Mutex::new(AudioEngine::new()));
    }

    //initializes the audio engine.
    pub fn init(&mut self) -> Result<()> {
        self.output = self.host.default_output_device();

        if (self.output.is_some()) {
            let conf_temp = self.output.as_ref().unwrap().default_output_config()?;
            let supported_format = conf_temp.sample_format();

            self.config = Some(conf_temp.into());

            /*
            let stream = self.output.as_ref().unwrap().build_output_stream(
                &self.config.as_ref().unwrap(),
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {},
                stream_error,
                None,
            );
            */
        }

        println!("Engine Initialized.");
        println!("Host Type: {:?}", self.host.id());

        return Ok(());
    }

    pub fn decode_and_play(&self, path: &Path) -> Result<()> {
        println!("Opening {:?}", path);

        let file = File::open(path)?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        println!("Creating codex registry.");
        let codexreg = default::get_codecs();

        println!("Creating Probe.");
        let probe = default::get_probe()
            .format(
                &Default::default(),
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .expect("Failed to probe file.");

        println!("Extracting format reader");
        let mut formatreader = probe.format;

        println!("Getting first track from reader.");
        let track = formatreader.tracks().first();

        if track.is_none() {
            return Err(anyhow::Error::msg("Failed to find track in file."));
        }

        let track = track.unwrap();

        println!("Creating decoder.");
        let mut decoder = codexreg.make(&track.codec_params, &DecoderOptions::default())?;

        println!("Cloning buffer for modification.");
        let buffer_clone = Arc::clone(&AUDIO_BUFFER);

        //spawns a thread to read the entire file and stream the contents to the audio engine
        println!("Beginning file read.");

        //used for locking ot ensure full audio plays.
        let mut totalpacks = 0;

        //thread::spawn(move || {
        while let Ok(packet) = formatreader.next_packet() {
            totalpacks += 1;
            if let Ok(decoded) = decoder.decode(&packet) {
                if let AudioBufferRef::F32(packet_buffer) = decoded {
                    let mut buf_lock = buffer_clone.lock().unwrap();
                    buf_lock.extend_from_slice(packet_buffer.chan(0));
                }
            }
        }
        println!("Total Packets read: {:?}", totalpacks);
        //});

        let buffer_clone = Arc::clone(&AUDIO_BUFFER);
        let stream = self.output.as_ref().unwrap().build_output_stream(
            &self.config.as_ref().unwrap(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                println!("Callback! {:?}", data.len());
                let mut bfr = buffer_clone.lock().unwrap();
                for sample in data {
                    if !bfr.is_empty() {
                        unsafe {
                            *sample = bfr.remove(0) * VOLUME;
                        }
                    } else {
                        *sample = 0.0;
                    }
                }
            },
            stream_error,
            None,
        )?;

        stream.play();

        let buffer_clone = Arc::clone( &AUDIO_BUFFER); 
        while !buffer_clone.lock().unwrap().is_empty() {

        }

        return Ok(());
    }

    pub fn decode(&self, file: File) -> Result<Box<dyn FormatReader>> {
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let codexreg = default::get_codecs();

        let probe = default::get_probe()
            .format(
                &Default::default(),
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .expect("Failed to probe file.");

        let mut formatreader = probe.format;

        return Ok(formatreader);
    }

    pub fn play(&self, reader: &mut Box<dyn FormatReader>) -> Result<()> {
        let codexreg = default::get_codecs();
        let track = reader
            .tracks()
            .first()
            .expect("Could not find track in file!");

        let mut decoder = codexreg.make(&track.codec_params, &DecoderOptions::default())?;

        let buffer_clone = Arc::clone(&AUDIO_BUFFER);
        //locks thread to decode entire file, should really be using thread but that has it's own issues, change later
        while let Ok(packet) = reader.next_packet() {
            if let Ok(decoded) = decoder.decode(&packet) {
                if let AudioBufferRef::F32(packet_buffer) = decoded {
                    let mut buf_lock = buffer_clone.lock().unwrap();
                    buf_lock.extend_from_slice(packet_buffer.chan(0));
                }
            }
        }

        return Ok(());
    }
}

#[allow(unused)]
pub fn test() {
    println!("Audio Module ran Test successfully!")
}

// has to stay out of AE class to ensure availability globally, singleton ensures this is never used anyways.
/*fn write<T: Sample>(data: &mut [f32], _: &cpal::OutputCallbackInfo) {
    println!("callback happened");
    let mut buffer_lock = AUDIO_BUFFER.lock().unwrap();

    for sample in data.iter_mut() {
        if !buffer_lock.is_empty() {
            //unsafe because of volume multiplier, volume should only ever be modified by locking code so it's aight.
            unsafe {
                *sample = buffer_lock.remove(0) * VOLUME;
            }
        } else {
            *sample = 0.0;
        }
    }
}
*/

fn stream_error(err: StreamError) {
    println!("Output stream errored: {:?}", err);
    return;
}
