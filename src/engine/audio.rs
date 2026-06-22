use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};

pub struct AudioSystem {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    volume: f32,
    shoot_data: Option<Vec<u8>>,
    explosion_data: Option<Vec<u8>>,
    place_data: Option<Vec<u8>>,
    click_data: Option<Vec<u8>>,
}

impl AudioSystem {
    pub fn new() -> Self {
        log::info!("Initializing AudioSystem...");
        
        // Ensure assets/sounds/ exists
        let sounds_dir = Path::new("assets/sounds");
        if !sounds_dir.exists() {
            if let Err(e) = fs::create_dir_all(sounds_dir) {
                log::error!("Failed to create directory assets/sounds: {}", e);
            }
        }

        // Load or generate assets
        let shoot_data = Self::load_or_generate(sounds_dir.join("shoot.wav"), SoundPreset::Shoot);
        let explosion_data = Self::load_or_generate(sounds_dir.join("explosion.wav"), SoundPreset::Explosion);
        let place_data = Self::load_or_generate(sounds_dir.join("place.wav"), SoundPreset::Place);
        let click_data = Self::load_or_generate(sounds_dir.join("click.wav"), SoundPreset::Click);

        // Try initializing audio device
        let (stream, stream_handle) = match OutputStream::try_default() {
            Ok((s, h)) => {
                log::info!("Audio device initialized successfully.");
                (Some(s), Some(h))
            }
            Err(e) => {
                log::warn!("Could not initialize default audio output device: {}. Audio will run in silent mode.", e);
                (None, None)
            }
        };

        Self {
            _stream: stream,
            stream_handle,
            volume: 0.5, // Default volume at 50%
            shoot_data,
            explosion_data,
            place_data,
            click_data,
        }
    }

    /// Plays a sound using the cached bytes and current volume.
    fn play_sound(&self, data: &Option<Vec<u8>>, name: &str) {
        let handle = match &self.stream_handle {
            Some(h) => h,
            None => return, // Silent mode
        };

        let bytes = match data {
            Some(b) => b,
            None => return,
        };

        let cursor = Cursor::new(bytes.clone());
        match Decoder::new(cursor) {
            Ok(source) => {
                // Apply amplification (volume)
                let source = source.amplify(self.volume).convert_samples();
                if let Err(e) = handle.play_raw(source) {
                    log::error!("Failed to play sound '{}': {:?}", name, e);
                }
            }
            Err(e) => {
                log::error!("Failed to decode sound '{}': {:?}", name, e);
            }
        }
    }

    pub fn play_shoot(&self) {
        self.play_sound(&self.shoot_data, "shoot");
    }

    pub fn play_explosion(&self) {
        self.play_sound(&self.explosion_data, "explosion");
    }

    pub fn play_place(&self) {
        self.play_sound(&self.place_data, "place");
    }

