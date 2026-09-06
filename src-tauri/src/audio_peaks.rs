use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

pub const PEAK_BUCKETS: usize = 180;
const WINDOW_SAMPLES: usize = 4096;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioPeaksResult {
    pub peaks: Vec<f32>,
    pub duration_secs: f64,
}

fn cache() -> &'static Mutex<HashMap<String, AudioPeaksResult>> {
    static CACHE: OnceLock<Mutex<HashMap<String, AudioPeaksResult>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn cache_key(artifact_id: &str, path: &Path) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let mtime_nanos = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    Ok(format!("{artifact_id}:{}:{mtime_nanos}", meta.len()))
}

pub fn get_cached(key: &str) -> Option<AudioPeaksResult> {
    cache().lock().ok()?.get(key).cloned()
}

pub fn put_cached(key: String, value: AudioPeaksResult) {
    if let Ok(mut guard) = cache().lock() {
        guard.insert(key, value);
    }
}

pub fn invalidate_artifact(artifact_id: &str) {
    let prefix = format!("{artifact_id}:");
    if let Ok(mut guard) = cache().lock() {
        guard.retain(|key, _| !key.starts_with(&prefix));
    }
}

pub fn compute_audio_peaks(path: &Path, num_buckets: usize) -> Result<AudioPeaksResult, String> {
    let file = File::open(path).map_err(|e| format!("Couldn’t read this audio file: {e}"))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| format!("Couldn’t read this audio file: {e}"))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "This file has no audio track.".to_string())?
        .clone();
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Couldn’t decode this audio file: {e}"))?;

    let mut sample_buf: Option<SampleBuffer<f32>> = None;
    let mut sample_buf_cap = 0u64;
    let mut sample_rate = track.codec_params.sample_rate.unwrap_or(0);
    let mut windows: Vec<f32> = Vec::new();
    let mut window_max = 0.0f32;
    let mut in_window = 0usize;
    let mut total_frames: u64 = 0;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(err)) if err.kind() == ErrorKind::UnexpectedEof => break,
            Err(SymphoniaError::ResetRequired) => break,
            Err(SymphoniaError::IoError(_)) | Err(SymphoniaError::DecodeError(_)) => continue,
            Err(_) => break,
        };
        if packet.track_id() != track_id {
            continue;
        }

        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(_) => break,
        };

        if sample_rate == 0 {
            sample_rate = decoded.spec().rate;
        }
        let channels = decoded.spec().channels.count().max(1);
        let needed = decoded.capacity() as u64;
        if sample_buf.is_none() || needed > sample_buf_cap {
            sample_buf = Some(SampleBuffer::<f32>::new(needed, *decoded.spec()));
            sample_buf_cap = needed;
        }
        let Some(buf) = sample_buf.as_mut() else { continue };
        buf.copy_interleaved_ref(decoded);
        let samples = buf.samples();
        let frames = samples.len() / channels;
        total_frames += frames as u64;
        for frame in 0..frames {
            let sample = samples[frame * channels];
            window_max = window_max.max(sample.abs());
            in_window += 1;
            if in_window >= WINDOW_SAMPLES {
                windows.push(window_max);
                window_max = 0.0;
                in_window = 0;
            }
        }
    }

    if in_window > 0 {
        windows.push(window_max);
    }

    let mut peaks = downsample_max(&windows, num_buckets.max(1));
    let peak = peaks.iter().copied().fold(0.001f32, f32::max);
    for value in &mut peaks {
        *value /= peak;
    }

    let duration_secs = if sample_rate > 0 {
        total_frames as f64 / f64::from(sample_rate)
    } else {
        0.0
    };

    Ok(AudioPeaksResult {
        peaks,
        duration_secs,
    })
}

fn downsample_max(values: &[f32], buckets: usize) -> Vec<f32> {
    if values.is_empty() {
        return vec![0.0; buckets];
    }
    let mut out = Vec::with_capacity(buckets);
    for i in 0..buckets {
        let start = i * values.len() / buckets;
        let mut end = (i + 1) * values.len() / buckets;
        if end <= start {
            end = (start + 1).min(values.len());
        } else {
            end = end.min(values.len());
        }
        let mut max = 0.0f32;
        for value in &values[start..end] {
            max = max.max(*value);
        }
        out.push(max);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn write_pcm16_wav(path: &Path, sample_rate: u32, samples: &[i16]) {
        let data_len = (samples.len() * 2) as u32;
        let mut bytes = Vec::with_capacity(44 + data_len as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&(36 + data_len).to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&(sample_rate * 2).to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        let mut file = File::create(path).expect("create wav");
        file.write_all(&bytes).expect("write wav");
    }

    fn temp_wav(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("bellanote-peaks-{name}-{}.wav", std::process::id()))
    }

    #[test]
    fn downsample_max_shrinks_by_taking_the_loudest_bin() {
        let values = [0.1, 0.8, 0.2, 0.4];
        let out = downsample_max(&values, 2);
        assert_eq!(out, vec![0.8, 0.4]);
    }

    #[test]
    fn downsample_max_empty_is_silence() {
        assert_eq!(downsample_max(&[], 4), vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn wav_peaks_are_quiet_then_loud() {
        let path = temp_wav("quiet-loud");
        let sample_rate = 44_100u32;
        let mut samples = vec![0i16; sample_rate as usize];
        samples.extend(std::iter::repeat(24_000i16).take(sample_rate as usize));
        write_pcm16_wav(&path, sample_rate, &samples);

        let result = compute_audio_peaks(&path, 20).expect("peaks");
        let _ = std::fs::remove_file(&path);

        assert_eq!(result.peaks.len(), 20);
        assert!((result.duration_secs - 2.0).abs() < 0.05);
        let first = result.peaks[..8].iter().copied().fold(0.0f32, f32::max);
        let last = result.peaks[12..].iter().copied().fold(0.0f32, f32::max);
        assert!(first < 0.15, "expected quiet start, got {first}");
        assert!(last > 0.8, "expected loud end, got {last}");
    }

    #[test]
    fn cache_round_trip_and_invalidate() {
        invalidate_artifact("art-1");
        let value = AudioPeaksResult {
            peaks: vec![0.2, 1.0],
            duration_secs: 1.5,
        };
        put_cached("art-1:10:20".into(), value.clone());
        put_cached("art-2:10:20".into(), value.clone());
        assert_eq!(get_cached("art-1:10:20").unwrap().duration_secs, 1.5);
        invalidate_artifact("art-1");
        assert!(get_cached("art-1:10:20").is_none());
        assert!(get_cached("art-2:10:20").is_some());
        invalidate_artifact("art-2");
    }
}
