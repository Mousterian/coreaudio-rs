use bindings::core_audio as ca;
use super::{Type, SubType, Manufacturer};
use error::{Error};
use std::mem;

pub struct AUGraph {
	graph: ca::AUGraph
}

impl AUGraph {

	/// Construct a new empty AUGraph. Use the returned AUGraphBuilder to add AudioUnits to it.
	pub fn new() -> AUGraphBuilder {
		unsafe {
			let mut graph = mem::uninitialized();
			let result = match Error::from_os_status(ca::NewAUGraph (&mut graph as *mut ca::AUGraph)) {
				Ok(()) => {
					println!("Created a new empty AUGraph.");
					Ok( AUGraph{ graph: graph } )
				}
				Err(err) => Err(err)
			};
			AUGraphBuilder{ graph_result: result }
		}

	}

}

impl Drop for AUGraph {
	fn drop(&mut self) {
		println!("In Drop for AUGraph");
		unsafe {
			use error;
			use std::error::Error;

			if let Err(err) = error::Error::from_os_status(ca::AUGraphStop(self.graph)) {
				panic!("{:?}", err.description());
			}
			if let Err(err) = error::Error::from_os_status(ca::AUGraphUninitialize(self.graph)) {
				panic!("{:?}", err.description());
			}
			if let Err(err) = error::Error::from_os_status(ca::AUGraphClose(self.graph)) {
				panic!("{:?}", err.description());
			}
		}
	}
}

pub struct AUGraphBuilder {
	graph_result: Result<AUGraph, Error>
}

impl AUGraphBuilder {
	pub fn add_node(self, ac_type : Type, ac_sub_type: SubType, ac_manufacturer: Manufacturer) -> AUGraphBuilder {
		let graph_result = match self.graph_result {
			Ok(au_graph) => {
				unsafe {
					let description = ca::AudioComponentDescription { 	componentType: ac_type as u32,
																		componentSubType: ac_sub_type as u32,
																		componentManufacturer: ac_manufacturer as u32,
																		/* TO DO: figure out does anybody actually use these? */
																		componentFlags: 0,
																		componentFlagsMask: 0 };
					let mut node: ca::AUNode = mem::uninitialized();
					match Error::from_os_status(ca::AUGraphAddNode(au_graph.graph,
																	&description as *const ca::AudioComponentDescription,
																	&mut node as *mut ca::AUNode)) {
						// store node somewhere ?
						Ok(()) => Ok(au_graph),
						Err(e) => Err(e)
					}
				}
			},
			Err(e) => Err(e)
		};
		AUGraphBuilder { graph_result: graph_result }
	}

	/// Finish building the audio unit graph, and open it.
	pub fn open(self) -> Result<AUGraph, Error> {
		let au_graph = try!(self.graph_result);
		unsafe {
			try!(Error::from_os_status(ca::AUGraphOpen(&mut *au_graph.graph as ca::AUGraph)));
			Ok(au_graph)
		}
	}
}
