use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Numero di campioni accumulati prima di eseguire il pitch detection.
/// Con 44100 Hz → ~93 ms di audio per analisi.
const BUFFER_SIZE: usize = 4096;

/// Soglia minima di volume (RMS) sotto cui non rilevare il pitch (silenzio/rumore).
const MIN_VOLUME: f32 = 0.01;

/// Range di frequenze cercate: 80 Hz (do basso) – 1200 Hz (fischio acuto).
const MIN_FREQ: f32 = 80.0;
const MAX_FREQ: f32 = 1200.0;

const TICKS_PER_BEAT: u16 = 480;
const TEMPO_BPM: f64 = 120.0;
const OUTPUT_MIDI: &str = "output.mid";

/// Quanti frame consecutivi con la stessa nota prima di confermarla.
/// Con ~46 ms/frame → 4 frame ≈ 185 ms di debounce.
const STABLE_FRAMES: u32 = 4;

// ----------------------- STATO DI REGISTRAZIONE -----------------------

struct NoteSegment {
    midi_note: u8,
    duration_ticks: u32,
    silence_before_ticks: u32,
}

struct RecordingState {
    segments: Vec<NoteSegment>,
    current_note: Option<u8>,
    current_ticks: u32,
    silence_before_current: u32,
    pending_silence: u32,
    ticks_per_step: u32,
    /// Nota candidata (rilevata ma non ancora confermata).
    candidate: Option<u8>,
    candidate_frames: u32,
}

impl RecordingState {
    fn new(ticks_per_step: u32) -> Self {
        Self {
            segments: Vec::new(),
            current_note: None,
            current_ticks: 0,
            silence_before_current: 0,
            pending_silence: 0,
            ticks_per_step,
            candidate: None,
            candidate_frames: 0,
        }
    }

    /// Riceve un rilevamento grezzo e lo filtra con isteresi prima di registrarlo.
    /// Una nota viene confermata solo dopo STABLE_FRAMES consecutivi con lo stesso valore.
    /// Finché non è confermata, la nota corrente viene prolungata (o il silenzio accumulato).
    fn push_raw(&mut self, detected: Option<u8>) {
        if detected == self.current_note {
            // Stessa nota già confermata: estendi
            self.push_step(detected);
            self.candidate = None;
            self.candidate_frames = 0;
        } else if detected == self.candidate {
            self.candidate_frames += 1;
            if self.candidate_frames >= STABLE_FRAMES {
                // Nuova nota (o silenzio) confermata
                self.push_step(detected);
                self.candidate = None;
                self.candidate_frames = 0;
            } else {
                // Ancora in debounce: tieni la nota corrente
                self.push_step(self.current_note);
            }
        } else {
            // Nuova candidata: resetta contatore
            self.candidate = detected;
            self.candidate_frames = 1;
            self.push_step(self.current_note);
        }
    }

    fn push_step(&mut self, midi_note: Option<u8>) {
        match (self.current_note, midi_note) {
            (Some(prev), Some(cur)) if prev == cur => {
                self.current_ticks += self.ticks_per_step;
            }
            (Some(prev), new_note) => {
                self.segments.push(NoteSegment {
                    midi_note: prev,
                    duration_ticks: self.current_ticks,
                    silence_before_ticks: self.silence_before_current,
                });
                self.silence_before_current = 0;
                if let Some(new) = new_note {
                    self.current_note = Some(new);
                    self.current_ticks = self.ticks_per_step;
                    self.pending_silence = 0;
                } else {
                    self.current_note = None;
                    self.current_ticks = 0;
                    self.pending_silence = self.ticks_per_step;
                }
            }
            (None, Some(new)) => {
                self.current_note = Some(new);
                self.current_ticks = self.ticks_per_step;
                self.silence_before_current = self.pending_silence;
                self.pending_silence = 0;
            }
            (None, None) => {
                self.pending_silence += self.ticks_per_step;
            }
        }
    }

    fn finalize_and_write(&mut self, path: &str) {
        if let Some(note) = self.current_note {
            if self.current_ticks > 0 {
                self.segments.push(NoteSegment {
                    midi_note: note,
                    duration_ticks: self.current_ticks,
                    silence_before_ticks: self.silence_before_current,
                });
            }
        }
        let tempo_us = (60_000_000.0 / TEMPO_BPM) as u32;
        write_midi_file(&self.segments, TICKS_PER_BEAT, tempo_us, path);
    }
}

// ----------------------- SCRITTURA FILE MIDI (formato 0, raw bytes) -----------------------

fn write_varint(buf: &mut Vec<u8>, mut value: u32) {
    let mut bytes = [0u8; 4];
    let mut count = 0;
    loop {
        bytes[count] = (value & 0x7F) as u8;
        value >>= 7;
        count += 1;
        if value == 0 {
            break;
        }
    }
    for i in (0..count).rev() {
        buf.push(if i > 0 { bytes[i] | 0x80 } else { bytes[i] });
    }
}

