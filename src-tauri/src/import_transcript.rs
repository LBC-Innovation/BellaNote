use crate::transcribe::TranscriptSegment;

fn parse_timestamp_to_ms(ts: &str) -> Option<i64> {
    let ts = ts.trim().replace(',', ".");
    let parts: Vec<&str> = ts.split(':').collect();
    if parts.len() == 3 {
        let h: i64 = parts[0].parse().ok()?;
        let m: i64 = parts[1].parse().ok()?;
        let (s_str, ms_str) = parts[2].split_once('.').unwrap_or((parts[2], "0"));
        let s: i64 = s_str.parse().ok()?;
        let ms: i64 = ms_str.chars().take(3).collect::<String>().parse().ok()?;
        return Some(((h * 3600 + m * 60 + s) * 1000) + ms);
    }
    if parts.len() == 2 {
        let m: i64 = parts[0].parse().ok()?;
        let (s_str, ms_str) = parts[1].split_once('.').unwrap_or((parts[1], "0"));
        let s: i64 = s_str.parse().ok()?;
        let ms: i64 = ms_str.chars().take(3).collect::<String>().parse().ok()?;
        return Some(((m * 60 + s) * 1000) + ms);
    }
    None
}

fn strip_tags(input: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn parse_cues(raw: &str) -> Vec<TranscriptSegment> {
    let mut segs = Vec::new();
    let lines: Vec<&str> = raw.lines().collect();
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line == "WEBVTT" || line.chars().all(|c| c.is_ascii_digit()) {
            i += 1;
            continue;
        }
        let mut time_line = line;
        if !time_line.contains("-->") && i + 1 < lines.len() && lines[i + 1].contains("-->") {
            i += 1;
            time_line = lines[i].trim();
        }
        if !time_line.contains("-->") {
            i += 1;
            continue;
        }
        let Some((start_raw, rest)) = time_line.split_once("-->") else {
            i += 1;
            continue;
        };
        let end_raw = rest.split_whitespace().next().unwrap_or("").trim();
        let start_ms = parse_timestamp_to_ms(start_raw).unwrap_or(0);
        let end_ms = parse_timestamp_to_ms(end_raw).unwrap_or(start_ms);
        i += 1;
        let mut text_lines = Vec::new();
        while i < lines.len() {
            let t = lines[i].trim_end();
            if t.trim().is_empty() {
                break;
            }
            text_lines.push(strip_tags(t).trim().to_string());
            i += 1;
        }
        let text = text_lines.join(" ").split_whitespace().collect::<Vec<_>>().join(" ");
        if !text.is_empty() {
            segs.push(TranscriptSegment {
                text,
                start_ms,
                end_ms,
                avg_logprob: None,
                no_speech_prob: None,
            });
        }
        i += 1;
    }
    segs
}

pub fn parse_transcript_file(path: &std::path::Path, raw: &str) -> Result<(Vec<TranscriptSegment>, String), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if raw.trim().is_empty() {
        return Err("The file has no text.".into());
    }
    if raw.len() > 2_000_000 {
        return Err("That transcript is larger than 2 MB.".into());
    }

    let segs = if ext == "txt" && !raw.contains("-->") {
        vec![TranscriptSegment {
            text: raw.trim().to_string(),
            start_ms: 0,
            end_ms: 0,
            avg_logprob: None,
            no_speech_prob: None,
        }]
    } else {
        parse_cues(raw)
    };

    if segs.is_empty() {
        return Err("Could not find any transcript text in that file.".into());
    }
    let full = segs.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
    Ok((segs, full))
}
