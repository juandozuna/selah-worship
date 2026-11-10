//! Native adapter lifecycle and chase scheduling. Only the UI publishes live output.
use selah_application::presenter::ControlStatus;
use selah_domain::*;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use tauri::Emitter;

#[derive(Clone, serde::Serialize)]
pub struct CommandEvent {
    pub command: PresentationCommand,
    pub origin: CommandOrigin,
}
pub type Sink = Arc<dyn Fn(PresentationCommand, CommandOrigin) + Send + Sync>;
pub type Clock = Arc<dyn Fn(ChaseSource, u64) + Send + Sync>;
#[derive(Default)]
struct Core {
    pending_positions: std::collections::VecDeque<u64>,
    project: Option<Project>,
    status: ControlStatus,
    previous: Option<u64>,
    last_signal: Option<Instant>,
    midi: bool,
    osc: bool,
    ltc: bool,
    errors: Vec<String>,
    anchor: Option<(Instant, u64)>,
}
struct Worker {
    stop: mpsc::Sender<()>,
    join: thread::JoinHandle<()>,
}

pub struct Runtime {
    core: Arc<Mutex<Core>>,
    worker: Mutex<Option<Worker>>,
    sink: Sink,
    learn: Arc<Mutex<Option<String>>>,
    app: tauri::AppHandle,
}
impl Runtime {
    pub fn new(app: tauri::AppHandle) -> Self {
        let handle = app.clone();
        Self {
            core: Arc::new(Mutex::new(Core::default())),
            worker: Mutex::new(None),
            sink: Arc::new(move |command, origin| {
                let _ = handle.emit("control://command", CommandEvent { command, origin });
            }),
            learn: Arc::new(Mutex::new(None)),
            app,
        }
    }
    pub fn configure(&self, project: Project) -> Result<(), String> {
        project.validate()?;
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| "Adapter lifecycle unavailable")?;
        self.stop_locked(&mut worker);
        {
            let mut c = self.core.lock().unwrap();
            c.status = ControlStatus {
                armed_source: project.settings.timecode.armed_source,
                ..Default::default()
            };
            c.previous = None;
            c.last_signal = None;
            c.anchor = None;
            c.project = Some(project);
        }
        self.start_locked(&mut worker);
        Ok(())
    }
    pub fn update_project(&self, project: Project) -> Result<(), String> {
        project.validate()?;
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| "Adapter lifecycle unavailable")?;
        let changed = {
            let mut c = self.core.lock().unwrap();
            let changed = c.project.as_ref().is_none_or(|old| {
                old.settings.midi != project.settings.midi
                    || old.settings.osc != project.settings.osc
                    || old.settings.timecode != project.settings.timecode
            });
            if c.project.as_ref().is_none_or(|old| {
                old.settings.timecode.armed_source != project.settings.timecode.armed_source
            }) {
                c.status.armed_source = project.settings.timecode.armed_source;
                c.previous = None;
            }
            c.project = Some(project);
            changed
        };
        if changed {
            self.stop_locked(&mut worker);
            self.start_locked(&mut worker);
        }
        Ok(())
    }
    // Call only after publication succeeds, with the accepted applicator status.
    pub fn sync(&self, status: ControlStatus) -> Result<(), String> {
        if status.time_frames > MAX_TIME_FRAMES {
            return Err("Invalid control position".into());
        }
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| "Adapter lifecycle unavailable")?;
        let changed;
        {
            let mut c = self.core.lock().unwrap();
            changed = accept_status(&mut c, status);
        }
        if changed {
            self.stop_locked(&mut worker);
            self.start_locked(&mut worker);
        }
        Ok(())
    }
    pub fn learn(&self, binding_id: Option<String>) -> Result<(), String> {
        if binding_id
            .as_ref()
            .is_some_and(|id| id.is_empty() || id.len() > 500)
        {
            return Err("Invalid MIDI binding ID".into());
        }
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| "Adapter lifecycle unavailable")?;
        if self.core.lock().unwrap().project.is_none() {
            return Err("Load a project before MIDI learn".into());
        }
        *self.learn.lock().unwrap() = binding_id;
        self.stop_locked(&mut worker);
        self.start_locked(&mut worker);
        Ok(())
    }
    pub fn stop(&self) {
        if let Ok(mut w) = self.worker.lock() {
            self.stop_locked(&mut w);
        }
    }
    fn stop_locked(&self, worker: &mut Option<Worker>) {
        if let Some(w) = worker.take() {
            let _ = w.stop.send(());
            let _ = w.join.join();
        }
        let mut c = self.core.lock().unwrap();
        c.midi = false;
        c.osc = false;
        c.ltc = false;
        c.last_signal = None;
        c.pending_positions.clear();
        c.anchor = None;
    }
    fn start_locked(&self, worker: &mut Option<Worker>) {
        let (project, source) = {
            let mut c = self.core.lock().unwrap();
            c.errors.clear();
            let Some(p) = c.project.clone() else { return };
            (p, c.status.armed_source)
        };
        let core = self.core.clone();
        let sink = self.sink.clone();
        let app = self.app.clone();
        let learn = self.learn.clone();
        let (tx, rx) = mpsc::channel();
        // Keep event serialization and scheduler locks off the real-time audio callback.
        let (clock_tx, clock_rx) = mpsc::sync_channel(64);
        let clock: Clock = Arc::new(move |source, frame| {
            let _ = clock_tx.try_send((source, frame));
        });
        let join = thread::spawn(move || {
            let midi = if project.settings.midi.enabled
                || source == ChaseSource::Mtc
                || learn.lock().unwrap().is_some()
            {
                match crate::midi::listen(
                    project.settings.midi.clone(),
                    project.settings.timecode.clone(),
                    sink.clone(),
                    clock.clone(),
                    app,
                    learn,
                ) {
                    Ok(v) => {
                        core.lock().unwrap().midi = true;
                        Some(v)
                    }
                    Err(e) => {
                        core.lock().unwrap().errors.push(format!("MIDI: {e}"));
                        None
                    }
                }
            } else {
                None
            };
            let ltc = if source == ChaseSource::Ltc {
                match ltc_input(project.settings.timecode.clone(), clock, core.clone()) {
                    Ok(v) => {
                        core.lock().unwrap().ltc = true;
                        Some(v)
                    }
                    Err(e) => {
                        core.lock().unwrap().errors.push(format!("LTC: {e}"));
                        None
                    }
                }
            } else {
                None
            };
            let osc = if project.settings.osc.enabled {
                match crate::osc::bind(project.settings.osc.port) {
                    Ok(v) => {
                        core.lock().unwrap().osc = true;
                        Some(v)
                    }
                    Err(e) => {
                        core.lock().unwrap().errors.push(format!("OSC: {e}"));
                        None
                    }
                }
            } else {
                None
            };
            while matches!(
                rx.recv_timeout(Duration::from_millis(5)),
                Err(mpsc::RecvTimeoutError::Timeout)
            ) {
                for (source, frame) in clock_rx.try_iter().take(64) {
                    let commands = {
                        let mut c = core.lock().unwrap();
                        clock_commands(&mut c, source, frame)
                    };
                    for command in commands {
                        sink(command, CommandOrigin::Timeline);
                    }
                }
                if let Some(socket) = &osc {
                    if let Err(e) = crate::osc::poll(socket, &sink) {
                        let mut c = core.lock().unwrap();
                        c.osc = false;
                        if c.errors.last() != Some(&e) {
                            c.errors.push(e);
                        }
                    }
                }
                let commands = {
                    let mut c = core.lock().unwrap();
                    if c.status.armed_source == ChaseSource::None && c.status.timeline_playing {
                        let base = c.status.time_frames;
                        let (start, frame) = *c.anchor.get_or_insert((Instant::now(), base));
                        let now = frame
                            .saturating_add(
                                (start.elapsed().as_secs_f64() * project.settings.timecode.fps)
                                    as u64,
                            )
                            .min(MAX_TIME_FRAMES);
                        schedule(&mut c, now)
                    } else {
                        Vec::new()
                    }
                };
                for command in commands {
                    sink(command, CommandOrigin::Timeline);
                }
            }
            drop((midi, ltc, osc));
        });
        *worker = Some(Worker { stop: tx, join });
    }
    pub fn status(&self) -> serde_json::Value {
        let c = self.core.lock().unwrap();
        let running = if c.status.armed_source == ChaseSource::None {
            c.anchor.is_some() && c.status.timeline_playing
        } else {
            c.last_signal
                .is_some_and(|t| t.elapsed() < Duration::from_secs(1))
                && !c.status.chase_paused
        };
        serde_json::json!({"commandApiVersion":1,"midiListening":c.midi,"oscListening":c.osc,
            "mtcAvailable":c.midi,"ltcAvailable":c.ltc,"timelineClockRunning":running,
            "armedSource":c.status.armed_source,"errors":c.errors})
    }
}

