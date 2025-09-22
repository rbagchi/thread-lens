import default_init, { parse_jstack_output_wasm, find_chronically_blocked_threads_wasm } from './thread-lens/pkg/thread_lens.js';

const outputElement = document.getElementById('output');

async function initializeWasm() {
    try {
        await default_init();
        outputElement.textContent = "Wasm module initialized. Ready for analysis.\n";
    } catch (e) {
        outputElement.textContent = `Error initializing Wasm: ${e}\n`;
        console.error("Error initializing Wasm:", e);
    }
}

async function handleSingleDumpAnalysis() {
    const input = document.getElementById('singleDumpInput');
    if (input.files.length === 0) {
        outputElement.textContent = "Please select a single .jstack file.\n";
        return;
    }

    const file = input.files[0];
    const reader = new FileReader();

    reader.onload = async (event) => {
        const sampleDump = event.target.result;
        outputElement.textContent = "--- Analyzing Single Thread Dump ---\n";
        try {
            const parsedDumpJson = parse_jstack_output_wasm(sampleDump);
            const parsedDump = JSON.parse(parsedDumpJson);
            outputElement.textContent += `Parsed Dump (JSON): ${JSON.stringify(parsedDump, null, 2)}\n`;
            outputElement.textContent += `Total threads in parsed dump: ${parsedDump.threads.length}\n`;
        } catch (e) {
            outputElement.textContent += `Error parsing dump: ${e}\n`;
            console.error("Error parsing dump:", e);
        }
    };

    reader.readAsText(file);
}

async function handleMultipleDumpsAnalysis() {
    const input = document.getElementById('multipleDumpsInput');
    if (input.files.length === 0) {
        outputElement.textContent = "Please select one or more .jstack files or a .zip file.\n";
        return;
    }

    const files = Array.from(input.files);
    const dumps = [];

    outputElement.textContent = "--- Analyzing Multiple Thread Dumps ---\n";

    const processFile = async (file) => {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            reader.onload = async (event) => {
                const dumpContent = event.target.result;
                try {
                    const parsedDumpJson = parse_jstack_output_wasm(dumpContent);
                    dumps.push(JSON.parse(parsedDumpJson));
                } catch (e) {
                    outputElement.textContent += `Error parsing file ${file.name}: ${e}\n`;
                    console.error(`Error parsing file ${file.name}:`, e);
                }
                resolve();
            };
            reader.onerror = (error) => reject(error);
            reader.readAsText(file);
        });
    };

    const processZipFile = async (zipFile) => {
        return new Promise(async (resolve, reject) => {
            try {
                const zip = await JSZip.loadAsync(zipFile);
                const jstackPromises = [];

                zip.forEach((relativePath, zipEntry) => {
                    if (!zipEntry.dir && relativePath.endsWith('.jstack')) {
                        jstackPromises.push(zipEntry.async('text').then(async (content) => {
                            try {
                                const parsedDumpJson = parse_jstack_output_wasm(content);
                                dumps.push(JSON.parse(parsedDumpJson));
                            } catch (e) {
                                outputElement.textContent += `Error parsing file ${relativePath} from zip: ${e}\n`;
                                console.error(`Error parsing file ${relativePath} from zip:`, e);
                            }
                        }));
                    }
                });

                await Promise.all(jstackPromises);
                resolve();
            } catch (e) {
                outputElement.textContent += `Error processing zip file ${zipFile.name}: ${e}\n`;
                console.error(`Error processing zip file ${zipFile.name}:`, e);
                reject(e);
            }
        });
    };

    const allPromises = [];

    for (const file of files) {
        if (file.type === 'application/zip' || file.name.endsWith('.zip')) {
            outputElement.textContent += `Processing zip file: ${file.name}...\n`;
            allPromises.push(processZipFile(file));
        } else if (file.name.endsWith('.jstack')) {
            outputElement.textContent += `Processing .jstack file: ${file.name}...\n`;
            allPromises.push(processFile(file));
        } else {
            outputElement.textContent += `Skipping unsupported file type: ${file.name}\n`;
        }
    }

    await Promise.all(allPromises);

    outputElement.textContent += `Found ${dumps.length} valid dumps.\n`;
    if (dumps.length < 2) {
        outputElement.textContent += "Not enough dumps (need at least 2) for chronic analysis.\n";
        return;
    }

    try {
        const dumpsArrayJson = JSON.stringify(dumps);
        const blockedThreadsJson = find_chronically_blocked_threads_wasm(dumpsArrayJson);
        const blockedThreads = JSON.parse(blockedThreadsJson);
        outputElement.textContent += `Chronically Blocked Threads (JSON): ${JSON.stringify(blockedThreads, null, 2)}\n`;
    } catch (e) {
        outputElement.textContent += `Error finding chronically blocked threads: ${e}\n`;
        console.error("Error finding chronically blocked threads:", e);
    }
}

document.addEventListener('DOMContentLoaded', () => {
    initializeWasm();

    document.getElementById('analyzeSingleDump').addEventListener('click', handleSingleDumpAnalysis);
    document.getElementById('analyzeMultipleDumps').addEventListener('click', handleMultipleDumpsAnalysis);
});