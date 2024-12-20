mod voice;
use voice::{EventType, VoiceScheduler};
use voxbox::*;

mod chords;
use chords::{ChordManager, SelectionHeuristic};

struct SmoothParam {
    pub value: f32,
    pub smoother: Smoother,
}

struct VoiceWithSmoother {
    pub voice: Voice,
    pub pitch: SmoothParam,
    pub gain: SmoothParam,
    pub gest: EventfulGesture,
}

const SHAPE1: [f32; 8] = [1.011, 0.201, 0.487, 0.440, 1.297, 2.368, 1.059, 2.225];
//const SHAPE2: [f32; 8] = [1.011, 0.201, 0.487, 0.440, 1.297, 0.368, 1.059, 1.225];
//const SHAPE2: [f32; 8] = [0.502, 2.807, 0.105, 0.431, 3.005, 0.784, 0.360, 2.326];
const SHAPE2: [f32; 8] = [1.896, 1.103, 0.066, 0.116, 2.723, 1.969, 0.826, 1.269];
impl VoiceWithSmoother {
    pub fn new(sr: usize, tract_len_cm: f32, oversample: u16) -> Self {
        let shape1 = [1.011, 0.201, 0.487, 0.440, 1.297, 2.368, 1.059, 2.225];
        //let shape1 = [1.011, 0.201, 0.487, 0.440, 1.297, 0.368, 1.059, 2.225];
        let mut v = VoiceWithSmoother {
            voice: Voice::new(sr, tract_len_cm, oversample),
            pitch: SmoothParam::new(sr, 60.),
            gain: SmoothParam::new(sr, 0.0),
            gest: EventfulGesture::default(),
        };

        v.gain.smoother.set_smooth(0.04);
        v.pitch.smoother.set_smooth(0.04);
        v.voice.pitch = 60.;
        v.voice.tract.drm(&shape1);
        v.voice.vibrato_depth(0.3);
        v.voice.vibrato_rate(6.1);
        v.voice.glottis.set_aspiration(0.1);

        v
    }

    pub fn tick(&mut self) -> f32 {
        self.voice.pitch = self.pitch.tick();
        let gain = self.gain.tick();

        self.voice.tick() * 0.7 * gain
    }

    pub fn tick_with_gesture(&mut self, clk: f32) -> f32 {
        self.voice.pitch = self.gest.tick(clk);
        let gain = self.gain.tick();

        self.voice.tick() * 0.7 * gain
    }

    pub fn reset(&mut self) {
        self.pitch.reset();
        self.gest.immediate(self.pitch.value);
    }

    #[allow(dead_code)]
    pub fn set_pitch(&mut self, pitch: f32) {
        self.pitch.value = pitch;
    }

    pub fn schedule_pitch(&mut self, pitch: f32) {
        //println!("scheduling pitch {pitch}");
        self.pitch.value = pitch;
        self.gest.clear();
        self.gest.scalar(pitch);
        self.gest.behavior(Behavior::GlissHuge);
    }
}

#[repr(C)]
pub struct VoxData {
    testvar: f32,
    lead: VoiceWithSmoother,
    lower: VoiceWithSmoother,
    upper: VoiceWithSmoother,
    reverb: BigVerb,
    delay: Delay,
    dcblk: DCBlocker,

    is_running: bool,
    pub x_axis: f32,
    px_axis: f32,
    pub y_axis: f32,
    y_axis_smooth: Smoother,
    py_axis: f32,
    pitches: Vec<i16>,
    base: u16,
    upper_lookup: [i16; 7],
    lower_lookup: [i16; 7],

    clk: Phasor,
    lphs: f32,
    last_pitch: f32,
    time: u32,
    pitch_changed: bool,
    pitch_last_changed: u32,
    cached_lead_pitch: f32,
    lower_has_changed: bool,
    lead_playing: bool,
    please_reset: bool,

    voice_manager: VoiceScheduler,
    chord_manager: ChordManager,
    shape: [f32; 8],
}

//0  1  2  3  4  5  6  7  8  9 10  11
//0  0  1  1  2  3  3  4  4  5  5  6
//do di re ri mi fa fi so si la te ti
// const PITCH_TO_DIATONIC: [i16; 12] = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6];

