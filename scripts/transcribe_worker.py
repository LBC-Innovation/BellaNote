#!/usr/bin/env python3
"""
Line-delimited JSON worker for faster-whisper. Loads the model once, then for each stdin line:

  {"wav_path": "/path/to/audio"}  # any path faster-whisper/PyAV accepts
  → {"segments":[{"text":"...","start_ms":0,"end_ms":100,...}, ...]}

  {"cmd": "extract_audio", "video_path": "...", "out_path": "..."}
  → {"ok": true}  (stream-copy into .m4a when possible; PCM WAV otherwise)

and flushes stdout.
"""
from __future__ import annotations

import argparse
import json
import math
import os
import sys
import wave


def _to_json_float(value) -> float | None:
    """Coerce CTranslate2 / numpy scalars to finite JSON-safe floats."""
    if value is None:
        return None
    try:
        if hasattr(value, "item"):
            value = value.item()
        v = float(value)
    except (TypeError, ValueError):
        return None
    if not math.isfinite(v):
        return None
    return v


def _first_audio_stream(container):
    for stream in container.streams:
        if stream.type == "audio":
            return stream
    return None


def _extract_audio_copy(video_path: str, out_path: str) -> None:
    """Remux the first audio track without re-encoding (cheap for AAC-in-MP4)."""
    import av

    inp = av.open(video_path)
    try:
        stream = _first_audio_stream(inp)
        if stream is None:
            raise ValueError("This video has no audio track.")
        # ipod = M4A-friendly MP4 audio container
        out = av.open(out_path, mode="w", format="ipod")
        try:
            out_stream = out.add_stream(template=stream)
            for packet in inp.demux(stream):
                if packet.dts is None:
                    continue
                packet.stream = out_stream
                out.mux(packet)
        finally:
            out.close()
    finally:
        inp.close()


def _extract_audio_wav(video_path: str, out_path: str) -> None:
    """Decode audio to 16-bit PCM WAV (fallback when stream-copy is not viable)."""
    import av

    inp = av.open(video_path)
    try:
        stream = _first_audio_stream(inp)
        if stream is None:
            raise ValueError("This video has no audio track.")

        resampler = None
        sample_rate = None
        channels = None
        pcm = bytearray()

        for frame in inp.decode(stream):
            if resampler is None:
                sample_rate = frame.sample_rate or 48_000
                layout = frame.layout.name if frame.layout else "mono"
                channels = len(frame.layout.channels) if frame.layout else 1
                resampler = av.AudioResampler(format="s16", layout=layout, rate=sample_rate)
            for out_frame in resampler.resample(frame):
                pcm.extend(bytes(out_frame.planes[0]))

        if resampler is not None:
            for out_frame in resampler.resample(None):
                pcm.extend(bytes(out_frame.planes[0]))

        if not pcm:
            raise ValueError("Could not decode audio from this video.")

        with wave.open(out_path, "wb") as wf:
            wf.setnchannels(channels or 1)
            wf.setsampwidth(2)
            wf.setframerate(sample_rate or 16_000)
            wf.writeframes(pcm)
    finally:
        inp.close()


def extract_audio(video_path: str, out_path: str) -> None:
    if not video_path or not out_path:
        raise ValueError("video_path and out_path are required")
    lower = out_path.lower()
    if lower.endswith((".m4a", ".mp4", ".aac")):
        _extract_audio_copy(video_path, out_path)
        return
    if lower.endswith(".wav"):
        _extract_audio_wav(video_path, out_path)
        return
    raise ValueError("out_path must end with .m4a or .wav")


def main() -> None:
    p = argparse.ArgumentParser(description="faster-whisper stdin/stdout worker")
    p.add_argument(
        "--model",
        default="small.en",
        help="Model size (e.g. tiny.en, base.en, small.en) or path to a CTranslate2 model directory",
    )
    p.add_argument(
        "--download-root",
        default=None,
        help="Directory where Whisper models are downloaded and cached",
    )
    args = p.parse_args()

    try:
        from faster_whisper import WhisperModel
    except ImportError as e:
        print(
            json.dumps(
                {
                    "error": "faster-whisper not installed. Run: pip install faster-whisper",
                    "detail": str(e),
                }
            ),
            flush=True,
        )
        sys.exit(1)

    try:
        model_kwargs = {
            "device": "auto",
            "compute_type": "auto",
        }
        if args.download_root:
            model_kwargs["download_root"] = args.download_root
        model = WhisperModel(args.model, **model_kwargs)
    except Exception as e:
        print(json.dumps({"error": "failed to load model", "detail": str(e)}), flush=True)
        sys.exit(1)

    print(json.dumps({"status": "ready"}), flush=True)

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
        except json.JSONDecodeError as e:
            print(json.dumps({"error": "bad json", "detail": str(e)}), flush=True)
            continue

        if req.get("cmd") == "extract_audio":
            video_path = req.get("video_path")
            out_path = req.get("out_path")
            try:
                extract_audio(video_path, out_path)
                if not out_path or not os.path.isfile(out_path) or os.path.getsize(out_path) == 0:
                    raise ValueError("Extracted audio file is missing or empty.")
                print(json.dumps({"ok": True}), flush=True)
            except Exception as e:
                if out_path:
                    try:
                        os.remove(out_path)
                    except OSError:
                        pass
                print(json.dumps({"error": "extract failed", "detail": str(e)}), flush=True)
            continue

        wav_path = req.get("wav_path")
        if not wav_path:
            print(json.dumps({"error": "missing wav_path"}), flush=True)
            continue

        try:
            segments, _info = model.transcribe(
                wav_path,
                language="en",
                beam_size=5,
                vad_filter=True,
            )
            out = []
            for s in segments:
                t = s.text.strip()
                if not t:
                    continue
                avg = _to_json_float(getattr(s, "avg_logprob", None))
                nsp = _to_json_float(getattr(s, "no_speech_prob", None))
                row = {
                    "text": t,
                    "start_ms": int(round(s.start * 1000)),
                    "end_ms": int(round(s.end * 1000)),
                }
                if avg is not None and nsp is not None:
                    row["avg_logprob"] = avg
                    row["no_speech_prob"] = nsp
                out.append(row)
            print(json.dumps({"segments": out}), flush=True)
        except Exception as e:
            print(json.dumps({"error": "transcribe failed", "detail": str(e)}), flush=True)


if __name__ == "__main__":
    main()
