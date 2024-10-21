import { RecordRTCPromisesHandler } from 'recordrtc';

const AUDIO_TYPE = 'audio';

export class WhisperSTT {
  private recorder: RecordRTCPromisesHandler | null;
  private stream: MediaStream | null;
  public isRecording: boolean;
  public isStopped: boolean;
  public isPaused: boolean;

  constructor() {
    this.recorder = null;
    this.stream = null;
    this.isRecording = false;
    this.isStopped = true;
    this.isPaused = false;
  }

  public pauseRecording = async (): Promise<void> => {
    if (!this.recorder) {
      throw new Error('Cannot pause recording: no recorder');
    }
    await this.recorder.pauseRecording();
    this.isPaused = true;
    this.isRecording = false;
  };

  public resumeRecording = async (): Promise<void> => {
    if (!this.recorder) {
      throw new Error('Cannot resume recording: no recorder');
    }
    await this.recorder.resumeRecording();
    this.isPaused = false;
    this.isRecording = true;
  };

  public startRecording = async (): Promise<void> => {
    try {
      this.stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      this.recorder = new RecordRTCPromisesHandler(this.stream, {
        type: AUDIO_TYPE,
      });
      this.recorder.startRecording();
      this.isRecording = true;
      this.isStopped = false;
    } catch (error: any) {
      this.isRecording = false;
      this.isStopped = true;
      throw new Error(`Error starting recording: ${error.message}`);
    }
  };

  public stopRecording = async (
    onFinish: (blob: Blob) => void
  ): Promise<void> => {
    if (!this.isRecording || !this.recorder) {
      throw new Error('Cannot stop recording: no recorder');
    }
    try {
      await this.recorder.stopRecording();
      const blob = await this.recorder.getBlob();
      onFinish(blob);
      this.stream?.getTracks().forEach((track) => {
        track.stop();
      });
      this.recorder = null;
      this.stream = null;
      this.isRecording = false;
      this.isStopped = true;
      this.isPaused = false;
    } catch (error: any) {
      this.isRecording = false;
      this.isStopped = true;
      throw new Error(`Error stopping recording: ${error.message}`);
    }
  };

  public abortRecording = async () => {
    if (!this.isRecording || !this.recorder) {
      throw new Error('Cannot stop recording: no recorder');
    }
    await this.recorder.stopRecording();
    this.recorder = null;
    this.stream = null;
    this.isRecording = false;
    this.isStopped = true;
    this.isPaused = false;
  };
}
