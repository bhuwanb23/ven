const SAMPLE_RATE = 44100;

function srng(seed: number) {
  let s = seed;
  return () => {
    s = (s * 16807 + 0) % 2147483647;
    return (s - 1) / 2147483646;
  };
}

function encodeWav(samples: Float32Array, sampleRate: number): string {
  const numChannels = 1;
  const bitsPerSample = 16;
  const byteRate = sampleRate * numChannels * (bitsPerSample / 8);
  const blockAlign = numChannels * (bitsPerSample / 8);
  const dataSize = samples.length * (bitsPerSample / 8);
  const buffer = new ArrayBuffer(44 + dataSize);
  const view = new DataView(buffer);

  const writeStr = (offset: number, str: string) => {
    for (let i = 0; i < str.length; i++) view.setUint8(offset + i, str.charCodeAt(i));
  };

  writeStr(0, "RIFF");
  view.setUint32(4, 36 + dataSize, true);
  writeStr(8, "WAVE");
  writeStr(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, numChannels, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, byteRate, true);
  view.setUint16(32, blockAlign, true);
  view.setUint16(34, bitsPerSample, true);
  writeStr(36, "data");
  view.setUint32(40, dataSize, true);

  for (let i = 0; i < samples.length; i++) {
    const s = Math.max(-1, Math.min(1, samples[i]));
    view.setInt16(44 + i * 2, s * 0x7fff, true);
  }

  const bytes = new Uint8Array(buffer);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i]);
  return "data:audio/wav;base64," + btoa(binary);
}

function noiseBuffer(len: number, seed: number): Float32Array {
  const rng = srng(seed);
  const buf = new Float32Array(len);
  for (let i = 0; i < len; i++) buf[i] = rng() * 2 - 1;
  return buf;
}

export function generateTypingClick(): string {
  const len = Math.floor(SAMPLE_RATE * 0.015);
  const buf = new Float32Array(len);
  const noise = noiseBuffer(len, 42);
  for (let i = 0; i < len; i++) {
    const env = Math.exp(-i / (len * 0.15));
    buf[i] = noise[i] * env * 0.25;
  }
  return encodeWav(buf, SAMPLE_RATE);
}

export function generateMouseClick(): string {
  const len = Math.floor(SAMPLE_RATE * 0.04);
  const buf = new Float32Array(len);
  for (let i = 0; i < len; i++) {
    const t = i / SAMPLE_RATE;
    const click = Math.sin(2 * Math.PI * 1200 * t) * Math.exp(-t / 0.005);
    const body = Math.sin(2 * Math.PI * 400 * t) * Math.exp(-t / 0.012);
    buf[i] = (click + body) * 0.2;
  }
  return encodeWav(buf, SAMPLE_RATE);
}

export function generateWhoosh(): string {
  const len = Math.floor(SAMPLE_RATE * 0.4);
  const buf = new Float32Array(len);
  const noise = noiseBuffer(len, 43);
  for (let i = 0; i < len; i++) {
    const t = i / SAMPLE_RATE;
    const freq = 200 + 1800 * (i / len);
    const filtered = Math.sin(2 * Math.PI * freq * t) * 0.3 + noise[i] * 0.2;
    const env = Math.exp(-Math.pow((i / len - 0.3) / 0.25, 2));
    buf[i] = filtered * env * 0.15;
  }
  return encodeWav(buf, SAMPLE_RATE);
}

export function generateWhooshShort(): string {
  const len = Math.floor(SAMPLE_RATE * 0.15);
  const buf = new Float32Array(len);
  const noise = noiseBuffer(len, 44);
  for (let i = 0; i < len; i++) {
    const t = i / SAMPLE_RATE;
    const freq = 300 + 1200 * (i / len);
    const filtered = Math.sin(2 * Math.PI * freq * t) * 0.3 + noise[i] * 0.2;
    const env = Math.exp(-Math.pow((i / len - 0.2) / 0.2, 2));
    buf[i] = filtered * env * 0.15;
  }
  return encodeWav(buf, SAMPLE_RATE);
}

export function generateErrorBuzz(): string {
  const len = Math.floor(SAMPLE_RATE * 0.3);
  const buf = new Float32Array(len);
  for (let i = 0; i < len; i++) {
    const t = i / SAMPLE_RATE;
    const saw = 2 * ((t * 100) % 1) - 1;
    const env = Math.exp(-t / 0.08);
    buf[i] = saw * env * 0.2;
  }
  return encodeWav(buf, SAMPLE_RATE);
}

export function generateSuccessDing(): string {
  const len = Math.floor(SAMPLE_RATE * 0.3);
  const buf = new Float32Array(len);
  for (let i = 0; i < len; i++) {
    const t = i / SAMPLE_RATE;
    const f1 = Math.sin(2 * Math.PI * 880 * t);
    const f2 = Math.sin(2 * Math.PI * 1320 * t) * 0.3;
    const env = Math.exp(-t / 0.08);
    buf[i] = (f1 + f2) * env * 0.2;
  }
  return encodeWav(buf, SAMPLE_RATE);
}

export function generateOrbChime(index: number): string {
  const len = Math.floor(SAMPLE_RATE * 0.2);
  const buf = new Float32Array(len);
  const baseFreq = 440 + index * 55;
  for (let i = 0; i < len; i++) {
    const t = i / SAMPLE_RATE;
    const f1 = Math.sin(2 * Math.PI * baseFreq * t);
    const f2 = Math.sin(2 * Math.PI * baseFreq * 1.5 * t) * 0.2;
    const env = Math.exp(-t / 0.05);
    buf[i] = (f1 + f2) * env * 0.15;
  }
  return encodeWav(buf, SAMPLE_RATE);
}

export function generateAmbientDrone(totalFrames: number): string {
  const totalSec = totalFrames / 30;
  const len = Math.floor(SAMPLE_RATE * totalSec);
  const buf = new Float32Array(len);
  for (let i = 0; i < len; i++) {
    const t = i / SAMPLE_RATE;
    const a = Math.sin(2 * Math.PI * 55 * t) * 0.03;
    const b = Math.sin(2 * Math.PI * 82.5 * t) * 0.02;
    const c = Math.sin(2 * Math.PI * 110 * t) * 0.015;
    const breathe = 1 + 0.3 * Math.sin(2 * Math.PI * 0.125 * t);
    buf[i] = (a + b + c) * breathe * 0.3;
  }
  return encodeWav(buf, SAMPLE_RATE);
}

export function generateScanWobble(): string {
  const len = Math.floor(SAMPLE_RATE * 0.3);
  const buf = new Float32Array(len);
  for (let i = 0; i < len; i++) {
    const t = i / SAMPLE_RATE;
    const freq = 200 + 400 * Math.sin(2 * Math.PI * 15 * t);
    const wave = Math.sin(2 * Math.PI * freq * t);
    const env = Math.exp(-Math.pow((i / len - 0.2) / 0.15, 2));
    buf[i] = wave * env * 0.12;
  }
  return encodeWav(buf, SAMPLE_RATE);
}
