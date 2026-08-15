const IV = new Uint32Array([
  0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
  0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
]);

const MSG_PERMUTATION = new Uint8Array([2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8]);
const CHUNK_LEN = 1024;
const BLOCK_LEN = 64;
const CHUNK_START = 1;
const CHUNK_END = 2;
const PARENT = 4;
const ROOT = 8;

function rotr32(value: number, shift: number): number {
  return ((value >>> shift) | (value << (32 - shift))) >>> 0;
}

function g(state: Uint32Array, a: number, b: number, c: number, d: number, mx: number, my: number): void {
  state[a] = (state[a]! + state[b]! + mx) >>> 0;
  state[d] = rotr32(state[d]! ^ state[a]!, 16);
  state[c] = (state[c]! + state[d]!) >>> 0;
  state[b] = rotr32(state[b]! ^ state[c]!, 12);
  state[a] = (state[a]! + state[b]! + my) >>> 0;
  state[d] = rotr32(state[d]! ^ state[a]!, 8);
  state[c] = (state[c]! + state[d]!) >>> 0;
  state[b] = rotr32(state[b]! ^ state[c]!, 7);
}

function round(state: Uint32Array, m: Uint32Array): void {
  g(state, 0, 4, 8, 12, m[0]!, m[1]!);
  g(state, 1, 5, 9, 13, m[2]!, m[3]!);
  g(state, 2, 6, 10, 14, m[4]!, m[5]!);
  g(state, 3, 7, 11, 15, m[6]!, m[7]!);
  g(state, 0, 5, 10, 15, m[8]!, m[9]!);
  g(state, 1, 6, 11, 12, m[10]!, m[11]!);
  g(state, 2, 7, 8, 13, m[12]!, m[13]!);
  g(state, 3, 4, 9, 14, m[14]!, m[15]!);
}

function permute(m: Uint32Array): Uint32Array {
  const result = new Uint32Array(16);
  for (let i = 0; i < 16; i += 1) result[i] = m[MSG_PERMUTATION[i]!]!;
  return result;
}

function compress(
  chainingValue: Uint32Array,
  blockWords: Uint32Array,
  counter: number,
  blockLen: number,
  flags: number,
): Uint32Array {
  const state = new Uint32Array(16);
  state.set(chainingValue, 0);
  state[8] = IV[0]!;
  state[9] = IV[1]!;
  state[10] = IV[2]!;
  state[11] = IV[3]!;
  state[12] = counter >>> 0;
  state[13] = Math.floor(counter / 0x100000000) >>> 0;
  state[14] = blockLen >>> 0;
  state[15] = flags >>> 0;

  let m = new Uint32Array(blockWords);
  for (let r = 0; r < 7; r += 1) {
    round(state, m);
    if (r !== 6) m = permute(m);
  }

  for (let i = 0; i < 8; i += 1) {
    state[i] = (state[i]! ^ state[i + 8]!) >>> 0;
    state[i + 8] = (state[i + 8]! ^ chainingValue[i]!) >>> 0;
  }
  return state;
}

function wordsFromBlock(block: Uint8Array): Uint32Array {
  const words = new Uint32Array(16);
  const padded = new Uint8Array(BLOCK_LEN);
  padded.set(block.subarray(0, Math.min(block.length, BLOCK_LEN)));
  const view = new DataView(padded.buffer, padded.byteOffset, padded.byteLength);
  for (let i = 0; i < 16; i += 1) words[i] = view.getUint32(i * 4, true);
  return words;
}

function wordsToBytes(words: Uint32Array): Uint8Array {
  const bytes = new Uint8Array(words.length * 4);
  const view = new DataView(bytes.buffer);
  for (let i = 0; i < words.length; i += 1) view.setUint32(i * 4, words[i]!, true);
  return bytes;
}

class Output {
  readonly inputCv: Uint32Array;
  readonly blockWords: Uint32Array;
  readonly counter: number;
  readonly blockLen: number;
  readonly flags: number;

  constructor(
    inputCv: Uint32Array,
    blockWords: Uint32Array,
    counter: number,
    blockLen: number,
    flags: number,
  ) {
    this.inputCv = inputCv;
    this.blockWords = blockWords;
    this.counter = counter;
    this.blockLen = blockLen;
    this.flags = flags;
  }

  chainingValue(): Uint32Array {
    return compress(this.inputCv, this.blockWords, this.counter, this.blockLen, this.flags).slice(0, 8);
  }