    pub fn play_click(&self) {
        self.play_sound(&self.click_data, "click");
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn get_volume(&self) -> f32 {
        self.volume
    }

    /// Loads the sound from disk if present, or generates it programmatically.
    fn load_or_generate(path: PathBuf, preset: SoundPreset) -> Option<Vec<u8>> {
        if path.exists() {
            match fs::read(&path) {
                Ok(bytes) => {
                    log::info!("Loaded sound from file: {:?}", path);
                    return Some(bytes);
                }
                Err(e) => {
                    log::error!("Failed to read sound file at {:?}: {}, regenerating...", path, e);
                }
            }
        }

        // Generate preset
        log::info!("Generating sound effect preset: {:?}", preset);
        let bytes = preset.generate();
        if let Err(e) = fs::write(&path, &bytes) {
            log::error!("Failed to write generated sound file to {:?}: {}", path, e);
        }
        Some(bytes)
    }
}

#[derive(Debug, Clone, Copy)]
enum SoundPreset {
    Shoot,
    Explosion,
    Place,
    Click,
}

impl SoundPreset {
    fn generate(&self) -> Vec<u8> {
        let sample_rate = 44100;
        let samples = match self {
            SoundPreset::Shoot => {
                let duration = 0.12;
                let num_samples = (sample_rate as f32 * duration) as usize;
                let mut samples = Vec::with_capacity(num_samples);
                let mut phase = 0.0f32;
                for i in 0..num_samples {
                    let t = i as f32 / sample_rate as f32;
                    let progress = t / duration;
                    let freq = 800.0 * (1.0 - progress) + 150.0 * progress;
                    phase += 2.0 * std::f32::consts::PI * freq / sample_rate as f32;
                    let amp = 1.0 - progress;
                    let sample = (phase.sin() * amp * 12000.0) as i16;
                    samples.push(sample);
                }
                samples
            }
            SoundPreset::Explosion => {
                let duration = 0.4;
                let num_samples = (sample_rate as f32 * duration) as usize;
                let mut samples = Vec::with_capacity(num_samples);
                
                // Simple RNG LCG to produce white noise
                let mut seed = 12345u32;
                let mut last_out = 0.0f32;
                let alpha = 0.12f32; // Cutoff coefficient for low rumble
                
                for i in 0..num_samples {
                    let t = i as f32 / sample_rate as f32;
                    let progress = t / duration;
                    
                    seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
                    let noise = ((seed >> 16) & 0xFFFF) as f32 / 65535.0 * 2.0 - 1.0;
                    
                    let filtered = alpha * noise + (1.0 - alpha) * last_out;
                    last_out = filtered;
                    
                    let amp = (1.0 - progress).powi(2);
                    let sample = (filtered * amp * 15000.0) as i16;
                    samples.push(sample);
                }
                samples
            }
            SoundPreset::Place => {
                let duration = 0.15;
                let num_samples = (sample_rate as f32 * duration) as usize;
                let mut samples = Vec::with_capacity(num_samples);
                let mut phase = 0.0f32;
                for i in 0..num_samples {
                    let t = i as f32 / sample_rate as f32;
                    let progress = t / duration;
                    let freq = 300.0 + progress * 300.0;
                    phase += 2.0 * std::f32::consts::PI * freq / sample_rate as f32;
                    
                    let amp = if progress < 0.2 {
                        progress / 0.2
                    } else {
                        (1.0 - progress) / 0.8
                    };
                    let sample = (phase.sin() * amp * 10000.0) as i16;
                    samples.push(sample);
                }
                samples
            }
            SoundPreset::Click => {
                let duration = 0.04;
                let num_samples = (sample_rate as f32 * duration) as usize;
                let mut samples = Vec::with_capacity(num_samples);
                let mut phase = 0.0f32;
                for i in 0..num_samples {
                    let t = i as f32 / sample_rate as f32;
                    let progress = t / duration;
                    let freq = 1200.0;
                    phase += 2.0 * std::f32::consts::PI * freq / sample_rate as f32;
                    let amp = (1.0 - progress).powi(3);
                    let sample = (phase.sin() * amp * 8000.0) as i16;
                    samples.push(sample);
                }
                samples
            }
        };

        generate_wav_bytes(&samples, sample_rate)
    }
}

/// Helper function to construct standard 44-byte PCM WAV file header and format sample bytes.
fn generate_wav_bytes(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(44 + samples.len() * 2);
    let subchunk2_size = (samples.len() * 2) as u32;
    let chunk_size = 36 + subchunk2_size;
    
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&chunk_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat = 1 PCM
    bytes.extend_from_slice(&1u16.to_le_bytes()); // NumChannels = 1 Mono
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 1 * 2;
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes()); // BlockAlign = 2
    bytes.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample = 16
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&subchunk2_size.to_le_bytes());
    
    for &sample in samples {
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_generation() {
        let preset = SoundPreset::Click;
        let bytes = preset.generate();
        assert!(bytes.len() > 44);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");
    }

    #[test]
    fn test_audio_system_initialization() {
        // AudioSystem should initialize cleanly without panic even in headless test environments.
        let sys = AudioSystem::new();
        assert_eq!(sys.volume, 0.5);
        assert!(sys.shoot_data.is_some());
        assert!(sys.explosion_data.is_some());
        assert!(sys.place_data.is_some());
        assert!(sys.click_data.is_some());
    }
}
