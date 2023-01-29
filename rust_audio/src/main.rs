use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::str;

fn main() -> std::io::Result<()> {
  
    let path = "assets/music_mono.wav";
    let mut file = File::open(path)?;
   // let mut file_output = File::create("wav.txt")?;

    let mut buffer_4 = [0; 4];
    let mut buffer_2 = [0; 2];

    file.seek(SeekFrom::Start(0))?;
    file.read(&mut buffer_4)?;
    println!("Chunk ID: {}", str::from_utf8(&buffer_4).unwrap());    

    file.seek(SeekFrom::Start(16))?;
    file.read(&mut buffer_4)?;
    println!("Chunk size: {}", u32::from_le_bytes(buffer_4));

    file.seek(SeekFrom::Start(22))?;
    file.read(&mut buffer_2)?;
    println!("Channel numbers: {}", u16::from_le_bytes(buffer_2));

    file.seek(SeekFrom::Start(24))?;
    file.read(&mut buffer_4)?;
    println!("Sample rate: {}", u32::from_le_bytes(buffer_4));

    file.seek(SeekFrom::Start(32))?;
    file.read(&mut buffer_2)?;
    println!(
        "Количество байт для одного сэмпла, включая все каналы: {}",
        u16::from_le_bytes(buffer_2)
    );

    file.seek(SeekFrom::Start(34))?;
    file.read(&mut buffer_2)?;
    println!("Bit per sample: {}", u16::from_le_bytes(buffer_2));

    file.seek(SeekFrom::Start(728))?;
    file.read(&mut buffer_4)?;
    println!("Chunk ID: {}", str::from_utf8(&buffer_4).unwrap());    

    file.seek(SeekFrom::Start(732))?;
    file.read(&mut buffer_4)?;
    let data_chunk_size = u32::from_le_bytes(buffer_4);
    println!("Количество байт в области данных: {}", data_chunk_size);

    // -32768 - 32768
    let mut samples: Vec<i16> = Vec::new();

    let mut i = 0;
    //Размер дата чанка N байт, на каждый сэмпл в 16 битах приходится два байта
    while i <= (data_chunk_size) as u64 {
        file.seek(SeekFrom::Start(736 + i))?;
        file.read(&mut buffer_2)?;
        let amp = i16::from_le_bytes(buffer_2);
        samples.push(amp);
        //writeln!(file_output, "{}", amp)?;

        i = i + 2;
    }

    println!("Количество сэмплов в векторе: {}", samples.len());

    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let host = cpal::default_host();
    let device = host.default_output_device().unwrap();
    let supported_config = device.default_output_config().unwrap();

    let config = supported_config.into();

    let mut i = 0;
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(2) {
                    for sample in frame.iter_mut() {
                        if i < samples.len() {
                            *sample = samples[i];                       
                        } 
                    }
                    
                    i += 1;
                }
                println!("Проигрываемый сэмпл: {}", i);
            },
            err_fn,
        )
        .unwrap();

    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
