use rust_audio::{AudioClip, AudioManager};
use std::{thread, time::Duration};

fn main() -> std::io::Result<()> {
    loop {
        let mut input = String::new();
        println!("Enter a command: ");

        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        let input = input.trim();
        if input == "quit" {
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() < 1 {
            println!("Not enough parameters");
        } else {
            let path = format!("assets/{}.wav", parts[0]);
            // аудио клип мутабельный, потому что мы записываем в него данные о текущем
            // воспроизводимом сэмпле в свойство position
            let mut audio_clip = AudioClip::new(path.as_str())?;
            audio_clip.is_looped = false;
            let audio_manager = AudioManager::new()?;
            let handle = thread::spawn(move || {
                audio_manager.play_audio(&mut audio_clip).unwrap();                
            });
            handle.join().unwrap();
        }
    }

    Ok(())
}
