// A minimal WASI (preview 1) host for Node: enough for chuk-basic, which
// reads one program file, reads standard input, and writes standard output
// and standard error. It needs no other library, so the extension can carry
// the interpreter and the language server as one WebAssembly file and run
// them with the Node that VS Code already has.
"use strict";

const fs = require("fs");

const ERRNO_BADF = 8;
const ERRNO_INVAL = 28;
const ERRNO_NOENT = 44;
const FILETYPE_CHARACTER_DEVICE = 2;
const FILETYPE_DIRECTORY = 3;
const FILETYPE_REGULAR_FILE = 4;
// the one directory the program can see, holding the files it is given
const PREOPEN_FD = 3;
const PREOPEN_NAME = Buffer.from("/");

const pause = new Int32Array(new SharedArrayBuffer(4));
const sleep = (ms) => Atomics.wait(pause, 0, 0, ms);

// Standard input may be a pipe that is not ready yet: wait and try again.
function readInput(buffer) {
  for (;;) {
    try {
      return fs.readSync(0, buffer, 0, buffer.length, null);
    } catch (error) {
      if (error.code === "EOF") return 0;
      if (error.code !== "EAGAIN") throw error;
      sleep(5);
    }
  }
}

function writeOutput(fd, bytes) {
  let done = 0;
  while (done < bytes.length) {
    try {
      done += fs.writeSync(fd, bytes, done, bytes.length - done);
    } catch (error) {
      // the reader went away (`... | head`): stop quietly
      if (error.code === "EPIPE") throw new Exit(0);
      if (error.code !== "EAGAIN") throw error;
      sleep(5);
    }
  }
}

/**
 * Runs a WASI command to completion and returns its exit code.
 *
 * @param {string} wasmPath   the compiled program
 * @param {object} [options]
 * @param {string[]} [options.args]   arguments after the program name
 * @param {Object<string, Uint8Array>} [options.files]  files the program
 *        can open, by the name it will use for them (such as "/prog.bas")
 */
