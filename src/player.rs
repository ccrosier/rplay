use rodio::{Sink, Decoder};
use std::io::BufReader;
use std::fs::File;
pub struct SourceFile {
    pub file_path: String,
}

impl SourceFile {
    pub fn decode(&self) -> Result<Decoder<BufReader<File>>, rodio::decoder::DecoderError> {
        let buf = BufReader::new(File::open(self.file_path.clone()).unwrap());
        Decoder::new(buf)
    }
}

pub struct Player {
    pub sink: Sink,
}

impl Player {
    
    pub fn add_to_queue(&self, source: SourceFile) {
        match source.decode() {
            Ok(s) => self.sink.append(s),
            Err(e) => eprintln!("Error adding to queue: {}", e)
        }
    }

    pub fn toggle_playback(&self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }
    
    pub fn progress(&self) -> (u64, u64) {
        (self.sink.get_pos().as_secs(), self.sink.len() as u64)
    }
    
    pub fn skip(&self) {
        self.sink.skip_one();
    }
    
    pub fn fast_forward(&self, secs: u64) {
        let cur_pos = self.sink.get_pos();
        let new_pos = cur_pos + std::time::Duration::new(secs, 0);
        self.sink.try_seek(new_pos).unwrap()
    }

    pub fn rewind(&self, secs: u64) {
        let cur_pos = self.sink.get_pos();
        let new_pos = cur_pos - std::time::Duration::new(secs, 0);
        self.sink.try_seek(new_pos).unwrap()
    }

    pub fn set_volume(&self, mult: f32) {
        self.sink.set_volume(mult);
    }
    
    pub fn restart_track(&self) {
        let _ = self.sink.try_seek(std::time::Duration::new(0, 0));
    }
}

