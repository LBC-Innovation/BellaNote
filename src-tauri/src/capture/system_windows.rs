//! WASAPI loopback of the default playback device (“what you hear”).
//! Microphone stays on `cpal`; both feed the shared `SystemMicMixer`.

use crate::capture::mic::StreamingResampler;
use crate::capture::mixer::SystemMicMixer;
use crate::capture::wav::PLAYBACK_RATE;
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

struct StereoRateConverter {
    in_rate: u32,
    left: StreamingResampler,
    right: StreamingResampler,
}

impl StereoRateConverter {
    fn new(in_rate: u32) -> Self {
        Self {
            in_rate,
            left: StreamingResampler::new(in_rate, PLAYBACK_RATE),
            right: StreamingResampler::new(in_rate, PLAYBACK_RATE),
        }
    }

    fn push(&mut self, frames: &[[f32; 2]], in_rate: u32) -> Vec<[f32; 2]> {
        if in_rate != self.in_rate && in_rate > 0 {
            *self = Self::new(in_rate);
        }
        if frames.is_empty() {
            return Vec::new();
        }
        if in_rate == PLAYBACK_RATE {
            return frames.to_vec();
        }
        let mut ls = Vec::with_capacity(frames.len());
        let mut rs = Vec::with_capacity(frames.len());
        for &[l, r] in frames {
            ls.push(l);
            rs.push(r);
        }
        let lo = self.left.push_mono(&ls);
        let ro = self.right.push_mono(&rs);
        let n = lo.len().min(ro.len());
        lo.into_iter()
            .zip(ro)
            .take(n)
            .map(|(l, r)| [l, r])
            .collect()
    }
}

pub struct SystemAudioSession {
    mixer: Arc<SystemMicMixer>,
    stop: Arc<AtomicBool>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl SystemAudioSession {
    pub async fn new(mixer: Arc<SystemMicMixer>) -> Result<Self> {
        Ok(Self {
            mixer,
            stop: Arc::new(AtomicBool::new(false)),
            join: Mutex::new(None),
        })
    }

    pub fn start(&self) -> Result<()> {
        self.stop.store(false, Ordering::Release);
        let (tx, rx) = mpsc::channel();
        let mixer = self.mixer.clone();
        let stop = self.stop.clone();
        let handle = std::thread::Builder::new()
            .name("wasapi-loopback".into())
            .spawn(move || run_loopback(mixer, stop, tx))
            .context("failed to start WASAPI loopback thread")?;
        *self.join.lock().expect("wasapi join mutex poisoned") = Some(handle);
        rx.recv()
            .map_err(|_| anyhow::anyhow!("WASAPI loopback thread exited before it was ready."))?
            .map_err(|e| anyhow::anyhow!("WASAPI loopback: {e}"))?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.join.lock().expect("wasapi join mutex poisoned").take() {
            let _ = handle.join();
        }
        Ok(())
    }
}

fn run_loopback(
    mixer: Arc<SystemMicMixer>,
    stop: Arc<AtomicBool>,
    ready: mpsc::Sender<Result<(), String>>,
) {
    if let Err(e) = capture_loop(mixer, stop, &ready) {
        let _ = ready.send(Err(e.to_string()));
        log::warn!("WASAPI loopback ended: {e}");
    }
}

fn capture_loop(
    mixer: Arc<SystemMicMixer>,
    stop: Arc<AtomicBool>,
    ready: &mpsc::Sender<Result<(), String>>,
) -> Result<()> {
    use std::collections::VecDeque;
    use wasapi::{initialize_mta, DeviceEnumerator, Direction, SampleType, StreamMode, WaveFormat};

    initialize_mta()
        .ok()
        .map_err(|e| anyhow::anyhow!("Could not initialize Windows audio: {e}"))?;

    let enumerator = DeviceEnumerator::new().map_err(|e| anyhow::anyhow!("{e}"))?;
    let device = enumerator
        .get_default_device(&Direction::Render)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("No default playback device for system audio")?;
    let mut audio_client = device
        .get_iaudioclient()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let desired = WaveFormat::new(32, 32, &SampleType::Float, PLAYBACK_RATE as usize, 2, None);
    let buffer_duration_hns = audio_client
        .get_device_period()
        .map(|(_, min_time)| min_time)
        .unwrap_or(200_000);
    let mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns,
    };
    // Render device + Capture direction sets AUDCLNT_STREAMFLAGS_LOOPBACK
    // (“what you hear”), matching macOS ScreenCaptureKit display audio.
    audio_client
        .initialize_client(&desired, &Direction::Capture, &mode)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Could not start WASAPI loopback on the default playback device")?;

    let h_event = audio_client
        .set_get_eventhandle()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let capture_client = audio_client
        .get_audiocaptureclient()
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    audio_client
        .start_stream()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let blockalign = desired.get_blockalign() as usize;
    let channels = desired.get_nchannels() as usize;
    let bits = desired.get_bitspersample();
    let is_float = matches!(desired.get_subformat(), Ok(SampleType::Float));
    let rate = desired.get_samplespersec();
    let mut sample_queue: VecDeque<u8> = VecDeque::with_capacity(blockalign.max(1) * 8_192);
    let mut converter = StereoRateConverter::new(rate);

    let _ = ready.send(Ok(()));

    while !stop.load(Ordering::Acquire) {
        if h_event.wait_for_event(80).is_err() {
            continue;
        }
        loop {
            let new_frames = capture_client
                .get_next_packet_size()
                .ok()
                .flatten()
                .unwrap_or(0);
            if new_frames == 0 {
                break;
            }
            if let Err(e) = capture_client.read_from_device_to_deque(&mut sample_queue) {
                log::warn!("WASAPI loopback read failed: {e}");
                break;
            }
        }
        if blockalign == 0 {
            continue;
        }
        let complete = sample_queue.len() / blockalign * blockalign;
        if complete == 0 {
            continue;
        }
        let mut bytes = Vec::with_capacity(complete);
        for _ in 0..complete {
            if let Some(b) = sample_queue.pop_front() {
                bytes.push(b);
            }
        }
        let frames =
            crate::capture::mixer::pcm_interleaved_to_stereo_f32(&bytes, channels, bits, is_float);
        let resampled = converter.push(&frames, rate);
        if !resampled.is_empty() {
            mixer.push_system_stereo(&resampled);
        }
    }

    let _ = audio_client.stop_stream();
    Ok(())
}
