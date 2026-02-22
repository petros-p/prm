use std::env;
use std::path::{Path, PathBuf};

use hound::WavReader;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

const DEFAULT_MODEL_DIR: &str = ".data/models";
const DEFAULT_MODEL_NAME: &str = "ggml-base.en.bin";

pub fn find_model_path() -> PathBuf {
    if let Ok(path) = env::var("PRM_WHISPER_MODEL") {
        return PathBuf::from(path);
    }
    PathBuf::from(DEFAULT_MODEL_DIR).join(DEFAULT_MODEL_NAME)
}

/// Transcribe a WAV file to text using a local Whisper model.
pub fn transcribe(wav_path: &Path) -> Result<String, String> {
    let model_path = find_model_path();
    if !model_path.exists() {
        return Err(format!(
            "Whisper model not found at {}.\n  Download: https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin\n  Save to: {}",
            model_path.display(),
            model_path.display()
        ));
    }

    let audio = load_wav_as_f32_mono_16k(wav_path)?;

    let ctx = WhisperContext::new_with_params(
        model_path.to_str().ok_or("Invalid model path")?,
        WhisperContextParameters::default(),
    )
    .map_err(|e| format!("Failed to load Whisper model: {:?}", e))?;

    let mut state = ctx
        .create_state()
        .map_err(|e| format!("Failed to create Whisper state: {:?}", e))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("en"));
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    state
        .full(params, &audio)
        .map_err(|e| format!("Transcription failed: {:?}", e))?;

    let n = state
        .full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {:?}", e))?;

    let mut text = String::new();
    for i in 0..n {
        let seg = state
            .full_get_segment_text(i)
            .map_err(|e| format!("Failed to get segment {}: {:?}", i, e))?;
        text.push_str(&seg);
    }

    Ok(text.trim().to_string())
}

fn load_wav_as_f32_mono_16k(path: &Path) -> Result<Vec<f32>, String> {
    let mut reader =
        WavReader::open(path).map_err(|e| format!("Failed to open WAV file: {}", e))?;

    let spec = reader.spec();
    let channels = spec.channels as usize;
    let sample_rate = spec.sample_rate;

    let raw_samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.map_err(|e| format!("WAV read error: {}", e)))
            .collect::<Result<Vec<_>, String>>()?,
        hound::SampleFormat::Int => {
            let max_val = (1i64 << (spec.bits_per_sample - 1)) as f32;
            match spec.bits_per_sample {
                16 => reader
                    .samples::<i16>()
                    .map(|s| {
                        s.map(|v| v as f32 / max_val)
                            .map_err(|e| format!("WAV read error: {}", e))
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                24 | 32 => reader
                    .samples::<i32>()
                    .map(|s| {
                        s.map(|v| v as f32 / max_val)
                            .map_err(|e| format!("WAV read error: {}", e))
                    })
                    .collect::<Result<Vec<_>, String>>()?,
                bits => return Err(format!("Unsupported bit depth: {}", bits)),
            }
        }
    };

    // Interleaved channels → mono (average)
    let mono: Vec<f32> = if channels == 1 {
        raw_samples
    } else {
        raw_samples
            .chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    };

    // Resample to 16 kHz if needed
    let samples_16k = if sample_rate == 16000 {
        mono
    } else {
        resample(&mono, sample_rate, 16000)
    };

    Ok(samples_16k)
}

/// Linear interpolation resampler — good enough for speech transcription.
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio).ceil() as usize;
    (0..new_len)
        .map(|i| {
            let src = i as f64 * ratio;
            let src_i = src.floor() as usize;
            let frac = (src - src_i as f64) as f32;
            let a = samples.get(src_i).copied().unwrap_or(0.0);
            let b = samples.get(src_i + 1).copied().unwrap_or(0.0);
            a + (b - a) * frac
        })
        .collect()
}