/// Distinguish acknowledgements of emitted positions from user seeks. The queue can
/// lag the native clock; accepting an older clock event must not rewind its anchor.
fn accept_status(c: &mut Core, status: ControlStatus) -> bool {
    let changed = c.status.armed_source != status.armed_source;
    let clock_ack = if let Some(index) = c
        .pending_positions
        .iter()
        .position(|frame| *frame == status.time_frames)
    {
        c.pending_positions.drain(..=index);
        true
    } else {
        false
    };
    let seek = c.status.time_frames != status.time_frames && !clock_ack;
    if changed || c.status.chase_paused != status.chase_paused {
        c.previous = None;
        c.last_signal = None;
        c.pending_positions.clear();
    } else if seek {
        c.previous = Some(status.time_frames);
        c.pending_positions.clear();
    }
    if c.status.timeline_playing != status.timeline_playing || seek {
        c.anchor = Some((Instant::now(), status.time_frames));
    }
    c.status = status;
    changed
}

fn clock_commands(c: &mut Core, source: ChaseSource, frame: u64) -> Vec<PresentationCommand> {
    if source == ChaseSource::None || c.status.armed_source != source {
        return Vec::new();
    }
    c.last_signal = Some(Instant::now());
    schedule(c, frame)
}

fn schedule(c: &mut Core, frame: u64) -> Vec<PresentationCommand> {
    if frame > MAX_TIME_FRAMES {
        return Vec::new();
    }
    let previous = c.previous.replace(frame);
    if c.status.armed_source != ChaseSource::None && c.status.chase_paused {
        return Vec::new();
    }
    if previous == Some(frame) {
        return Vec::new();
    }
    if c.pending_positions.len() == 1024 {
        c.pending_positions.pop_front();
    }
    c.pending_positions.push_back(frame);
    let mut commands = vec![PresentationCommand::TimelineSeek { time_frames: frame }];
    if let Some(p) = &c.project {
        let mut cues: Vec<_> = p
            .library
            .iter()
            .flat_map(|song| &song.cues)
            .filter(|cue| match previous {
                Some(last) if frame > last => cue.time_frames > last && cue.time_frames <= frame,
                Some(_) => false,
                None => cue.time_frames == frame,
            })
            .collect();
        // Stable sorting preserves library/song cue array order for ties.
        cues.sort_by_key(|cue| cue.time_frames);
        commands.extend(cues.into_iter().map(|cue| cue.command.clone()));
    }
    commands
}

