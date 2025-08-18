// Main thread logic
const fileInput = document.getElementById('file-input');
const fileLabel = document.getElementById('file-label');
const fileNameSpan = document.getElementById('file-name');
const sourceFormat = document.getElementById('source-format');
const targetFormat = document.getElementById('target-format');
const convertBtn = document.getElementById('convert-btn');
const status = document.getElementById('status');
const downloadContainer = document.getElementById('download-link-container');
const loader = document.getElementById('loader');

let selectedFile = null;

// Create a worker to handle the conversion in the background
const worker = new Worker('worker.js', { type: 'module' });

function updateFileDisplay(file) {
  if (file) {
    selectedFile = file;
    fileNameSpan.textContent = file.name;
    fileLabel.classList.add('file-loaded');
  } else {
    selectedFile = null;
    fileNameSpan.textContent = 'Click to browse or drag & drop a file';
    fileLabel.classList.remove('file-loaded');
  }
}

fileInput.addEventListener('change', () => {
  updateFileDisplay(fileInput.files[0]);
});

// Drag and drop functionality
fileLabel.addEventListener('dragover', (e) => {
  e.preventDefault();
  fileLabel.classList.add('dragover');
});

fileLabel.addEventListener('dragleave', (e) => {
  e.preventDefault();
  fileLabel.classList.remove('dragover');
});

fileLabel.addEventListener('drop', (e) => {
  e.preventDefault();
  fileLabel.classList.remove('dragover');
  if (e.dataTransfer.files.length > 0) {
    fileInput.files = e.dataTransfer.files;
    updateFileDisplay(e.dataTransfer.files[0]);
  }
});

convertBtn.addEventListener('click', async () => {
  if (!selectedFile) {
    status.textContent = 'Please select a file first!';
    status.className = 'status-message error';
    return;
  }

  // --- Start Conversion ---
  status.textContent = 'Reading file...';
  status.className = 'status-message';
  downloadContainer.innerHTML = '';
  convertBtn.disabled = true;
  loader.classList.add('visible');

  const reader = new FileReader();
  reader.readAsArrayBuffer(selectedFile);

  reader.onload = (e) => {
    const inputData = new Uint8Array(e.target.result);
    const srcFmt = sourceFormat.value;
    const tgtFmt = targetFormat.value;

    status.textContent = `Converting from ${srcFmt.toUpperCase()} to ${tgtFmt.toUpperCase()}...`;

    // Post data to the worker to process.
    // The ArrayBuffer is transferred, not copied, for performance.
    worker.postMessage({
      inputData,
      srcFmt,
      tgtFmt
    }, [inputData.buffer]);
  };

  reader.onerror = () => {
    status.textContent = 'Error reading file.';
    status.className = 'status-message error';
    convertBtn.disabled = false;
    loader.classList.remove('visible');
  };
});

// --- Listen for messages from the worker ---
worker.onmessage = (e) => {
  const { status: msgStatus, data, error } = e.data;

  if (msgStatus === 'success') {
    const blob = new Blob([data], { type: 'application/octet-stream' });
    const url = URL.createObjectURL(blob);

    const newFileName = selectedFile.name.split('.').slice(0, -1).join('.') + `.${targetFormat.value}`;

    downloadContainer.innerHTML = `<a href="${url}" download="${newFileName}">Download ${newFileName}</a>`;
    status.textContent = 'Conversion successful!';
    status.className = 'status-message success';
  } else {
    status.textContent = `Error: ${error}`;
    status.className = 'status-message error';
    console.error('Conversion error from worker:', error);
  }

  // --- Finish Conversion ---
  convertBtn.disabled = false;
  loader.classList.remove('visible');
};

worker.onerror = (e) => {
  status.textContent = `A critical error occurred in the background process. Please reload the page.`;
  status.className = 'status-message error';
  convertBtn.disabled = false;
  loader.classList.remove('visible');
  console.error('An unhandled error occurred in the worker:', e);
};