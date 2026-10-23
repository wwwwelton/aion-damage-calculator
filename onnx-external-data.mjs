// Rewrites an ONNX model so that graph initializers live in a sidecar file
// (ONNX "external data" format). ONNX Runtime Web then copies each weight into
// its WebAssembly heap once instead of holding the protobuf and the weights at
// the same time, which halves the resident memory of a 0.5B-parameter model.
import { closeSync, openSync, readFileSync, writeSync } from 'node:fs';

const MODEL_GRAPH = 7;
const GRAPH_INITIALIZER = 5;
const TENSOR_RAW_DATA = 9;
const TENSOR_EXTERNAL_DATA = 13;
const TENSOR_DATA_LOCATION = 14;
const LENGTH_DELIMITED = 2;
const ALIGNMENT = 64;

function readVarint(buffer, offset) {
  let value = 0n;
  let shift = 0n;
  for (;;) {
    const byte = buffer[offset++];
    value |= BigInt(byte & 0x7f) << shift;
    if (byte < 0x80) return [value, offset];
    shift += 7n;
  }
}

function encodeVarint(value) {
  const bytes = [];
  let remaining = BigInt(value);
  do {
    let byte = Number(remaining & 0x7fn);
    remaining >>= 7n;
    if (remaining > 0n) byte |= 0x80;
    bytes.push(byte);
  } while (remaining > 0n);
  return Buffer.from(bytes);
}

// Splits a message into [{ field, wireType, bytes }] where `bytes` is the raw
// payload for length-delimited fields, and the encoded value otherwise.
function* fields(buffer) {
  let offset = 0;
  while (offset < buffer.length) {
    const [tag, afterTag] = readVarint(buffer, offset);
    const field = Number(tag >> 3n);
    const wireType = Number(tag & 7n);
    if (wireType === LENGTH_DELIMITED) {
      const [length, start] = readVarint(buffer, afterTag);
      const end = start + Number(length);
      yield { field, wireType, bytes: buffer.subarray(start, end) };
      offset = end;
    } else {
      let end = afterTag;
      if (wireType === 0) end = readVarint(buffer, afterTag)[1];
      else if (wireType === 1) end = afterTag + 8;
      else if (wireType === 5) end = afterTag + 4;
      else throw new Error(`Unsupported wire type ${wireType} in ONNX protobuf`);
      yield { field, wireType, bytes: buffer.subarray(afterTag, end) };
      offset = end;
    }
  }
}

function encodeField(field, wireType, bytes) {
  const tag = encodeVarint((BigInt(field) << 3n) | BigInt(wireType));
  return wireType === LENGTH_DELIMITED
    ? Buffer.concat([tag, encodeVarint(bytes.length), bytes])
    : Buffer.concat([tag, bytes]);
}

function encodeEntry(key, value) {
  return Buffer.concat([
    encodeField(1, LENGTH_DELIMITED, Buffer.from(key)),
    encodeField(2, LENGTH_DELIMITED, Buffer.from(String(value)))
  ]);
}

export function convertToExternalData(sourcePath, modelPath, dataPath, threshold = 1024) {
  const model = readFileSync(sourcePath);
  const dataName = dataPath.slice(dataPath.lastIndexOf('/') + 1);
  const dataFile = openSync(dataPath, 'w');
  let dataOffset = 0;
  const zeros = Buffer.alloc(ALIGNMENT);

  const convertTensor = tensor => {
    const parts = [];
    let raw = null;
    for (const entry of fields(tensor)) {
      if (entry.field === TENSOR_RAW_DATA && entry.bytes.length >= threshold) raw = entry.bytes;
      else parts.push(encodeField(entry.field, entry.wireType, entry.bytes));
    }
    if (!raw) return tensor;
    const padding = (ALIGNMENT - (dataOffset % ALIGNMENT)) % ALIGNMENT;
    if (padding) { writeSync(dataFile, zeros, 0, padding); dataOffset += padding; }
    writeSync(dataFile, raw);
    parts.push(
      encodeField(TENSOR_EXTERNAL_DATA, LENGTH_DELIMITED, encodeEntry('location', dataName)),
      encodeField(TENSOR_EXTERNAL_DATA, LENGTH_DELIMITED, encodeEntry('offset', dataOffset)),
      encodeField(TENSOR_EXTERNAL_DATA, LENGTH_DELIMITED, encodeEntry('length', raw.length)),
      encodeField(TENSOR_DATA_LOCATION, 0, encodeVarint(1))
    );
    dataOffset += raw.length;
    return Buffer.concat(parts);
  };

  const convertGraph = graph => Buffer.concat([...fields(graph)].map(entry =>
    encodeField(entry.field, entry.wireType,
      entry.field === GRAPH_INITIALIZER ? convertTensor(entry.bytes) : entry.bytes)));

  const output = Buffer.concat([...fields(model)].map(entry =>
    encodeField(entry.field, entry.wireType,
      entry.field === MODEL_GRAPH ? convertGraph(entry.bytes) : entry.bytes)));
  closeSync(dataFile);
  const modelFile = openSync(modelPath, 'w');
  writeSync(modelFile, output);
  closeSync(modelFile);
  return { modelBytes: output.length, dataBytes: dataOffset };
}