impl VoxData {
    pub fn new(sr: usize) -> Self {
        let tract_len_cm = 14.0;
        let oversample = 2;
        let mut vd = VoxData {
            testvar: 12345.0,
            lead: VoiceWithSmoother::new(sr, tract_len_cm, oversample),
            lower: VoiceWithSmoother::new(sr, tract_len_cm * 1.1, oversample),
            upper: VoiceWithSmoother::new(sr, tract_len_cm * 0.99, oversample),
            is_running: true,
            reverb: BigVerb::new(sr),
            dcblk: DCBlocker::new(sr),
            x_axis: 0.,
            px_axis: -1.,
            pitches: vec![0, 2, 4, 5, 7, 9, 11, 12, 14, 16, 17, 19, 21],
            base: 60,
            lower_lookup: [-5, -3, -1, 0, 2, 4, 5],
            upper_lookup: [4, 5, 7, 9, 11, 12, 14],
            clk: Phasor::new(sr, 0.),
            lphs: -1.,
            cached_lead_pitch: -1.,
            last_pitch: -1.,
            time: 0,
            pitch_last_changed: 0,
            pitch_changed: false,
            lower_has_changed: false,
            lead_playing: true,
            please_reset: false,
            voice_manager: VoiceScheduler::default(),
            chord_manager: ChordManager::default(),
            delay: Delay::new(sr, 0.2),
            y_axis: 0.,
            py_axis: 0.,
            shape: [0.; 8],
            y_axis_smooth: Smoother::new(sr),
        };

        vd.reverb.size = 0.91;
        vd.voice_manager.populate_hooks();
        vd.clk.set_freq(1.0);
        vd.upper.voice.vibrato_depth(0.1);
        vd.upper.voice.vibrato_rate(6.2);
        vd.upper.gain.smoother.set_smooth(0.08);
        vd.upper.pitch.smoother.set_smooth(0.08);

        vd.lower.voice.vibrato_depth(0.1);
        vd.lower.voice.vibrato_rate(6.0);
        vd.lower.gain.smoother.set_smooth(0.09);
        vd.lower.pitch.smoother.set_smooth(0.09);

        vd.chord_manager.populate();
        //vd.chord_manager.chord_behavior = SelectionHeuristic::LeastMovement;
        //vd.chord_manager.chord_behavior = SelectionHeuristic::LeastUsed;
        vd.chord_manager.chord_behavior = SelectionHeuristic::LazyLeastUsed;

        vd.y_axis_smooth.set_smooth(0.01);
        vd
    }

    fn check_for_pitch_changes(&mut self) {
        let pitch = (self.x_axis * self.pitches.len() as f32) as usize;
        let pitch = pitch.clamp(0, self.pitches.len() - 1);
        let pitch = 60.0 + self.pitches[pitch] as f32;

        let did_pitch_change = self.last_pitch > 0. && self.last_pitch != pitch;

        if did_pitch_change {
            //println!("changed {} -> {} at {}", self.last_pitch, pitch, self.time);
            self.last_pitch = pitch;
            self.pitch_last_changed = self.time;
            self.voice_manager.change(pitch as u32);
            // tell chord manager to change chords
            //self.chord_manager.change(pitch as u16);
        }

        // instantaneous update of lead pitch
        self.lead.pitch.value = pitch;

        if self.please_reset {
            //println!("resetting THE LEAD PITCH");
            self.lead.reset();
            //self.chord_manager.change(pitch as u16);
        }

        // if self.please_reset || did_pitch_change {
        //     self.chord_manager.change(pitch as u16);
        // }
    }

    // TODO: I know, a bit repetitive...
    pub fn upper_pitch_lookup(&self, _pitch: f32) -> f32 {
        //let (idx, octave) = self.get_scale_degree(pitch as u16, self.base);
        //self.base as f32 + 12.0 * octave + self.upper_lookup[idx] as f32

        // This should be retrieved from the voice state manager
        self.chord_manager.find_upper_pitch() as f32
    }

    pub fn lower_pitch_lookup(&self, _pitch: f32) -> f32 {
        //let (idx, octave) = self.get_scale_degree(pitch as u16, self.base);
        //self.base as f32 + 12.0 * octave + self.lower_lookup[idx] as f32

        // This should be retrieved from the voice state manager
        self.chord_manager.find_lower_pitch() as f32
    }

