<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import {
  open as dialogOpen,
  save as dialogSave,
} from "@tauri-apps/plugin-dialog";

interface FileMetadata {
  name: string;
  size: number;
}

interface SelectedFile {
  name: string;
  size: number;
  path: string;
}

interface ConversionResult {
  path: string;
  size: number;
}

const supportedFormats = {
  binary_ply: "Binary PLY",
  ascii_ply: "ASCII PLY",
  spz_v2: "SPZ (v2)",
  splat: "SPLAT",
  csv: "CSV",
};

const selectedFormat = ref<keyof typeof supportedFormats>("binary_ply");
const convertedFormat = ref<keyof typeof supportedFormats>("binary_ply");
const isConverting = ref(false);
const selectedFile = ref<SelectedFile | null>(null);
const conversionError = ref<string | null>(null);
const outputFileName = ref<string>("");
const conversionTime = ref<number | null>(null);
const convertedFileSize = ref<number | null>(null);

const tempFilePath = ref<string | null>(null);

function formatTime(ms: number): string {
  if (ms < 1000) {
    return `${ms.toFixed(2)}ms`;
  } else if (ms < 60000) {
    return `${(ms / 1000).toFixed(2)}s`;
  } else {
    return `${(ms / 60000).toFixed(2)}m`;
  }
}

const displaySupportedFormats = ["ply", "spz", "splat", "csv"];
const acceptedExtensions = displaySupportedFormats.join(", ");

function onDragOver(e: DragEvent) {
  e.preventDefault();
}

function onDrop(e: DragEvent) {
  e.preventDefault();
  const file = e.dataTransfer?.files?.item(0);
  const path = (file as any)?.path;
  if (file && path) {
    handleFileSelection({
      name: file.name,
      size: file.size,
      path: path,
    });
  }
}

async function browseFiles() {
  try {
    const path = await dialogOpen({
      multiple: false,
      title: "Select a File to Convert",
      filters: [
        {
          name: "Gaussian splatting files",
          extensions: displaySupportedFormats,
        },
      ],
    });

    if (path) {
      const metadata = await invoke<FileMetadata>("get_file_metadata", {
        path,
      });
      handleFileSelection({ ...metadata, path });
    }
  } catch (error) {
    console.error("Error opening file dialog:", error);
    conversionError.value = "Failed to open file.";
  }
}

function handleFileSelection(file: SelectedFile) {
  selectedFile.value = file;
  conversionError.value = null;
  tempFilePath.value = null;
  conversionTime.value = null;
  updateOutputFileName();
}

function updateOutputFileName() {
  if (!selectedFile.value) return;
  const baseName = selectedFile.value.name.replace(/\.[^/.]+$/, "");
  const extension =
    {
      splat: "splat",
      ascii_ply: "ply",
      binary_ply: "ply",
      spz_v2: "spz",
      csv: "csv",
    }[selectedFormat.value] || "bin";
  outputFileName.value = `${baseName}.${extension}`;
}

watch(selectedFormat, updateOutputFileName);

async function convertFile() {
  if (!selectedFile.value) return;

  isConverting.value = true;
  conversionError.value = null;
  tempFilePath.value = null;
  convertedFileSize.value = null;
  convertedFormat.value = selectedFormat.value;
  const startTime = performance.now();

  try {
    const result = await invoke<ConversionResult>("convert_to_temp_file", {
      inputPath: selectedFile.value.path,
      sourceFormat: selectedFile.value.name.split(".").pop() || "",
      targetFormat: selectedFormat.value,
    });

    tempFilePath.value = result.path;
    convertedFileSize.value = result.size;

    conversionTime.value = performance.now() - startTime;
  } catch (error) {
    conversionError.value =
      typeof error === "string"
        ? error
        : "An unknown conversion error occurred";
  } finally {
    isConverting.value = false;
  }
}

