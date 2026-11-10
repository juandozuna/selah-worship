import type { CommandIntent, ControlStatus, Live, MidiMap, MidiBinding } from './types';

/** Keep preparation, publication, acceptance and native acknowledgement in one queue slot. */
export function queueControlCommand(
  previous: Promise<void>,
  prepare: () => Promise<CommandIntent>,
  publish: (live: Live) => Promise<void>,
  accept: (intent: CommandIntent) => void,
  sync: (status: ControlStatus) => Promise<void>,
): Promise<void> {
  return previous.catch(() => {}).then(async () => {
    const intent = await prepare();
    if (intent.live) await publish(intent.live);
    accept(intent);
    await sync(intent.status);
  });
}

export type MidiLearnEvent = Pick<MidiBinding, 'channel' | 'message' | 'number'> & { bindingId: string };
/** Ignore stale captures and reject malformed/duplicate inputs before autosave. */
export function applyMidiLearn(map: MidiMap, event: MidiLearnEvent, learning: string | null): MidiMap {
  if (event.bindingId !== learning || !map.bindings.some(b => b.id === event.bindingId)) return map;
  if (!Number.isInteger(event.channel) || event.channel < 1 || event.channel > 16 ||
      !['note', 'controlChange'].includes(event.message) || !Number.isInteger(event.number) || event.number < 0 || event.number > 127) {
    throw new Error('Invalid MIDI learn input.');
  }
  if (map.bindings.some(b => b.id !== event.bindingId && b.channel === event.channel && b.message === event.message && b.number === event.number)) {
    throw new Error('This MIDI input is already assigned to another row.');
  }
  return {...map, bindings: map.bindings.map(b => b.id === event.bindingId
    ? {...b, channel: event.channel, message: event.message, number: event.number} : b)};
}
