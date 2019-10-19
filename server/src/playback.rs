use crate::queue::{Queue, QueueCallback};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use parking_lot::Mutex;
use rodio::Sample;
use std::sync::Arc;
use std::thread;

pub fn establish<C>(queue_callback: C) -> Arc<Mutex<Queue<f32, C>>>
where
    C: QueueCallback<f32> + Send + 'static,
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

    let volume = 0.5_f32;
    let queue = Arc::new(Mutex::new(Queue::new(volume, format, queue_callback)));
    let queue_for_audio_thread = Arc::clone(&queue);

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
                audio_callback(stream_data, &queue_for_audio_thread)
            });
        })
        .expect("error spawning audio thread");

    queue
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