function run(wasmPath, { args = [], files = {} } = {}) {
  const module = new WebAssembly.Module(fs.readFileSync(wasmPath));
  let memory;
  const view = () => new DataView(memory.buffer);
  const bytes = () => new Uint8Array(memory.buffer);
  const argv = ["chuk-basic", ...args].map((arg) => Buffer.from(arg + "\0"));
  const normalise = (name) => name.replace(/^\/+/, "");
  const known = new Map(Object.entries(files).map(([name, data]) => [normalise(name), data]));
  const open = new Map();
  let nextFd = PREOPEN_FD + 1;

  const iovecs = (ptr, count) => {
    const list = [];
    for (let i = 0; i < count; i++) {
      list.push([view().getUint32(ptr + i * 8, true), view().getUint32(ptr + i * 8 + 4, true)]);
    }
    return list;
  };
  const kind = (fd) => {
    if (fd >= 0 && fd <= 2) return FILETYPE_CHARACTER_DEVICE;
    if (fd === PREOPEN_FD) return FILETYPE_DIRECTORY;
    if (open.has(fd)) return FILETYPE_REGULAR_FILE;
    return undefined;
  };

  const wasi = {
    args_sizes_get(countPtr, sizePtr) {
      view().setUint32(countPtr, argv.length, true);
      view().setUint32(sizePtr, argv.reduce((n, a) => n + a.length, 0), true);
      return 0;
    },
    args_get(argvPtr, bufPtr) {
      for (const arg of argv) {
        view().setUint32(argvPtr, bufPtr, true);
        bytes().set(arg, bufPtr);
        argvPtr += 4;
        bufPtr += arg.length;
      }
      return 0;
    },
    environ_sizes_get(countPtr, sizePtr) {
      view().setUint32(countPtr, 0, true);
      view().setUint32(sizePtr, 0, true);
      return 0;
    },
    environ_get() {
      return 0;
    },
    // RND is seeded from these two unless the program is run with --seed
    clock_time_get(_id, _precision, timePtr) {
      view().setBigUint64(timePtr, BigInt(Date.now()) * 1000000n, true);
      return 0;
    },
    random_get(ptr, length) {
      require("crypto").randomFillSync(bytes(), ptr, length);
      return 0;
    },
    fd_prestat_get(fd, ptr) {
      if (fd !== PREOPEN_FD) return ERRNO_BADF;
      view().setUint8(ptr, 0); // a directory
      view().setUint32(ptr + 4, PREOPEN_NAME.length, true);
      return 0;
    },
    fd_prestat_dir_name(fd, ptr, length) {
      if (fd !== PREOPEN_FD) return ERRNO_BADF;
      if (length < PREOPEN_NAME.length) return ERRNO_INVAL;
      bytes().set(PREOPEN_NAME, ptr);
      return 0;
    },
    fd_fdstat_get(fd, ptr) {
      const type = kind(fd);
      if (type === undefined) return ERRNO_BADF;
      bytes().fill(0, ptr, ptr + 24);
      view().setUint8(ptr, type);
      view().setBigUint64(ptr + 8, 0xffffffffffffffffn, true); // rights
      view().setBigUint64(ptr + 16, 0xffffffffffffffffn, true);
      return 0;
    },
    fd_filestat_get(fd, ptr) {
      const type = kind(fd);
      if (type === undefined) return ERRNO_BADF;
      bytes().fill(0, ptr, ptr + 64);
      view().setUint8(ptr + 16, type);
      view().setBigUint64(ptr + 24, 1n, true); // links
      view().setBigUint64(ptr + 32, BigInt(open.has(fd) ? open.get(fd).data.length : 0), true);
      return 0;
    },
    path_open(dirFd, _dirFlags, pathPtr, pathLength, _oflags, _rights, _inheriting, _fdFlags, fdPtr) {
      if (dirFd !== PREOPEN_FD) return ERRNO_BADF;
      const name = normalise(Buffer.from(bytes().subarray(pathPtr, pathPtr + pathLength)).toString());
      const data = known.get(name);
      if (data === undefined) return ERRNO_NOENT;
      const fd = nextFd++;
      open.set(fd, { data, position: 0 });
      view().setUint32(fdPtr, fd, true);
      return 0;
    },
    fd_close(fd) {
      return open.delete(fd) || (fd >= 0 && fd <= PREOPEN_FD) ? 0 : ERRNO_BADF;
    },
    fd_read(fd, iovsPtr, iovsLength, nreadPtr) {
      const file = open.get(fd);
      if (fd !== 0 && file === undefined) return ERRNO_BADF;
      let total = 0;
      for (const [ptr, length] of iovecs(iovsPtr, iovsLength)) {
        if (length === 0) continue;
        if (file) {
          const chunk = file.data.subarray(file.position, file.position + length);
          bytes().set(chunk, ptr);
          file.position += chunk.length;
          total += chunk.length;
          if (chunk.length < length) break;
        } else {
          // the keyboard, or a pipe: whatever has arrived, one read at a time
          const buffer = Buffer.alloc(length);
          total = readInput(buffer);
          bytes().set(buffer.subarray(0, total), ptr);
          break;
        }
      }
      view().setUint32(nreadPtr, total, true);
      return 0;
    },
    fd_write(fd, iovsPtr, iovsLength, nwrittenPtr) {
      if (fd !== 1 && fd !== 2) return ERRNO_BADF;
      let total = 0;
      for (const [ptr, length] of iovecs(iovsPtr, iovsLength)) {
        if (length > 0) writeOutput(fd, bytes().slice(ptr, ptr + length));
        total += length;
      }
      view().setUint32(nwrittenPtr, total, true);
      return 0;
    },
    proc_exit(code) {
      throw new Exit(code);
    },
  };

  const instance = new WebAssembly.Instance(module, { wasi_snapshot_preview1: wasi });
  memory = instance.exports.memory;
  try {
    instance.exports._start();
    return 0;
  } catch (error) {
    if (error instanceof Exit) return error.code;
    throw error;
  }
}

class Exit extends Error {
  constructor(code) {
    super(`exit ${code}`);
    this.code = code;
  }
}

module.exports = { run };