fn write_midi_file(segments: &[NoteSegment], ticks_per_beat: u16, tempo_us: u32, path: &str) {
    let mut track: Vec<u8> = Vec::new();

    // Set Tempo meta event
    write_varint(&mut track, 0);
    track.extend_from_slice(&[0xFF, 0x51, 0x03]);
    track.push(((tempo_us >> 16) & 0xFF) as u8);
    track.push(((tempo_us >> 8) & 0xFF) as u8);
    track.push((tempo_us & 0xFF) as u8);

    // Program Change: canale 0 → pianoforte
    write_varint(&mut track, 0);
    track.extend_from_slice(&[0xC0, 0x00]);

    for seg in segments {
        // Note On (delta = silenzio precedente)
        write_varint(&mut track, seg.silence_before_ticks);
        track.extend_from_slice(&[0x90, seg.midi_note, 80]);
        // Note Off (delta = durata)
        write_varint(&mut track, seg.duration_ticks);
        track.extend_from_slice(&[0x80, seg.midi_note, 0]);
    }

    // End of Track
    write_varint(&mut track, 0);
    track.extend_from_slice(&[0xFF, 0x2F, 0x00]);

    let mut file: Vec<u8> = Vec::new();
    // Header MThd
    file.extend_from_slice(b"MThd");
    file.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]);
    file.extend_from_slice(&[0x00, 0x00]); // formato 0
    file.extend_from_slice(&[0x00, 0x01]); // 1 track
    file.extend_from_slice(&ticks_per_beat.to_be_bytes());
    // Track MTrk
    file.extend_from_slice(b"MTrk");
    file.extend_from_slice(&(track.len() as u32).to_be_bytes());
    file.extend_from_slice(&track);

    std::fs::write(path, &file).expect("Errore scrittura file MIDI");
}

// ----------------------- MAIN -----------------------

fn main() {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .expect("Nessun microfono trovato");
    println!("Microfono: {}", device.name().unwrap());

    let config = device
        .default_input_config()
        .expect("Config input fallita");
    println!("Config: {:?}", config);

    let sample_rate = config.sample_rate().0 as f32;

    // Ogni step = BUFFER_SIZE/2 campioni (overlap 50%)
    let step_samples = (BUFFER_SIZE / 2) as f64;
    let ticks_per_step = (step_samples / sample_rate as f64
        * TEMPO_BPM
        / 60.0
        * TICKS_PER_BEAT as f64)
        .round()
        .max(1.0) as u32;

    let state = Arc::new(Mutex::new(RecordingState::new(ticks_per_step)));
    let state_ctrlc = Arc::clone(&state);

    ctrlc::set_handler(move || {
        let mut rec = state_ctrlc.lock().unwrap();
        rec.finalize_and_write(OUTPUT_MIDI);
        let count = rec.segments.len();
        println!(
            "\nRegistrazione terminata. {} note salvate in \"{}\".",
            count, OUTPUT_MIDI
        );
        std::process::exit(0);
    })
    .expect("Impossibile impostare handler CTRL+C");

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => {
            build_stream_f32(&device, &config.into(), sample_rate, Arc::clone(&state))
        }
        cpal::SampleFormat::I16 => {
            build_stream_i16(&device, &config.into(), sample_rate, Arc::clone(&state))
        }
        cpal::SampleFormat::U16 => {
            build_stream_u16(&device, &config.into(), sample_rate, Arc::clone(&state))
        }
        _ => panic!("Formato audio non supportato"),
    };

    stream.play().expect("Errore avvio stream");
    println!("In ascolto... premi CTRL+C per fermare e salvare il file MIDI.");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

