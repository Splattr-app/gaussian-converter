// Worker thread logic
import init, { convert } from '../pkg/wasm.js';

let wasmInitialized = false;

/**
 * Initializes the WebAssembly module. This function is designed to be called
 * once and will handle any potential initialization errors.
 */
async function initializeWasm() {
  if (wasmInitialized) {
    return;
  }
  try {
    // The wasm-pack generated init() function loads the wasm file.
    await init();
    wasmInitialized = true;
  } catch (err) {
    console.error("WASM initialization failed in worker:", err);
    // We throw the error so it can be caught by the message handler
    // and reported back to the main thread.
    throw new Error("Failed to load the conversion module. It may be missing or blocked.");
  }
}

// Listen for messages from the main thread
self.onmessage = async (e) => {
  try {
    // Ensure the WASM module is initialized before proceeding.
    // This will only run the initialization logic on the very first message.
    await initializeWasm();

    const { inputData, srcFmt, tgtFmt } = e.data;

    // Perform the heavy computation (this is synchronous within the worker).
    const outputData = convert(inputData, srcFmt, tgtFmt);

    // Send the result back to the main thread.
    // The ArrayBuffer is transferred for performance (zero-copy).
    self.postMessage({
      status: 'success',
      data: outputData
    }, [outputData.buffer]);

  } catch (err) {
    // If any error occurs (either during initialization or conversion),
    // send a structured error message back to the main thread.
    self.postMessage({
      status: 'error',
      error: err.message || String(err) || 'An unknown error occurred.'
    });
  }
};