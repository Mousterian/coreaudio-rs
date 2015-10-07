extern crate libc;
extern crate coreaudio_rs as coreaudio;

use std::env;

use coreaudio::audio_toolbox::audio_file::*;
use coreaudio::audio_unit::graph::*;
use coreaudio::error::Error;

fn play_file(filename: &str) -> Result<(),Error> {

    let audio_file_id = try!( open_audio_file(filename) );
    println!("got audio_file_id {:?} ", audio_file_id);

    let _data_format_result = try!( get_data_format(audio_file_id) );

    // TO DO: the apple playfile sample has a long complicated stringify method for the asbd, we should port it
    // and call it here

    // lets set up our playing state now
    let graph_builder = AUGraph::new();

    Ok(())
}

fn main() {
    let mut args = env::args();
    // ummmm this is a new world of ugliness
    // there must be a nicer way of doing this
    args.next();

    match args.next() {
        Some(filename) => {
            println!("Using filename: {}", filename);

            let result = play_file(&filename[..]);

            match result {
                    Ok(_) => {
                    println!("Everything is ok.");
                },
                    Err(err) => {
                    panic!("Could not play file, error: {:?}", err);
                }
                }
        },
        None => {
            println!("Usage: playfile <path to audio file>");
        }
    }
()
}