/// SMPTE labels -> absolute elapsed frames; drop-frame only removes skipped labels.
pub fn label_frames(h: u8, m: u8, s: u8, f: u8, settings: &TimecodeSettings) -> Option<u64> {
    let rate = settings.fps.ceil() as u64;
    if h >= 24 || m >= 60 || s >= 60 || u64::from(f) >= rate {
        return None;
    }
    if settings.drop_frame && m % 10 != 0 && s == 0 && f < 2 {
        return None;
    }
    let minutes = u64::from(h) * 60 + u64::from(m);
    let labels = (u64::from(h) * 3600 + u64::from(m) * 60 + u64::from(s)) * rate + u64::from(f);
    Some(
        labels
            - if settings.drop_frame {
                2 * (minutes - minutes / 10)
            } else {
                0
            },
    )
}

fn mtc_rate_matches(rate: u8, settings: &TimecodeSettings) -> bool {
    match rate {
        0 => settings.fps == 24.0,
        1 => settings.fps == 25.0,
        2 => settings.fps == 29.97 && settings.drop_frame,
        3 => [29.97, 30.0].contains(&settings.fps) && !settings.drop_frame,
        _ => false,
    }
}

#[derive(Default)]
pub struct MtcAssembler {
    nibbles: [u8; 8],
    next: u8,
}
impl MtcAssembler {
    pub fn push(&mut self, data: &[u8], settings: &TimecodeSettings) -> Option<u64> {
        // Universal real-time full-frame locate (no quarter-frame latency correction).
        if data.len() == 10 && data[..2] == [0xf0, 0x7f] && data[3..5] == [1, 1] && data[9] == 0xf7
        {
            self.next = 0;
            if !mtc_rate_matches(data[5] >> 5, settings) {
                return None;
            }
            return label_frames(data[5] & 31, data[6], data[7], data[8], settings);
        }
        if data.len() != 2 || data[0] != 0xf1 || data[1] > 127 {
            return None;
        }
        let part = data[1] >> 4;
        if part == 0 {
            self.next = 0;
        }
        if part != self.next {
            self.next = 0;
            return None;
        }
        self.nibbles[part as usize] = data[1] & 15;
        self.next += 1;
        if self.next != 8 {
            return None;
        }
        self.next = 0;
        let n = self.nibbles;
        if n[1] > 1 || n[3] > 3 || n[5] > 3 || n[7] > 7 || !mtc_rate_matches(n[7] >> 1, settings) {
            return None;
        }
        // Forward MTC takes two frames to transmit a complete set.
        label_frames(
            n[6] | ((n[7] & 1) << 4),
            n[4] | (n[5] << 4),
            n[2] | (n[3] << 4),
            n[0] | (n[1] << 4),
            settings,
        )
        .map(|v| v + 2)
    }
}

