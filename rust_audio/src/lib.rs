use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Error};
use std::str;
use std::thread;
use std::time::Duration;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Clone)]
// Наш аудио файл, представленный в виде вектора со значениями амплитуды, сохраняемой в типе
// u16, что представляет собой 2^16 = 65 536 и позволяет сохранять значения от -32 768 до 32 768
pub struct AudioClip {
    pub samples: Vec<i16>,
    pub sample_rate: u16,
    pub channels: u16,
    pub position: usize, // позиция текущего проигрываемого сэмпла
    pub is_looped: bool 
}

// Содержит инфу о хосте и устройстве по умолчанию
pub struct AudioManager {
    pub host: cpal::Host,
    pub device: cpal::Device
}

// Вектор из аудио клипов 
// Если вектор пустой, то ничего не воспроизводится и буфер заполняется тишиной
// Если векторе есть аудио клипы, буфер заполняется суммой сэмплов от каждого клипа по индивидуальной позиции
// которая трекается для каждого клипа
pub struct AudioPool {
    pub audio_clip: AudioClip,
    pub sample_position: i32,
    pub is_loop: bool
}

impl AudioClip {
    // Анализируем файл wav по чанкам и заполняем вектор значений амплитуды возвращая наш AudioClip
    pub fn new(path: &str) -> Result<AudioClip, Error> {

        let mut file = File::open(path)?;

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

        let mut audio_clip = AudioClip {
            samples: Vec::new(),
            sample_rate: 48000,
            channels: 2,
            position: 0,
            is_looped: false
        };        

        let mut i = 0;
        //Размер дата чанка N байт, на каждый сэмпл в 16 битах приходится два байта
        while i <= (data_chunk_size) as u64 {
            file.seek(SeekFrom::Start(736 + i))?;
            file.read(&mut buffer_2)?;
            let amp = i16::from_le_bytes(buffer_2);
            audio_clip.samples.push(amp);
            i = i + 2;
        }

        Ok(audio_clip)

    }

    // fn play(&self) {
    //     unimplemented!()
    // }

    // fn pause(&self) {
    //     unimplemented!()
    // }

    // fn stop(&self) {
    //     unimplemented!()
    // }
}

impl AudioManager {
    pub fn new() -> Result<AudioManager, std::io::Error>  {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let audio_manager = AudioManager {
            host,
            device
        };
        Ok(audio_manager)
    }

    pub fn play_audio(&self, audio_clip1: &mut AudioClip) -> std::io::Result<()> {
        // Не нравится, что пришлось клонировать наш клип, поскольку иначе 
        // программа ругалась на передачу клипа в анонимную функцию
        // Типа мы не можем захватить ссылку на переменную в анонимную функцию, поскольку
        // непонятно время жизни этой переменной, короче нужно решить вопрос без клонирования
        let mut audio_clip = audio_clip1.clone();
        // ---
        let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
        let supported_config = self.device.default_output_config().unwrap();
        let config = supported_config.into();  
        let stream = self.device
            .build_output_stream(
                &config,
                 move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
    
                    // Заполняю буфер пустотой, чтобы если у нас закончились сэмплы,
                    // а поток еще работает, то была тишина вместо неприятно треска
                    for sample in data.iter_mut() {
                        *sample = cpal::Sample::EQUILIBRIUM;
                    }
    
                    'outer: for frame in data.chunks_mut(2) {
                        for sample in frame.iter_mut() {
                            if audio_clip.position < audio_clip.samples.len() {
                                *sample = audio_clip.samples[audio_clip.position];                       
                            } else {
                                if audio_clip.is_looped {
                                    // мы проиграли весь аудио файл и сбрасываем позицию воспроизведения 
                                    // в ноль, чтобы начать воспроизведение с начала
                                    audio_clip.position = 0;            
                                } else {
                                    break 'outer;
                                }
                            }
                        }                    
                        audio_clip.position += 1;
                    }                
                },
                err_fn, None
            )
            .unwrap();
    
        stream.play().unwrap();     

        thread::sleep(Duration::from_secs(10));

        Ok(())
           
    }
}
