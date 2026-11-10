export type TransitionSpec = { kind: string; durationMs: number; easing: string; direction: string; feather: number };
export type TextAnimSpec = { kind: string; unit: string; durationMs: number; staggerMs: number; easing: string; direction: string; order: string };
export type Motion = { transition: TransitionSpec; text: TextAnimSpec };
export type MotionOverride = { transition?: TransitionSpec | null; text?: TextAnimSpec | null };
export type Block = { id: string; section: string; primary: string; secondary: string | null; reference: string | null; motion?: MotionOverride | null };
export type Song = { cues?: TimelineCue[]; id: string; title: string; artist: string; language: string; kind: string; source: { name: string; url: string | null; license: string }; blocks: Block[]; motion?: MotionOverride | null };
export type Region = { id: string; name: string; x: number; y: number; width: number; height: number; fontSize: number; fontFamily: string; color: string; align: 'left'|'center'|'right'; binding: 'primary'|'secondary'|'reference'; outline: number; shadow: boolean; anim?: string | null };
export type MediaKind = 'image' | 'video';
export type MediaRole = 'background' | 'foreground';
export type MediaFit = 'cover' | 'contain' | 'fill';
export type MediaAsset = { id: string; name: string; path: string; kind: MediaKind; mime: string };
export type MediaLayer = { id: string; name: string; role: MediaRole; kind: MediaKind; assetId: string; x: number; y: number; width: number; height: number; opacity: number; muted: boolean; audioDeviceId?: string | null; fit?: MediaFit };
export type Layout = { width: number; height: number; regions: Region[]; mediaLayers?: MediaLayer[] };
export type Settings = { midi?: MidiMap; osc?: OscSettings; timecode?: TimecodeSettings; language: 'en'|'es'; feedName: string; aiProvider: string; aiModel: string; aiEndpoint: string; localServerPath: string; localModelPath: string };
export type DisplayOutput = { monitor: number; background: string; layout: Layout };
export type DisplayInfo = { index: number; name: string; x: number; y: number; width: number; height: number; scale: number; primary: boolean };
export type DisplayOverview = { displays: DisplayInfo[]; open: boolean };
export type Project = { fonts?: FontAsset[]; version: number; name: string; library: Song[]; service: string[]; layout: Layout; settings: Settings; motion?: Motion; display?: DisplayOutput | null; media?: MediaAsset[]; streamComposite?: boolean };
export type Live = { songId: string|null; blockIndex: number|null; block: Block|null; layout: Layout; testPattern: boolean; aiTranslation?: boolean; motion?: Motion | null; media?: MediaAsset[]; streamComposite?: boolean };
export type Candidate = { title: string; url: string; description: string };
export type OutputStatus = { protocol: string; sending: boolean; receivers: number|null; error: string|null };
export type Verse = { translation: string; book: string; chapter: number; verse: number; text: string };
export type AudioDevice = { id: string; label: string };
declare global { interface Window { __TAURI_INTERNALS__?: { invoke: <T>(command: string,args?:Record<string,unknown>)=>Promise<T> } } }

export type ChaseSource = 'none' | 'mtc' | 'ltc';
export type CommandOrigin = 'keyboard' | 'midi' | 'osc' | 'timeline';
export type PresentationCommand =
  | { type: 'next' | 'prev' | 'clear' | 'chaseResync' | 'timelinePlay' | 'timelinePause' }
  | { type: 'go'; songId: string; blockIndex: number }
  | { type: 'chaseArm'; source: ChaseSource }
  | { type: 'timelineSeek'; timeFrames: number };
export type MidiBinding = { id: string; channel: number; message: 'note' | 'controlChange'; number: number; command: PresentationCommand };
export type MidiMap = { enabled: boolean; deviceId: string | null; bindings: MidiBinding[] };
export type OscSettings = { enabled: boolean; port: number };
export type TimecodeSettings = { fps: 24 | 25 | 29.97 | 30; dropFrame: boolean; armedSource: ChaseSource; pauseOnManual: boolean };
export type TimelineCue = { id: string; timeFrames: number; command: PresentationCommand };
export type FontAsset = { id: string; family: string; relativePath: string };
export type ControlStatus = { armedSource: ChaseSource; chasePaused: boolean; timelinePlaying: boolean; timeFrames: number };
export type CommandIntent = { live: Live | null; status: ControlStatus };
export type ControlCapabilities = { commandApiVersion: number; midiListening: boolean; oscListening: boolean; mtcAvailable: boolean; ltcAvailable: boolean; timelineClockRunning: boolean };
