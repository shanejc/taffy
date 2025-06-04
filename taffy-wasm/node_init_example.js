// Example of how to properly initialize taffy-wasm in Node.js/Vitest environments
// This should go in your satori-taffy project

import init from './path/to/taffy_wasm.js';
import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

// Detect if we're in Node.js environment
const isNodeEnvironment = typeof process !== 'undefined' && 
                          process.versions && 
                          process.versions.node;

export async function initializeWasm() {
    if (isNodeEnvironment) {
        // Node.js/Vitest environment - load WASM file manually
        const __filename = fileURLToPath(import.meta.url);
        const __dirname = dirname(__filename);
        
        // Adjust path to your actual WASM file location
        const wasmPath = join(__dirname, 'path/to/taffy_wasm_bg.wasm');
        const wasmBytes = readFileSync(wasmPath);
        
        // Initialize with bytes directly
        return await init(wasmBytes);
    } else {
        // Browser environment - let it fetch automatically
        return await init();
    }
}

// Alternative approach using try/catch
export async function initializeWasmFallback() {
    try {
        // Try automatic initialization first (works in browsers)
        return await init();
    } catch (error) {
        if (error.message.includes('fetch failed') && isNodeEnvironment) {
            // Fallback to manual loading for Node.js
            const { readFileSync } = await import('fs');
            const { fileURLToPath } = await import('url');
            const { dirname, join } = await import('path');
            
            const __filename = fileURLToPath(import.meta.url);
            const __dirname = dirname(__filename);
            const wasmPath = join(__dirname, 'path/to/taffy_wasm_bg.wasm');
            const wasmBytes = readFileSync(wasmPath);
            
            return await init(wasmBytes);
        }
        throw error; // Re-throw if it's a different error
    }
} 