/// Rileva la frequenza fondamentale (Hz) in un buffer di campioni f32 normalizzati [-1, 1].
/// Usa l'autocorrelazione: cerca il lag τ che massimizza la somma s[i]*s[i+τ].
/// Restituisce None se il segnale è troppo debole o non si trova un picco chiaro.
fn detect_pitch(samples: &[f32], sample_rate: f32) -> Option<f32> {
    let n = samples.len();

    // Converti il range di frequenze in lagrange di campioni
    let min_lag = (sample_rate / MAX_FREQ).ceil() as usize;
    let max_lag = (sample_rate / MIN_FREQ).floor() as usize;

    if max_lag >= n {
        return None;
    }

    // Calcolo dell'autocorrelazione per ogni lag candidato
    let mut best_lag = 0usize;
    let mut best_corr = f32::NEG_INFINITY;

    for lag in min_lag..=max_lag {
        let corr: f32 = (0..n - lag)
            .map(|i| samples[i] * samples[i + lag])
            .sum();

        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    // Verifica che il picco trovato sia effettivamente positivo (segnale periodico)
    if best_lag == 0 || best_corr <= 0.0 {
        return None;
    }

    Some(sample_rate / best_lag as f32)
}

/// Converte una frequenza in Hz nel nome della nota MIDI più vicina (es. "La4", "Do#5").
fn freq_to_note(freq: f32) -> String {
    let note_names = ["Do", "Do#", "Re", "Re#", "Mi", "Fa", "Fa#", "Sol", "Sol#", "La", "La#", "Si"];

    // MIDI note 69 = La4 = 440 Hz
    let midi = (69.0 + 12.0 * (freq / 440.0).log2()).round() as i32;
    if midi < 0 || midi > 127 {
        return format!("{:.1} Hz (fuori range MIDI)", freq);
    }

    let octave = (midi / 12) - 1;
    let note = note_names[(midi % 12) as usize];
    format!("{}{} (MIDI {})", note, octave, midi)
}

/// Processa un buffer f32 normalizzato: restituisce la nota MIDI rilevata (o None per silenzio).
fn process_buffer(buffer: &[f32], sample_rate: f32) -> Option<u8> {
    let rms = rms_f32(buffer);

    if rms < MIN_VOLUME {
        return None;
    }

    match detect_pitch(buffer, sample_rate) {
        Some(freq) => {
            let midi = (69.0_f32 + 12.0 * (freq / 440.0).log2()).round() as i32;
            let midi = midi.clamp(0, 127) as u8;
            println!(
                "Volume: {:.4} | Freq: {:>7.2} Hz | Nota: {}",
                rms, freq, freq_to_note(freq)
            );
            Some(midi)
        }
        None => {
            println!("Volume: {:.4} | Pitch non rilevato", rms);
            None
        }
    }
}

// ----------------------- STREAM per F32 -----------------------

fn build_stream_f32(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_rate: f32,
    state: Arc<Mutex<RecordingState>>,
) -> cpal::Stream {
    let mut buffer: Vec<f32> = Vec::with_capacity(BUFFER_SIZE * 2);

    device
        .build_input_stream(
            config,
            move |data: &[f32], _| {
                buffer.extend_from_slice(data);

                while buffer.len() >= BUFFER_SIZE {
                    let midi = process_buffer(&buffer[..BUFFER_SIZE], sample_rate);
                    state.lock().unwrap().push_raw(midi);
                    // Overlap 50%: mantieni l'ultima metà per una rilevazione più fluida
                    let keep = BUFFER_SIZE / 2;
                    buffer.drain(..BUFFER_SIZE - keep);
                }
            },
            err_fn,
            None,
        )
        .unwrap()
}

fn rms_f32(input: &[f32]) -> f32 {
    let sum: f32 = input.iter().map(|s| s * s).sum();
    (sum / input.len() as f32).sqrt()
}

// ----------------------- STREAM per I16 -----------------------

fn build_stream_i16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_rate: f32,
    state: Arc<Mutex<RecordingState>>,
) -> cpal::Stream {
    let mut buffer: Vec<f32> = Vec::with_capacity(BUFFER_SIZE * 2);

    device
        .build_input_stream(
            config,
            move |data: &[i16], _| {
                // Converti i16 → f32 normalizzato [-1, 1]
                buffer.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));

                while buffer.len() >= BUFFER_SIZE {
                    let midi = process_buffer(&buffer[..BUFFER_SIZE], sample_rate);
                    state.lock().unwrap().push_raw(midi);
                    let keep = BUFFER_SIZE / 2;
                    buffer.drain(..BUFFER_SIZE - keep);
                }
            },
            err_fn,
            None,
        )
        .unwrap()
}

// ----------------------- STREAM per U16 -----------------------

fn build_stream_u16(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_rate: f32,
    state: Arc<Mutex<RecordingState>>,
) -> cpal::Stream {
    let mut buffer: Vec<f32> = Vec::with_capacity(BUFFER_SIZE * 2);

    device
        .build_input_stream(
            config,
            move |data: &[u16], _| {
                // Converti u16 → f32 normalizzato [-1, 1] (centrato su 32768)
                buffer.extend(data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0));

                while buffer.len() >= BUFFER_SIZE {
                    let midi = process_buffer(&buffer[..BUFFER_SIZE], sample_rate);
                    state.lock().unwrap().push_raw(midi);
                    let keep = BUFFER_SIZE / 2;
                    buffer.drain(..BUFFER_SIZE - keep);
                }
            },
            err_fn,
            None,
        )
        .unwrap()
}

// ----------------------- ERROR HANDLER -----------------------

fn err_fn(err: cpal::StreamError) {
    eprintln!("Errore stream: {}", err);
}
