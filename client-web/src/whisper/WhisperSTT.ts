// The MIT License (MIT)
//
// Copyright (c) 2023 Nitai Aharoni
// Copyright (c) 2024 Shane Vandegrift
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
// -----------------------------------------------------------------------------
//
// Code taken from (with modifications):
// https://github.com/nitaiaharoni1/whisper-speech-to-text/blob/8dfaf57bb20eadb2aa5958e8c8edd20e3ba34d57/src/WhisperSTT.ts

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
