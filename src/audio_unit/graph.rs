use bindings::core_audio as ca;
use super::*;
use error::{Error};
use std::mem;
use std::ptr;

// TO DO: couldn't seem to pick up the macro definition from mod.rs in this file
// so duplicating for the time being, something to do with macro export.
macro_rules! try_os_status {
($expr:expr) => (try!(Error::from_os_status($expr)))
}

pub struct AUNode {
    instance: ca::AUNode
}

impl AUNode {
    pub fn new(node: ca::AUNode) -> AUNode {
        AUNode{ instance: node }
    }
}

pub struct AUGraph {
    instance: ca::AUGraph
}

impl AUGraph {

    /// Construct a new empty AUGraph.
    pub fn new() -> Result<AUGraph,Error> {
        unsafe {
            let mut graph = mem::uninitialized();
            match Error::from_os_status(ca::NewAUGraph (&mut graph as *mut ca::AUGraph)) {
                Ok(()) => {
                    Ok( AUGraph{ instance: graph } )
                }
                Err(err) => Err(err)
            }
        }

    }

    /// wraps AUGraphAddNode
    pub fn add_node(&self, ac_type : Type, ac_sub_type: SubType, ac_manufacturer: Manufacturer) -> Result<AUNode,Error> {
        unsafe {
            let description = ca::AudioComponentDescription { 	componentType: ac_type as u32,
                                                                componentSubType: ac_sub_type as u32,
                                                                componentManufacturer: ac_manufacturer as u32,
                                                                /* TO DO: figure out does anybody actually use these? */
                                                                componentFlags: 0,
                                                                componentFlagsMask: 0 };
            let mut node: ca::AUNode = mem::uninitialized();
            match Error::from_os_status(ca::AUGraphAddNode(self.instance,
                                        &description as *const ca::AudioComponentDescription,
                                        &mut node as *mut ca::AUNode)) {
                Ok(()) => Ok(AUNode::new(node)),
                Err(e) => Err(e)
            }
        }
    }

    /// Finish building the audio unit graph, and open it, wraps AUGraphOpen
    pub fn open(&self) -> Result<(), Error> {
        unsafe {
            try_os_status!(ca::AUGraphOpen(&mut *self.instance as ca::AUGraph));
            Ok(())
        }
    }

    /// wraps AUGraphNodeInfo
    pub fn node_info(&self, node: AUNode) -> Result<AudioUnit, Error> {
        unsafe {
            let description: *mut ca::AudioComponentDescription = ptr::null_mut();
            let mut audio_unit : ca::AudioUnit = mem::uninitialized();


            match Error::from_os_status(ca::AUGraphNodeInfo(self.instance, node.instance, description, &mut audio_unit)) {
                Ok(()) => {
                    // we cannot construct a wrapper object via AudioUnit::new() here as it would double initialise and
                    // cause a panic. AudioUnit::from_graph wraps it but does not try to initialise or drop it.
                    Ok(AudioUnit::from_graph(audio_unit))
                }
                Err(e) => Err(e)
            }
        }
    }
}

impl Drop for AUGraph {
    fn drop(&mut self) {
        unsafe {
            use error;
            use std::error::Error;

            if let Err(err) = error::Error::from_os_status(ca::AUGraphStop(self.instance)) {
                panic!("{:?}", err.description());
            }
            if let Err(err) = error::Error::from_os_status(ca::AUGraphUninitialize(self.instance)) {
                panic!("{:?}", err.description());
            }
            if let Err(err) = error::Error::from_os_status(ca::AUGraphClose(self.instance)) {
                panic!("{:?}", err.description());
            }
        }
    }
}

