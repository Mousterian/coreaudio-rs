extern crate libc;
extern crate coreaudio_rs as coreaudio;

use std::env;

use coreaudio::audio_toolbox::audio_file::*;
use coreaudio::audio_unit::graph::*;
use coreaudio::audio_unit::{Type, SubType, Manufacturer};
use coreaudio::error::Error;

fn play_file(filename: &String) -> Result<(),Error> {

    let audio_file_id = try!( open_audio_file(filename) );
    println!("got audio_file_id {:?} ", audio_file_id);

    let data_format = try!( get_data_format(audio_file_id) );
	println!("got data_format {:?} ", data_format);

    // TO DO: the apple playfile sample has a long complicated stringify method for the asbd, perhaps we should port it
    // and call it here?

    let graph = try!(AUGraph::new());

	let default_output_node = try!(graph.add_node(Type::Output, SubType::DefaultOutput, Manufacturer::Apple));

	let file_player_node = try!(graph.add_node(Type::Generator, SubType::AudioFilePlayer, Manufacturer::Apple));

	try!(graph.open());

	let audio_unit = try!(graph.node_info(file_player_node));

	// prepare the file AU for playback

	// set its output channels


    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: playfile <path to audio file>");
    }
    else {
        println!("Using filename: {}", args[1]);

        let result = play_file(&args[1]);

        match result {
                Ok(_) => {
                println!("Everything is ok.");
            },
                Err(err) => {
                panic!("Could not play file, error: {:?}", err);
            }
        }
    }
	()
}
