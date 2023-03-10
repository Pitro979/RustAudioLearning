use lv2::prelude::*;
use wmidi::*;

#[derive(PortCollection)]
pub struct Ports{
    control:InputPort<AtomPort>,
    input:InputPort<Audio>,
    output:OutputPort<Audio>,
}

#[derive(FeatureCollection)]
pub struct Features<'a>{
    map:LV2Map<'a>,
}

#[derive(URIDCollection)]
pub struct URIDs{
    atom: AtomURIDCollection,
    midi: MidiURIDCollection,
    unit: UnitURIDCollection,
}

#[uri("https://github.com/Pitro979/RustAudioLearning/lv2/gate")]
pub struct Gate{
    n_active_notes: u64,
    program: u8,
    urids: URIDs,
}

impl Gate{
    fn write_output(&mut self, ports: &mut Ports, offset:usize,mut len:usize){
        if ports.input.len() < offset + len {
            len = ports.input.len() - offset; 
        }

        let active: bool = match self.program {
            i if i == 0 => self.n_active_notes > 0,
            _ => self.n_active_notes == 0,
        };

        let input = &ports.input[offset..offset + len];
        let output = &mut ports.output[offset..offset + len];
        
        if active {
            output.copy_from_slice(input);
        }
        else{
            for frame in output.iter_mut(){
                *frame = 0.0;
            }
        }
    }
}

impl Plugin for Gate{
    type Ports = Ports;

    type InitFeatures = Features<'static>;
    type AudioFeatures = ();

    fn new(_plugin_info: &PluginInfo, features: &mut Features<'static>) -> Option<Self>{
        Some(Self{
            n_active_notes: 0,
            program: 0,
            urids: features.map.populate_collection()?,
        })
    }

    fn run(&mut self, ports: &mut Ports, _ : &mut (), _ : u32,) {
        let mut offset:usize = 0;
        let control_sequence= ports
            .control
            .read(self.urids.atom.sequence, self.urids.unit.beat)
            .unwrap();

        for (timestamp, message) in control_sequence{
            let timestamp: usize = if let Some(timestamp) = timestamp.as_frames(){
                timestamp as usize
            } else{
                continue;
            };
            let message = if let Some(message) = message.read(self.urids.midi.wmidi,()){
                message
            }
            else{
                continue;
            };

            match message {
                MidiMessage::NoteOn(_,_,_) => self.n_active_notes += 1,
                MidiMessage::NoteOff(_,_,_) => self.n_active_notes -=1,
                MidiMessage::ProgramChange(_,program) => {
                    let program = program.into();
                    if program == 0 || program == 1 {
                        self.program = program;
                    }
                }
                _ => (),
            }
            self.write_output(ports, offset, timestamp+offset);
            offset = timestamp;
        }
        self.write_output(ports,offset,ports.input.len() - offset);
    }

    fn activate(&mut self, _features: &mut Features<'static>) {
        self.n_active_notes=0;
        self.program=0;
    }
}

lv2_descriptors!(Gate);