fn ltc_input(
    settings: TimecodeSettings,
    clock: Clock,
    core: Arc<Mutex<Core>>,
) -> Result<cpal::Stream, String> {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    let device = cpal::default_host()
        .default_input_device()
        .ok_or("No default audio input")?;
    let config = device.default_input_config().map_err(|e| e.to_string())?;
    let channels = usize::from(config.channels());
    let decoder =
        ltc::LTCDecoder::with_capacity(config.sample_rate().0 as f32 / settings.fps as f32, 32);
    let error = move |e: cpal::StreamError| {
        let mut c = core.lock().unwrap();
        c.ltc = false;
        c.errors.push(format!("LTC: {e}"));
    };
    fn build<T: cpal::SizedSample + cpal::Sample>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        channels: usize,
        mut decoder: ltc::LTCDecoder,
        settings: TimecodeSettings,
        clock: Clock,
        error: impl FnMut(cpal::StreamError) + Send + 'static,
    ) -> Result<cpal::Stream, String>
    where
        f32: cpal::FromSample<T>,
    {
        device
            .build_input_stream(
                config,
                move |data: &[T], _| {
                    // LTC is mono: read the first input channel, never average stereo signals.
                    let mono: Vec<f32> = data
                        .chunks_exact(channels)
                        .map(|frame| frame[0].to_sample::<f32>())
                        .collect();
                    decoder.write_samples(&mono);
                    for frame in &mut decoder {
                        if frame.drop_frame != settings.drop_frame {
                            continue;
                        }
                        if let Some(value) = label_frames(
                            frame.hour,
                            frame.minute,
                            frame.second,
                            frame.frame,
                            &settings,
                        ) {
                            clock(ChaseSource::Ltc, value);
                        }
                    }
                },
                error,
                None,
            )
            .map_err(|e| e.to_string())
    }
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build::<f32>(
            &device,
            &config.into(),
            channels,
            decoder,
            settings,
            clock,
            error,
        ),
        cpal::SampleFormat::I16 => build::<i16>(
            &device,
            &config.into(),
            channels,
            decoder,
            settings,
            clock,
            error,
        ),
        cpal::SampleFormat::U16 => build::<u16>(
            &device,
            &config.into(),
            channels,
            decoder,
            settings,
            clock,
            error,
        ),
        _ => Err("Unsupported LTC input sample format".into()),
    }?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mtc_requires_complete_ordered_cycle() {
        let settings = TimecodeSettings {
            fps: 24.0,
            ..Default::default()
        };
        let mut mtc = MtcAssembler::default();
        assert_eq!(mtc.push(&[0xf1, 0x70], &settings), None);
        for i in 0..7 {
            assert_eq!(mtc.push(&[0xf1, i << 4], &settings), None);
        }
        assert_eq!(mtc.push(&[0xf1, 0x70], &settings), Some(2));
        assert_eq!(
            mtc.push(&[0xf0, 0x7f, 0x7f, 1, 1, 0, 1, 0, 0, 0xf7], &settings),
            Some(1440)
        );
    }
    #[test]
    fn internal_playback_can_resume_while_external_chase_stays_paused() {
        let mut c = Core::default();
        c.status.chase_paused = true;
        c.status.timeline_playing = true;
        assert_eq!(schedule(&mut c, 1).len(), 1);
        assert!(c.status.chase_paused);
    }
    #[test]
    fn delayed_queue_ack_does_not_rewind_internal_clock() {
        let mut c = Core::default();
        let status = ControlStatus {
            timeline_playing: true,
            ..Default::default()
        };
        accept_status(&mut c, status.clone());
        let anchor = c.anchor;
        schedule(&mut c, 1);
        schedule(&mut c, 2);
        accept_status(
            &mut c,
            ControlStatus {
                time_frames: 1,
                ..status.clone()
            },
        );
        assert_eq!(c.anchor, anchor);
        accept_status(
            &mut c,
            ControlStatus {
                time_frames: 100,
                ..status
            },
        );
        assert_eq!(c.anchor.unwrap().1, 100);
        assert_eq!(c.previous, Some(100));
    }
    #[test]
    fn source_is_exclusive_and_pause_does_not_apply_manual_commands() {
        let mut c = Core::default();
        c.status.armed_source = ChaseSource::Mtc;
        assert!(clock_commands(&mut c, ChaseSource::Ltc, 10).is_empty());
        assert_eq!(c.previous, None);
        assert_eq!(clock_commands(&mut c, ChaseSource::Mtc, 10).len(), 1);
        c.status.chase_paused = true;
        assert!(clock_commands(&mut c, ChaseSource::Mtc, 11).is_empty());
        assert!(c.status.chase_paused);
    }
    #[test]
    fn mtc_supports_project_rates_and_rejects_mismatch() {
        for (fps, drop_frame, rate) in [
            (24.0, false, 0),
            (25.0, false, 1),
            (29.97, true, 2),
            (29.97, false, 3),
            (30.0, false, 3),
        ] {
            let settings = TimecodeSettings {
                fps,
                drop_frame,
                ..Default::default()
            };
            let mut decoder = MtcAssembler::default();
            assert_eq!(
                decoder.push(
                    &[0xf0, 0x7f, 0x7f, 1, 1, rate << 5, 0, 1, 0, 0xf7],
                    &settings
                ),
                Some(fps.ceil() as u64)
            );
        }
        assert_eq!(
            MtcAssembler::default().push(
                &[0xf0, 0x7f, 0x7f, 1, 1, 0, 0, 0, 0, 0xf7],
                &TimecodeSettings::default()
            ),
            None
        );
    }
    #[test]
    fn synthetic_ltc_audio_decodes_to_elapsed_frames() {
        // 48 kHz / 30 fps / 80 bits = 20 samples per bit, biphase-mark encoded.
        let word = (0xbffcu128 << 64) | (1 << 16); // 00:00:01:00, forward sync
        let mut audio = Vec::new();
        let mut level = 0.8f32;
        for _ in 0..12 {
            for bit in 0..80 {
                level = -level;
                audio.extend(std::iter::repeat_n(level, 10));
                if word & (1 << bit) != 0 {
                    level = -level;
                }
                audio.extend(std::iter::repeat_n(level, 10));
            }
        }
        let mut decoder = ltc::LTCDecoder::with_capacity(1600.0, 16);
        decoder.write_samples(&audio);
        let frames: Vec<_> = (&mut decoder).into_iter().collect();
        assert!(!frames.is_empty());
        for f in frames {
            assert_eq!(
                label_frames(
                    f.hour,
                    f.minute,
                    f.second,
                    f.frame,
                    &TimecodeSettings::default()
                ),
                Some(30)
            );
        }
    }
    #[test]
    fn drop_frame_is_label_conversion() {
        let s = TimecodeSettings {
            fps: 29.97,
            drop_frame: true,
            ..Default::default()
        };
        assert_eq!(label_frames(0, 1, 0, 0, &s), None);
        assert_eq!(label_frames(0, 1, 0, 2, &s), Some(1800));
        assert_eq!(label_frames(0, 10, 0, 0, &s), Some(17982));
        assert_eq!(label_frames(1, 0, 0, 0, &s), Some(107892));
    }
    #[test]
    fn cues_cross_frames_preserve_ties_and_rearm_after_rewind() {
        let mut p: Project =
            serde_json::from_str(include_str!("../../tests/fixtures/control-project.json"))
                .unwrap();
        p.library[0].cues = vec![
            TimelineCue {
                id: "a".into(),
                time_frames: 2,
                command: PresentationCommand::Next,
            },
            TimelineCue {
                id: "b".into(),
                time_frames: 2,
                command: PresentationCommand::Clear,
            },
        ];
        let mut c = Core {
            project: Some(p),
            ..Default::default()
        };
        schedule(&mut c, 0);
        assert_eq!(schedule(&mut c, 3).len(), 3);
        assert!(schedule(&mut c, 3).is_empty());
        schedule(&mut c, 0);
        assert_eq!(schedule(&mut c, 2).len(), 3);
        c.status.armed_source = ChaseSource::Mtc;
        c.status.chase_paused = true;
        assert!(schedule(&mut c, 4).is_empty());
    }
}
