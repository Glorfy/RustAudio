use rust_audio::{AudioClip, AudioManager, AudioSource};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn main() -> std::io::Result<()> {
 
    let audio_pool = Arc::new(Mutex::new(Vec::<AudioSource>::new()));
    let audio_manager = AudioManager::new()?;
    let audio_pool_clone = Arc::clone(&audio_pool);
    thread::spawn(move || {
        audio_manager
            .open_audio_stream(Arc::clone(&audio_pool))
            .unwrap();
    });

    loop {
        let mut input = String::new();
        println!("Enter a command: ");

        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim() {
            "quit" => break,
            "print" => {
                let a_pool = audio_pool_clone.lock().unwrap();
                for clip in a_pool.iter() {
                    println!("Audio pool contain file: {}", clip.filename);
                }
                continue;
            }
            "size" => {
                let a_pool = audio_pool_clone.lock().unwrap();
                println!("Current size of audio pool is: {}", a_pool.len());
                continue;
            }
            _ => (),
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() < 2 {
            println!("Not enough parameters");
        } else {
            match parts[1] {
                "add" => {
                    let path = format!("assets/{}.wav", parts[0]);
                    let audio_clip = AudioClip::new(path.as_str())?;
                    let new_audio_source = AudioSource {
                        audio_clip: audio_clip,
                        sample_position: 0,
                        is_loop: false,
                        filename: parts[0].to_string(),
                    };
    
                    let mut a_pool = audio_pool_clone.lock().unwrap();
                    a_pool.push(new_audio_source);
                },
                "rem" => {
                    let mut a_pool = audio_pool_clone.lock().unwrap();                    
                },
                _ => (),
            }
            if parts[1] == "add" {

            }
        }
    }

    Ok(())
}
