use crate::timecode::{Clock, MtcAssembler, Sink};
use midir::{Ignore, MidiInput, MidiInputConnection};
use selah_domain::*;
use std::sync::{Arc, Mutex};
use tauri::Emitter;

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: String,
    pub name: String,
}
pub fn devices() -> Result<Vec<Device>, String> {
    let input = MidiInput::new("Selah ports").map_err(|e| e.to_string())?;
    Ok(input
        .ports()
        .iter()
        .filter_map(|port| {
            input.port_name(port).ok().map(|name| Device {
                id: name.clone(),
                name,
            })
        })
        .collect())
}
pub fn raw(data: &[u8]) -> Option<(u8, MidiMessage, u8)> {
    if data.len() != 3 || data[1] > 127 || data[2] > 127 {
        return None;
    }
    let kind = match data[0] & 0xf0 {
        0x90 if data[2] > 0 => MidiMessage::Note,
        0xb0 => MidiMessage::ControlChange,
        _ => return None,
    };
    Some(((data[0] & 15) + 1, kind, data[1]))
}
pub fn listen(
    map: MidiMap,
    settings: TimecodeSettings,
    sink: Sink,
    clock: Clock,
    app: tauri::AppHandle,
    learn: Arc<Mutex<Option<String>>>,
) -> Result<MidiInputConnection<()>, String> {
    let mut input = MidiInput::new("Selah control").map_err(|e| e.to_string())?;
    input.ignore(Ignore::None);
    let port = input
        .ports()
        .into_iter()
        .find(|p| {
            map.device_id
                .as_ref()
                .is_none_or(|id| input.port_name(p).ok().as_ref() == Some(id))
        })
        .ok_or("MIDI input device not found")?;
    let mut assembler = MtcAssembler::default();
    input.connect(&port, "Selah commands and MTC", move |_, data, _| {
        if let Some(frame) = assembler.push(data, &settings) { clock(ChaseSource::Mtc, frame); }
        if let Some((channel,message,number)) = raw(data) {
            if let Some(binding_id) = learn.lock().unwrap().take() {
                let _ = app.emit("control://midi-learn", serde_json::json!({"bindingId":binding_id,"channel":channel,"message":message,"number":number}));
                return;
            }
            if !map.enabled { return; }
            for binding in &map.bindings {
                if binding.channel == channel && binding.message == message && binding.number == number {
                    sink(binding.command.clone(), CommandOrigin::Midi);
                }
            }
        }
    }, ()).map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn notes_and_cc() {
        assert_eq!(raw(&[0x90, 60, 0]), None);
        assert_eq!(raw(&[0x80, 60, 127]), None);
        assert_eq!(raw(&[0x9f, 60, 1]), Some((16, MidiMessage::Note, 60)));
        assert_eq!(raw(&[0xb0, 1, 0]), Some((1, MidiMessage::ControlChange, 1)));
    }
}