  rootBytes(length: number): Uint8Array {
    const result = new Uint8Array(length);
    let offset = 0;
    let outputBlockCounter = 0;
    while (offset < length) {
      const block = wordsToBytes(compress(
        this.inputCv,
        this.blockWords,
        outputBlockCounter,
        this.blockLen,
        this.flags | ROOT,
      ));
      const take = Math.min(block.length, length - offset);
      result.set(block.subarray(0, take), offset);
      offset += take;
      outputBlockCounter += 1;
    }
    return result;
  }
}

class ChunkState {
  private cv: Uint32Array;
  private readonly block = new Uint8Array(BLOCK_LEN);
  private blockLen = 0;
  private blocksCompressed = 0;
  readonly chunkCounter: number;
  private readonly flags: number;

  constructor(
    keyWords: Uint32Array,
    chunkCounter: number,
    flags: number,
  ) {
    this.cv = new Uint32Array(keyWords);
    this.chunkCounter = chunkCounter;
    this.flags = flags;
  }

  length(): number {
    return this.blocksCompressed * BLOCK_LEN + this.blockLen;
  }

  private startFlag(): number {
    return this.blocksCompressed === 0 ? CHUNK_START : 0;
  }

  update(input: Uint8Array): void {
    let offset = 0;
    while (offset < input.length) {
      if (this.blockLen === BLOCK_LEN) {
        const output = compress(
          this.cv,
          wordsFromBlock(this.block),
          this.chunkCounter,
          BLOCK_LEN,
          this.flags | this.startFlag(),
        );
        this.cv = output.slice(0, 8);
        this.blocksCompressed += 1;
        this.blockLen = 0;
        this.block.fill(0);
      }
      const want = Math.min(BLOCK_LEN - this.blockLen, input.length - offset);
      this.block.set(input.subarray(offset, offset + want), this.blockLen);
      this.blockLen += want;
      offset += want;
    }
  }

  output(): Output {
    return new Output(
      new Uint32Array(this.cv),
      wordsFromBlock(this.block.subarray(0, this.blockLen)),
      this.chunkCounter,
      this.blockLen,
      this.flags | this.startFlag() | CHUNK_END,
    );
  }
}

function parentOutput(left: Uint32Array, right: Uint32Array, keyWords: Uint32Array, flags: number): Output {
  const blockWords = new Uint32Array(16);
  blockWords.set(left, 0);
  blockWords.set(right, 8);
  return new Output(new Uint32Array(keyWords), blockWords, 0, BLOCK_LEN, flags | PARENT);
}

function parentCv(left: Uint32Array, right: Uint32Array, keyWords: Uint32Array, flags: number): Uint32Array {
  return parentOutput(left, right, keyWords, flags).chainingValue();
}

export function blake3(input: Uint8Array): Uint8Array {
  const keyWords = new Uint32Array(IV);
  const flags = 0;
  let chunkState = new ChunkState(keyWords, 0, flags);
  const cvStack: Uint32Array[] = [];
  let offset = 0;

  while (offset < input.length) {
    if (chunkState.length() === CHUNK_LEN) {
      let newCv = chunkState.output().chainingValue();
      let totalChunks = chunkState.chunkCounter + 1;
      while ((totalChunks & 1) === 0) {
        const left = cvStack.pop();
        if (!left) throw new Error("BLAKE3 tree stack underflow");
        newCv = parentCv(left, newCv, keyWords, flags);
        totalChunks >>= 1;
      }
      cvStack.push(newCv);
      chunkState = new ChunkState(keyWords, chunkState.chunkCounter + 1, flags);
    }
    const want = Math.min(CHUNK_LEN - chunkState.length(), input.length - offset);
    chunkState.update(input.subarray(offset, offset + want));
    offset += want;
  }

  let output = chunkState.output();
  while (cvStack.length > 0) {
    const left = cvStack.pop()!;
    output = parentOutput(left, output.chainingValue(), keyWords, flags);
  }
  return output.rootBytes(32);
}

export function blake3HexUtf8(input: string): string {
  return Buffer.from(blake3(new TextEncoder().encode(input))).toString("hex");
}

export const BLAKE3_EMPTY_HEX = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";

export function assertBlake3SelfTest(): void {
  const observed = blake3HexUtf8("");
  if (observed !== BLAKE3_EMPTY_HEX) {
    throw new Error(`REFUSED:BLAKE3_SELF_TEST_FAILED:${observed}`);
  }
}
