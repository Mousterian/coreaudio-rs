use bindings::core_audio as ca;
use error::{Error, AudioUnitError};
use std::mem;

pub struct AUGraph {
	graph: ca::AUGraph
}

impl AUGraph {

	/// Construct a new empty AUGraph. Use the returned AUGraphBuilder to add AudioUnits to it.
	pub fn new() -> AUGraphBuilder {
		unsafe {
			let mut graph = mem::uninitialized();
			let graph_result = match Error::from_os_status(ca::NewAUGraph (&mut graph as *mut ca::AUGraph)) {
				Ok(()) => {
					println!("Created a new empty AUGraph.");
					Ok( AUGraph{ graph: graph } )
				}
				Err(err) => Err(err)
			};
			AUGraphBuilder{ graph_result: graph_result }
		}

	}

}

pub struct AUGraphBuilder {
	graph_result: Result<AUGraph, Error>
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