    pub fn tick(&mut self) -> f32 {
        if self.please_reset {
            //println!("RESETTING");
            self.clk.reset();
            self.last_pitch = -1.;
            self.time = 0;
            self.pitch_last_changed = 0;
            self.lower_has_changed = false;
            //self.lower.reset();
            //self.lead.reset();
            //self.upper.reset();
        }

        let clk = self.clk.tick();

        if self.lphs >= 0. && self.lphs > clk {
            self.voice_manager.tick();
            println!("tick {}", self.voice_manager.state.time);
            while let Some(evt) = self.voice_manager.pop_next_event() {
                match evt {
                    EventType::LowerOn => {
                        println!("Lower on!");
                        self.lower.gain.value = 0.8;
                        let pitch = self.lead.pitch.value;
                        self.chord_manager.change(pitch as u16);
                        // let (idx, octave) = self.get_scale_degree(pitch as u16, self.base);
                        // let pitch =
                        //     self.base as f32 + 12.0 * octave + self.lower_lookup[idx] as f32;
                        let pitch = self.lower_pitch_lookup(pitch);
                        self.lower.schedule_pitch(pitch);
                        // cache lower pitch in chord manager
                        self.chord_manager.cache_lower(pitch as u16);
                        self.lower.reset();
                    }
                    EventType::LowerChange => {
                        println!("Lower change!");
                        self.lower.gain.value = 0.8;
                        let pitch = self.lead.pitch.value;
                        self.chord_manager.change(pitch as u16);
                        //let (idx, octave) = self.get_scale_degree(pitch as u16, self.base);
                        //let pitch =
                        //    self.base as f32 + 12.0 * octave + self.lower_lookup[idx] as f32;
                        let pitch = self.lower_pitch_lookup(pitch);
                        self.lower.schedule_pitch(pitch);
                        // cache lower pitch in chord manager
                        self.chord_manager.cache_lower(pitch as u16);
                    }
                    EventType::UpperOn => {
                        println!("Upper On!");
                        self.upper.gain.value = 0.8;
                        let pitch = self.lead.pitch.value;
                        //let (idx, octave) = self.get_scale_degree(pitch as u16, self.base);
                        //let pitch =
                        //    self.base as f32 + 12.0 * octave + self.upper_lookup[idx] as f32;
                        let pitch = self.upper_pitch_lookup(pitch);
                        self.upper.schedule_pitch(pitch);
                        // cache upper pitch in chord manager
                        self.chord_manager.cache_upper(pitch as u16);
                        self.upper.reset();
                    }
                    EventType::UpperChange => {
                        println!("Upper Change!");
                        self.upper.gain.value = 0.8;
                        let pitch = self.lead.pitch.value;
                        let pitch = self.upper_pitch_lookup(pitch);
                        self.upper.schedule_pitch(pitch);
                        // cache upper pitch in chord manager
                        self.chord_manager.cache_upper(pitch as u16);
                    }
                }
            }
            self.last_pitch = self.lead.pitch.value;
        }

        if self.x_axis != self.px_axis {
            self.px_axis = self.x_axis;
            self.check_for_pitch_changes();
        }

        self.please_reset = false;
        self.lphs = clk;

        let y_axis = self.y_axis;

        let y_axis = self.y_axis_smooth.tick(y_axis);

        // some exponential scaling for vibrato
        // an attempt to "save" the vibrato for the upper region
        let expmap = (1.0 - (3. * y_axis).exp()) / (1.0 - (3f32).exp());
        self.lead.voice.vibrato_depth(0.1 + 0.8 * expmap);
        self.lead.voice.vibrato_rate(6.01 + 0.4 * expmap);

        self.lower.voice.vibrato_depth(0.1 + 0.4 * expmap);
        self.lower.voice.vibrato_rate(6.1 + 0.4 * expmap);
        self.upper.voice.vibrato_depth(0.1 + 0.4 * expmap);
        self.upper.voice.vibrato_rate(5.97 + 0.4 * expmap);

        for i in 0..8 {
            self.shape[i] = y_axis * SHAPE1[i] + (1.0 - y_axis) * SHAPE2[i];
        }

        self.lead.voice.tract.drm(&self.shape);
        self.upper.voice.tract.drm(&self.shape);
        self.lower.voice.tract.drm(&self.shape);

        self.lead.voice.glottis.set_shape(y_axis * 0.2 + 0.3);
        self.upper.voice.glottis.set_shape(y_axis * 0.2 + 0.3);
        self.lower.voice.glottis.set_shape(y_axis * 0.2 + 0.3);

        let gain = db2lin(0. - 1. * (1. - y_axis));
        let lead = self.lead.tick() * gain;
        let lower = self.lower.tick_with_gesture(clk) * gain;
        let upper = self.upper.tick_with_gesture(clk) * gain;
        //let lower = self.lower.tick();
        //let upper = self.upper.tick();

        let voices = (lead + lower * 0.8 + upper * 0.8) * 0.33;
        let voices_send = voices;
        let (r, _) = self.reverb.tick(voices_send, voices_send);
        let r = self.dcblk.tick(r);
        let del = self.delay.tick(r);

        (voices + del * 0.1) * 0.6
    }

