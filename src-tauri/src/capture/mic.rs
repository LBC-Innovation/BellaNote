//! Default microphone capture via `cpal`. Does not require screen-recording permission.
//! The `cpal::Stream` is created and dropped on a dedicated thread (it is not `Send`).

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub type SampleSink = Arc<dyn Fn(&[f32]) + Send + Sync>;

/// OS-reported name of the default input device.
pub fn default_input_device_name() -> Result<String, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No default input device.".to_string())?;
    device.name().map_err(|e| e.to_string())
}

pub(crate) struct StreamingResampler {
    in_rate: u32,
    out_rate: u32,
    pending: Vec<f32>,
    next_out: u64,
    input_base: u64,
}

impl StreamingResampler {
    pub(crate) fn new(in_rate: u32, out_rate: u32) -> Self {
        Self {
            in_rate,
            out_rate,
            pending: Vec::new(),
            next_out: 0,
            input_base: 0,
        }
    }

    pub(crate) fn push_mono(&mut self, mono: &[f32]) -> Vec<f32> {
        self.pending.extend_from_slice(mono);
        if self.in_rate == 0 || self.out_rate == 0 {
            return Vec::new();
        }
        if self.in_rate == self.out_rate {
            return std::mem::take(&mut self.pending);
        }
        if self.in_rate == 48_000 && self.out_rate == 16_000 {
            return self.drain_48k_to_16k();
        }
        self.drain_linear()
    }

    fn drain_48k_to_16k(&mut self) -> Vec<f32> {
        let mut out = Vec::new();
        let mut i = 0usize;
        while i + 3 <= self.pending.len() {
            let v = (self.pending[i] + self.pending[i + 1] + self.pending[i + 2]) / 3.0;
            out.push(v);
            i += 3;
        }
        if i > 0 {
            self.pending.drain(..i);
            self.input_base += i as u64;
        }
        out
    }

    fn drain_linear(&mut self) -> Vec<f32> {
        let ratio = self.in_rate as f64 / self.out_rate as f64;
        let mut out = Vec::new();
        loop {
            let src_pos = (self.next_out as f64) * ratio;
            let idx_global = src_pos.floor() as u64;
            let idx_rel = (idx_global - self.input_base) as usize;
            if idx_rel + 1 >= self.pending.len() {
                break;
            }
            let frac = (src_pos - idx_global as f64) as f32;
            let s0 = self.pending[idx_rel];
            let s1 = self.pending[idx_rel + 1];
            out.push(s0 + (s1 - s0) * frac);
            self.next_out += 1;
        }
        let want_start = (self.next_out as f64 * ratio).floor() as u64;
        if want_start > self.input_base {
            let drop = (want_start - self.input_base) as usize;
            let drop = drop.min(self.pending.len().saturating_sub(3));
            if drop > 0 {
                self.pending.drain(..drop);
                self.input_base += drop as u64;
            }
        }
        out
    }
}

fn frames_to_mono_f32(data: &[f32], channels: usize) -> Vec<f32> {
    if channels <= 1 {
        return data.to_vec();
    }
    let frame_count = data.len() / channels;
    let mut v = Vec::with_capacity(frame_count);
    for f in 0..frame_count {
        let base = f * channels;
        let mut s = 0.0f32;
        for c in 0..channels {
            s += data[base + c];
        }
        v.push(s / channels as f32);
    }
    v
}

fn push_resampled(mono: &[f32], resampler: &Arc<Mutex<StreamingResampler>>, sink: &SampleSink) {
    let resampled = {
        let mut r = resampler.lock().expect("resampler mutex poisoned");
        r.push_mono(mono)
    };
    if !resampled.is_empty() {
        sink(&resampled);
    }
}

fn mic_thread_main(
    sink: SampleSink,
    out_rate: u32,
    stop_rx: Receiver<()>,
    ready_tx: Sender<Result<(), String>>,
) {
    let run = || -> Result<()> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No default input device")?;
        let supported = device
            .default_input_config()
            .context("default_input_config")?;
        let sample_format = supported.sample_format();
        let channels = supported.channels() as usize;
        let in_rate = supported.sample_rate().0;

        log::info!(
            "mic: opening stream — device={:?}, rate={} Hz, channels={}, format={:?}",
            device.name().unwrap_or_else(|_| "(unknown)".into()),
            in_rate,
            channels,
            sample_format
        );

        let stream_config = StreamConfig {
            channels: supported.channels(),
            sample_rate: supported.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };
        let resampler = Arc::new(Mutex::new(StreamingResampler::new(in_rate, out_rate)));
        let err_fn = |e| log::error!("cpal input error: {e}");

        let stream = match sample_format {
            SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                {
                    let sink = sink.clone();
                    let resampler = resampler.clone();
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        let mono = frames_to_mono_f32(data, channels);
                        push_resampled(&mono, &resampler, &sink);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                {
                    let sink = sink.clone();
                    let resampler = resampler.clone();
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        let scaled: Vec<f32> = data.iter().map(|&s| s as f32 / 32768.0).collect();
                        let mono = frames_to_mono_f32(&scaled, channels);
                        push_resampled(&mono, &resampler, &sink);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::I32 => device.build_input_stream(
                &stream_config,
                {
                    let sink = sink.clone();
                    let resampler = resampler.clone();
                    let scale = i32::MAX as f32;
                    move |data: &[i32], _: &cpal::InputCallbackInfo| {
                        let scaled: Vec<f32> = data.iter().map(|&s| s as f32 / scale).collect();
                        let mono = frames_to_mono_f32(&scaled, channels);
                        push_resampled(&mono, &resampler, &sink);
                    }
                },
                err_fn,
                None,
            ),
            SampleFormat::F64 => device.build_input_stream(
                &stream_config,
                {
                    let sink = sink.clone();
                    let resampler = resampler.clone();
                    move |data: &[f64], _: &cpal::InputCallbackInfo| {
                        let scaled: Vec<f32> = data.iter().map(|&s| s as f32).collect();
                        let mono = frames_to_mono_f32(&scaled, channels);
                        push_resampled(&mono, &resampler, &sink);
                    }
                },
                err_fn,
                None,
            ),
            fmt => {
                return Err(anyhow::anyhow!(
                    "Unsupported microphone sample format: {fmt:?}"
                ));
            }
        }
        .context("build_input_stream")?;

        stream.play().context("stream.play")?;
        let _ = ready_tx.send(Ok(()));
        let _ = stop_rx.recv();
        drop(stream);
        Ok(())
    };

    if let Err(e) = run() {
        let _ = ready_tx.send(Err(e.to_string()));
    }
}

pub struct MicCaptureSession {
    stop_tx: Sender<()>,
    join: Mutex<Option<JoinHandle<()>>>,
}

impl MicCaptureSession {
    pub fn new(sink: SampleSink, out_rate: u32) -> Result<Self> {
        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        let (ready_tx, ready_rx) = mpsc::channel();
        let join = std::thread::spawn(move || {
            mic_thread_main(sink, out_rate, stop_rx, ready_tx);
        });
        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                stop_tx,
                join: Mutex::new(Some(join)),
            }),
            Ok(Err(e)) => Err(anyhow::anyhow!(e)),
            Err(_) => Err(anyhow::anyhow!("Microphone thread dropped before ready")),
        }
    }

    pub fn stop(&self) -> Result<()> {
        let _ = self.stop_tx.send(());
        if let Some(j) = self.join.lock().expect("mic join mutex poisoned").take() {
            let _ = j.join();
        }
        Ok(())
    }
}
