use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Numero di campioni accumulati prima di eseguire il pitch detection.
/// Con 44100 Hz → ~93 ms di audio per analisi.
const BUFFER_SIZE: usize = 4096;

/// Soglia minima di volume (RMS) sotto cui non rilevare il pitch (silenzio/rumore).
const MIN_VOLUME: f32 = 0.01;

/// Range di frequenze cercate: 80 Hz (do basso) – 1200 Hz (fischio acuto).
const MIN_FREQ: f32 = 80.0;
const MAX_FREQ: f32 = 1200.0;

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

    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream_f32(&device, &config.into(), sample_rate),
        cpal::SampleFormat::I16 => build_stream_i16(&device, &config.into(), sample_rate),
        cpal::SampleFormat::U16 => build_stream_u16(&device, &config.into(), sample_rate),
        _ => panic!("Formato audio non supportato"),
    };

    stream.play().expect("Errore avvio stream");

    println!("In ascolto... CTRL+C per uscire");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

// ----------------------- PITCH DETECTION (autocorrelazione) -----------------------

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

/// Processa un buffer f32 normalizzato: stampa volume e, se abbastanza forte, il pitch.
fn process_buffer(buffer: &[f32], sample_rate: f32) {
    let rms = rms_f32(buffer);

    if rms < MIN_VOLUME {
        // Silenzio: non analizzare
        return;
    }

    match detect_pitch(buffer, sample_rate) {
        Some(freq) => println!(
            "Volume: {:.4} | Freq: {:>7.2} Hz | Nota: {}",
            rms, freq, freq_to_note(freq)
        ),
        None => println!("Volume: {:.4} | Pitch non rilevato", rms),
    }
}

// ----------------------- STREAM per F32 -----------------------

fn build_stream_f32(device: &cpal::Device, config: &cpal::StreamConfig, sample_rate: f32) -> cpal::Stream {
    let mut buffer: Vec<f32> = Vec::with_capacity(BUFFER_SIZE * 2);

    device
        .build_input_stream(
            config,
            move |data: &[f32], _| {
                buffer.extend_from_slice(data);

                while buffer.len() >= BUFFER_SIZE {
                    process_buffer(&buffer[..BUFFER_SIZE], sample_rate);
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

fn build_stream_i16(device: &cpal::Device, config: &cpal::StreamConfig, sample_rate: f32) -> cpal::Stream {
    let mut buffer: Vec<f32> = Vec::with_capacity(BUFFER_SIZE * 2);

    device
        .build_input_stream(
            config,
            move |data: &[i16], _| {
                // Converti i16 → f32 normalizzato [-1, 1]
                buffer.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));

                while buffer.len() >= BUFFER_SIZE {
                    process_buffer(&buffer[..BUFFER_SIZE], sample_rate);
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

fn build_stream_u16(device: &cpal::Device, config: &cpal::StreamConfig, sample_rate: f32) -> cpal::Stream {
    let mut buffer: Vec<f32> = Vec::with_capacity(BUFFER_SIZE * 2);

    device
        .build_input_stream(
            config,
            move |data: &[u16], _| {
                // Converti u16 → f32 normalizzato [-1, 1] (centrato su 32768)
                buffer.extend(data.iter().map(|&s| (s as f32 - 32768.0) / 32768.0));

                while buffer.len() >= BUFFER_SIZE {
                    process_buffer(&buffer[..BUFFER_SIZE], sample_rate);
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
