use crate::queue::{Queue, QueueEventSink};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use cpal::{Format, SampleFormat};
use parking_lot::Mutex;
use rodio::Sample;
use std::sync::Arc;
use std::thread;

pub struct PlaybackSource<E> {
    pub queue: Queue<f32, E>,
    pub controls: PlaybackControls,
}

impl<E: QueueEventSink> PlaybackSource<E> {
    fn new(volume: f32, audio_format: Format, event_sink: E) -> Self {
        assert_eq!(
            audio_format.data_type,
            SampleFormat::F32,
            "Only f32 samples supported"
        );
        PlaybackSource {
            queue: Queue::new(audio_format, event_sink),
            controls: PlaybackControls {
                paused: false,
                muted: false,
                volume,
            },
        }
    }
}

impl<E: QueueEventSink> Iterator for PlaybackSource<E> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.controls.paused {
            Some(Sample::zero_value())
        } else if self.controls.muted {
            self.queue.next();
            Some(Sample::zero_value())
        } else {
            self.queue
                .next()
                .map(|sample| sample.amplify(self.controls.volume))
        }
    }
}

pub struct PlaybackControls {
    pub paused: bool,
    pub muted: bool,
    pub volume: f32,
}

pub fn establish<E>(event_sink: E) -> Arc<Mutex<PlaybackSource<E>>>
where
    E: QueueEventSink + Send + 'static,
{
    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let device = host
        .default_output_device()
        .expect("no audio output device found");
    let mut supported_formats_range = device
        .supported_output_formats()
        .expect("error while querying formats");
    let format = supported_formats_range
        .next()
        .expect("no supported format?!")
        .with_max_sample_rate();
    let stream_id = event_loop
        .build_output_stream(&device, &format)
        .expect("error building audio stream");
    event_loop
        .play_stream(stream_id)
        .expect("failed to play audio stream");

    let volume = 0.5f32;
    let source = Arc::new(Mutex::new(PlaybackSource::new(volume, format, event_sink)));
    let source_for_audio_thread = Arc::clone(&source);

    thread::Builder::new()
        .name("audio thread".to_string())
        .spawn(move || {
            event_loop.run(move |stream_id, stream_result| {
                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        log::error!("error in audio stream {:?}: {}", stream_id, err);
                        return;
                    }
                };
                audio_callback(stream_data, &source_for_audio_thread)
            });
        })
        .expect("error spawning audio thread");

    source
}

fn audio_callback(stream_data: cpal::StreamData, audio_source: &Mutex<impl Iterator<Item = f32>>) {
    match stream_data {
        cpal::StreamData::Output {
            buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
        } => {
            let mut audio_source = audio_source.lock();
            for slot in buffer.iter_mut() {
                *slot = audio_source.next().unwrap_or_else(Sample::zero_value);
            }
        }
        _ => panic!("we only support playing f32 samples"),
    }
}