    pub fn running(&mut self) -> u8 {
        let mut state = 1;

        if !self.is_running {
            state = 0;
        }

        state
    }

    pub fn process(&mut self, outbuf: *mut f32, sz: usize) {
        let outbuf: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(outbuf, sz) };
        for n in 0..sz {
            outbuf[n] = self.tick();
        }
    }
}

fn db2lin(db: f32) -> f32 {
    10_f32.powf(db / 20.0)
}

impl Drop for VoxData {
    fn drop(&mut self) {
        println!("dropping");
    }
}

impl SmoothParam {
    pub fn new(sr: usize, ival: f32) -> Self {
        let mut sp = SmoothParam {
            value: ival,
            smoother: Smoother::new(sr),
        };
        sp.smoother.set_smooth(0.07);
        sp
    }

    pub fn tick(&mut self) -> f32 {
        self.smoother.tick(self.value)
    }

    pub fn reset(&mut self) {
        self.smoother.snap_to_value(self.value);
    }
}

#[no_mangle]
pub extern "C" fn newdsp(sr: u32) -> Box<VoxData> {
    let sr = sr as usize;
    println!("samplerate: {}", sr);
    Box::new(VoxData::new(sr))
}

#[no_mangle]
pub extern "C" fn testgetter(vd: &mut VoxData) -> f32 {
    vd.testvar
}

#[no_mangle]
pub extern "C" fn vox_tick(vd: &mut VoxData) -> f32 {
    vd.tick()
}

#[no_mangle]
pub extern "C" fn vox_poll(_vd: &mut VoxData) {
    // vd.poll();
}

#[no_mangle]
pub extern "C" fn vox_running(vd: &mut VoxData) -> u8 {
    vd.running()
}

#[no_mangle]
pub extern "C" fn vox_free(vd: &mut VoxData) {
    let ptr = unsafe { Box::from_raw(vd) };
    drop(ptr);
}

#[no_mangle]
pub extern "C" fn vox_pitch(vd: &mut VoxData, pitch: f32) {
    vd.lead.pitch.value = pitch;
}

#[no_mangle]
pub extern "C" fn vox_gate(vd: &mut VoxData, gate: f32) {
    if gate == 0.0 {
        vd.voice_manager.off();
        vd.lead.gain.value = 0.;
        vd.upper.gain.value = 0.;
        vd.lower.gain.value = 0.;
        vd.lead_playing = false;
    } else {
        vd.voice_manager.on();
        vd.please_reset = true;
        vd.lead.gain.value = gate * 0.8;
        vd.lead.reset();
        vd.lead_playing = true;
    }
}

#[no_mangle]
pub extern "C" fn vox_tongue_shape(vd: &mut VoxData, x: f32, y: f32) {
    vd.lead.voice.tract.tongue_shape(x, y);
}

#[no_mangle]
pub extern "C" fn vox_x_axis(vd: &mut VoxData, x: f32) {
    vd.x_axis = x;
}

#[no_mangle]
pub extern "C" fn vox_y_axis(vd: &mut VoxData, y: f32) {
    vd.y_axis = y;
}

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut f32 {
    let mut buf = Vec::<f32>::with_capacity(size);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as *mut f32
}

#[no_mangle]
pub extern "C" fn process(vd: &mut VoxData, outbuf: *mut f32, sz: usize) {
    //let outbuf: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(unsafe { outbuf }, sz) };

    //for n in 0..sz {
    //    outbuf[n] = vd.tick();
    //}

    // for smp in outbuf.iter_mut().take(sz) {
    //     *smp = vd.tick();
    // }
    vd.process(outbuf, sz);
}
