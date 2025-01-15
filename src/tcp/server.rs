use crate::error::Result;
use crate::handlers::{ProcessingHandler, RecordingHandler, TranscriptionHandler};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;

pub struct TcpServer {
    listener: TcpListener,
    recorder: Box<dyn RecordingHandler>,
    transcriber: Arc<dyn TranscriptionHandler + Send + Sync>,
    processor: Arc<dyn ProcessingHandler + Send + Sync>,
}

impl TcpServer {
    pub fn new(
        addr: &str,
        recorder: Box<dyn RecordingHandler>,
        transcriber: Arc<dyn TranscriptionHandler + Send + Sync>,
        processor: Arc<dyn ProcessingHandler + Send + Sync>,
    ) -> Result<Self> {
        let listener = TcpListener::bind(addr)?;

        Ok(Self {
            listener,
            recorder,
            transcriber,
            processor,
        })
    }

    pub async fn listen(&self) -> Result<()> {
        let (stream, _addr) = self.listener.accept()?;
        self.handle_client(stream).await?;
        Ok(())
    }

    async fn handle_client(&self, stream: TcpStream) -> Result<()> {
        let mut reader = BufReader::new(&stream);
        let mut writer = &stream;
        let mut line = String::new();
        let mut recording_active = false;

        while reader.read_line(&mut line)? > 0 {
            let response = match line.trim() {
                "START_RECORDING" => {
                    self.recorder.start()?;
                    recording_active = true;
                    "Recording started.".to_string()
                }
                "STOP_RECORDING" if recording_active => {
                    let audio = self.recorder.stop()?;
                    recording_active = false;
                    self.transcriber.transcribe(&audio).await?
                }
                "STOP_RECORDING" => "No recording in progress.".to_string(),
                _ => "Unknown command.".to_string(),
            };

            writeln!(writer, "{}", response)?;
            line.clear();
        }

        Ok(())
    }
}
