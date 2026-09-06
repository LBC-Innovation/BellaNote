#!/usr/bin/env python3
"""
Line-delimited JSON worker for faster-whisper. Loads the model once, then for each stdin line:
  {"wav_path": "/path/to/chunk.wav"}  # any path faster-whisper/ffmpeg accepts (we still write temp WAV chunks from Rust)
prints one line of JSON:
  {"segments":[{"text":"...","start_ms":0,"end_ms":100,"avg_logprob":-0.3,"no_speech_prob":0.1}, ...]}
and flushes stdout.
"""
from __future__ import annotations

import argparse
import json
import math
import sys


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
