use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub struct AudioSystem {
    _stream: Option<OutputStream>,
    stream_handle: Option<OutputStreamHandle>,
    volume: f32,
    music_volume: f32,
    music_muted: bool,
    music_sink: Option<rodio::Sink>,
    shoot_data: Option<Vec<u8>>,
    explosion_data: Option<Vec<u8>>,
    place_data: Option<Vec<u8>>,
    click_data: Option<Vec<u8>>,
    _music_data: Option<Vec<u8>>,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
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
        let explosion_data =
            Self::load_or_generate(sounds_dir.join("explosion.wav"), SoundPreset::Explosion);
        let place_data = Self::load_or_generate(sounds_dir.join("place.wav"), SoundPreset::Place);
        let click_data = Self::load_or_generate(sounds_dir.join("click.wav"), SoundPreset::Click);
        let music_data = Self::load_or_generate(sounds_dir.join("music.wav"), SoundPreset::Music);

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

        let mut music_sink = None;
        if let Some(ref handle) = stream_handle {
            if let Some(ref m_data) = music_data {
                match rodio::Sink::try_new(handle) {
                    Ok(sink) => {
                        let cursor = Cursor::new(m_data.clone());
                        match Decoder::new(cursor) {
                            Ok(source) => {
                                sink.append(source.repeat_infinite().convert_samples::<f32>());
                                sink.set_volume(0.3); // Default music volume at 30%
                                sink.play();
                                music_sink = Some(sink);
                                log::info!("Background music playback started.");
                            }
                            Err(e) => {
                                log::error!("Failed to decode music: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to create music sink: {:?}", e);
                    }
                }
            }
        }

        Self {
            _stream: stream,
            stream_handle,
            volume: 0.5, // Default volume at 50%
            music_volume: 0.3,
            music_muted: false,
            music_sink,
            shoot_data,
            explosion_data,
            place_data,
            click_data,
            _music_data: music_data,
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

    pub fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume.clamp(0.0, 1.0);
        self.update_music_volume();
    }

    pub fn get_music_volume(&self) -> f32 {
        self.music_volume
    }

    pub fn set_music_muted(&mut self, muted: bool) {
        self.music_muted = muted;
        self.update_music_volume();
    }

    pub fn get_music_muted(&self) -> bool {
        self.music_muted
    }

    fn update_music_volume(&self) {
        if let Some(sink) = &self.music_sink {
            let volume = if self.music_muted {
                0.0
            } else {
                self.music_volume
            };
            sink.set_volume(volume);
        }
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
                    log::error!(
                        "Failed to read sound file at {:?}: {}, regenerating...",
                        path,
                        e
                    );
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
    Music,
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
            SoundPreset::Music => {
                let duration = 8.0; // 8 seconds
                let num_samples = (sample_rate as f32 * duration) as usize;
                let mut samples = Vec::with_capacity(num_samples);

                let melody_freqs = [
                    261.63, 329.63, 392.00, 523.25, // C4, E4, G4, C5
                    220.00, 261.63, 329.63, 440.00, // A3, C4, E4, A4
                    174.61, 220.00, 261.63, 349.23, // F3, A3, C4, F4
                    196.00, 246.94, 293.66, 392.00, // G3, B3, D4, G4
                ];
                let bass_freqs = [
                    65.41, // C2
                    55.00, // A1
                    43.65, // F1
                    49.00, // G1
                ];

                let mut melody_phase = 0.0f32;
                let mut bass_phase = 0.0f32;

                for i in 0..num_samples {
                    let t = i as f32 / sample_rate as f32;
                    let beat = (t / 0.5) as usize % 16;
                    let t_beat = t % 0.5;

                    let m_freq = melody_freqs[beat];
                    melody_phase += 2.0 * std::f32::consts::PI * m_freq / sample_rate as f32;

                    let b_freq = bass_freqs[beat / 4];
                    bass_phase += 2.0 * std::f32::consts::PI * b_freq / sample_rate as f32;

                    // Pluck envelope for melody: starts fast, decays exponentially
                    let melody_amp = (-12.0 * t_beat).exp();

                    // Simple sine melody
                    let melody_val = melody_phase.sin() * melody_amp;

                    // Bass note (triangle wave or soft sine, held for 4 beats)
                    // Let's use a soft sine wave with a slower attack/decay
                    let t_chord = t % 2.0;
                    let bass_amp = if t_chord < 0.2 {
                        t_chord / 0.2
                    } else {
                        ((2.0 - t_chord) / 1.8).clamp(0.0, 1.0)
                    };
                    let bass_val = bass_phase.sin() * bass_amp * 0.7;

                    // Mix them
                    let mixed = (melody_val * 0.4 + bass_val * 0.6) * 10000.0;
                    samples.push(mixed as i16);
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
    let byte_rate = sample_rate * 2;
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
    fn test_music_generation() {
        let preset = SoundPreset::Music;
        let bytes = preset.generate();
        assert!(bytes.len() > 44);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
    }

    #[test]
    fn test_audio_system_initialization() {
        // AudioSystem should initialize cleanly without panic even in headless test environments.
        let mut sys = AudioSystem::new();
        assert_eq!(sys.volume, 0.5);
        assert_eq!(sys.music_volume, 0.3);
        assert!(!sys.music_muted);
        assert!(sys.shoot_data.is_some());
        assert!(sys.explosion_data.is_some());
        assert!(sys.place_data.is_some());
        assert!(sys.click_data.is_some());
        assert!(sys._music_data.is_some());

        sys.set_music_volume(0.8);
        assert_eq!(sys.get_music_volume(), 0.8);

        sys.set_music_muted(true);
        assert!(sys.get_music_muted());
    }
}
