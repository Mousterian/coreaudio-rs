extern crate libc;
extern crate coreaudio_rs as coreaudio;

use std::env;

use coreaudio::audio_toolbox::audio_file::*;
use coreaudio::audio_unit::graph::*;
use coreaudio::error::Error;

fn play_file(filename: &String) -> Result<(),Error> {

    let audio_file_id = try!( open_audio_file(filename) );
    println!("got audio_file_id {:?} ", audio_file_id);

    let _data_format_result = try!( get_data_format(audio_file_id) );

    // TO DO: the apple playfile sample has a long complicated stringify method for the asbd, we should port it
    // and call it here

    // lets set up our playing state now
    let _graph_builder = AUGraph::new();

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