async function saveFile() {
  if (!tempFilePath.value) return;
  try {
    const savePath = await dialogSave({
      title: "Save Converted File",
      defaultPath: outputFileName.value,
      filters: [
        {
          name: `${supportedFormats[convertedFormat.value]} file`,
          extensions: [outputFileName.value.split(".").pop() || ""],
        },
      ],
    });

    if (savePath) {
      await invoke("save_converted_file", {
        tempPath: tempFilePath.value,
        finalPath: savePath,
      });
    }
  } catch (error) {
    conversionError.value = "Failed to save file.";
    console.error(error);
  }
}

function clearAll() {
  selectedFile.value = null;
  tempFilePath.value = null;
  convertedFileSize.value = null;
  conversionError.value = null;
  outputFileName.value = "";
  conversionTime.value = null;
}
</script>

<template>
  <div class="min-h-screen bg-gradient-to-br from-background to-muted/20 p-6">
    <div class="max-w-4xl mx-auto">
      <!-- Header -->
      <div class="text-center mb-8">
        <div class="flex items-center justify-center gap-1">
          <img src="/favicon.ico" class="w-15" />
          <h1 class="text-4xl font-bold text-foreground mb-2">GS Flux</h1>
        </div>
        <p class="text-muted-foreground">
          Convert between different Gaussian Splatting formats with ease
        </p>
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 w-full">
        <!-- Input Section -->
        <Card>
          <CardHeader>
            <CardTitle>Input File</CardTitle>
          </CardHeader>
          <CardContent v-auto-animate>
            <!-- Input area -->
            <button
              class="group relative rounded-lg border-2 border-dashed border-border p-8 text-center hover:border-primary/50 transition-colors w-full cursor-pointer mb-4"
              @click="browseFiles"
              @drop.prevent="onDrop"
              @dragover.prevent="onDragOver"
              :class="isConverting ? 'opacity-60 pointer-events-none' : ''"
            >
              <div class="pointer-events-none flex flex-col items-center gap-3">
                <div
                  class="w-12 h-12 rounded-full bg-primary/10 text-primary flex items-center justify-center"
                >
                  <Icon name="lucide:upload" class="w-6 h-6" />
                </div>
                <div class="space-y-2">
                  <p class="text-sm">
                    <span class="font-medium">Click to select</span> or drag and
                    drop a file
                  </p>
                  <p class="text-xs text-muted-foreground">
                    Supported formats: {{ acceptedExtensions }}
                  </p>
                </div>
              </div>
            </button>

            <!-- Selected file info -->
            <div v-if="selectedFile" class="bg-muted/50 rounded-lg p-2.5 px-3">
              <div class="flex items-center gap-3">
                <div
                  class="w-8 h-8 rounded-lg bg-green-100 text-green-600 flex items-center justify-center dark:bg-green-900/20 dark:text-green-400"
                >
                  <Icon name="lucide:file" class="w-4 h-4" />
                </div>
                <div class="flex-1">
                  <p class="font-medium text-foreground">
                    {{ selectedFile.name }}
                  </p>
                  <p class="text-sm text-muted-foreground">
                    {{ (selectedFile.size / 1024 / 1024).toFixed(2) }} MB
                  </p>
                </div>
                <Button variant="ghost" size="sm" @click="clearAll">
                  <Icon name="lucide:x" class="w-4 h-4" />
                </Button>
              </div>
            </div>
          </CardContent>
          <CardFooter class="flex justify-between items-center">
            <!-- Conversion settings -->
            <div>
              <label class="block text-sm font-medium text-foreground mb-2">
                Convert to format
              </label>
              <Select v-model="selectedFormat" :disabled="isConverting">
                <SelectTrigger>
                  <SelectValue placeholder="Select a format" />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectLabel>Formats</SelectLabel>
                    <SelectItem
                      v-for="(value, key) in supportedFormats"
                      :key="key"
                      :value="key"
                    >
                      {{ value }}
                    </SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>

            <Button
              @click="convertFile"
              :disabled="!selectedFile || isConverting"
              size="lg"
            >
              <Icon
                v-if="isConverting"
                name="lucide:loader-2"
                class="w-4 h-4 mr-2 animate-spin"
              />
              <Icon v-else name="lucide:zap" class="w-4 h-4 mr-2" />
              {{ isConverting ? "Converting..." : "Convert File" }}
            </Button>
          </CardFooter>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Output</CardTitle>
          </CardHeader>

          <CardContent>
            <!-- Conversion status -->
            <div v-if="isConverting" class="text-center py-8">
              <div
                class="w-12 h-12 rounded-full bg-primary/10 text-primary flex items-center justify-center mx-auto mb-4"
              >
                <Icon name="lucide:loader-2" class="w-6 h-6 animate-spin" />
              </div>
              <p class="text-muted-foreground">Converting your file...</p>
            </div>

            <!-- Error state -->
            <div v-else-if="conversionError" class="text-center py-8">
              <div
                class="w-12 h-12 rounded-full bg-red-100 text-red-600 flex items-center justify-center mx-auto mb-4 dark:bg-red-900/20 dark:text-red-400"
              >
                <Icon name="lucide:alert-circle" class="w-6 h-6" />
              </div>
              <p class="text-red-600 font-medium mb-2 dark:text-red-400">
                Conversion failed
              </p>
              <p class="text-sm text-muted-foreground">{{ conversionError }}</p>
            </div>

            <!-- Success state -->
            <div v-else-if="tempFilePath" class="space-y-4">
              <div
                class="bg-green-50 rounded-lg p-4 dark:bg-green-900/20 dark:border dark:border-green-800/50"
              >
                <div class="flex items-center gap-3">
                  <div
                    class="w-8 h-8 rounded-lg bg-green-100 text-green-600 flex items-center justify-center dark:bg-green-900/20 dark:text-green-400"
                  >
                    <Icon name="lucide:check" class="w-4 h-4" />
                  </div>
                  <div class="flex-1">
                    <p class="font-medium text-green-900 dark:text-green-100">
                      Conversion successful!
                    </p>
                    <p class="text-sm text-green-700 dark:text-green-300">
                      {{ outputFileName }}
                    </p>
                  </div>
                </div>
              </div>

              <div class="bg-muted/50 rounded-lg p-4">
                <div class="flex items-center justify-between mb-2">
                  <span class="text-sm font-medium text-foreground"
                    >File size</span
                  >
                  <span class="text-sm text-muted-foreground">
                    {{
                      convertedFileSize !== null
                        ? (convertedFileSize / 1024 / 1024).toFixed(2)
                        : "N/A"
                    }}
                    MB
                  </span>
                </div>
                <div class="flex items-center justify-between mb-2">
                  <span class="text-sm font-medium text-foreground"
                    >Format</span
                  >
                  <span class="text-sm text-muted-foreground">{{
                    supportedFormats[convertedFormat]
                  }}</span>
                </div>
                <div class="flex items-center justify-between">
                  <span class="text-sm font-medium text-foreground"
                    >Conversion time</span
                  >
                  <span class="text-sm text-muted-foreground">{{
                    conversionTime ? formatTime(conversionTime) : "N/A"
                  }}</span>
                </div>
              </div>

              <Button @click="saveFile" class="w-full" size="lg">
                <Icon name="lucide:download" class="w-4 h-4 mr-2" />
                Save {{ outputFileName }}
              </Button>
            </div>

            <!-- Empty state -->
            <div v-else class="text-center py-8 text-muted-foreground">
              <div
                class="w-12 h-12 rounded-full bg-muted text-muted-foreground flex items-center justify-center mx-auto mb-4"
              >
                <Icon name="lucide:file-down" class="w-6 h-6" />
              </div>
              <p>Convert a file to see the output here</p>
            </div>
          </CardContent>

          <CardFooter> </CardFooter>
        </Card>
      </div>
    </div>
  </div>
</template>
