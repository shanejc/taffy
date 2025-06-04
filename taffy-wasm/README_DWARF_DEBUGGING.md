# DWARF Debugging Setup for Taffy WASM

This guide will help you set up full DWARF debugging for WebAssembly in Chrome DevTools.

## 🔧 Prerequisites

### 1. Install Chrome DevTools DWARF Extension

Install the **C/C++ DevTools Support (DWARF)** extension from the Chrome Web Store:
https://chromewebstore.google.com/detail/cc++-devtools-support-dwa/pdcpmagijalfljmkmjngeonclgbbannb

### 2. Enable DWARF Support in DevTools

1. Open Chrome DevTools (F12)
2. Go to **Settings** (⚙️ icon)  
3. Go to **Experiments** tab
4. Enable: **"WebAssembly Debugging: Enable DWARF support"**
5. Restart DevTools (close and reopen)

## 🚀 Build Configuration

The `Cargo.toml` has been configured with:

```toml
# Configure wasm-pack to preserve DWARF debug info in dev builds
[package.metadata.wasm-pack.profile.dev.wasm-bindgen]
dwarf-debug-info = true

[profile.dev]
debug = 2        # Full debug info including DWARF
opt-level = 0    # No optimizations
```

## 🔨 Building for Debug

Use the proper build command that preserves DWARF symbols:

```bash
npm run build:dev
# or directly:
wasm-pack build --target web --dev
```

⚠️ **Important**: Do NOT use `--release` flag when you want debugging capabilities!

## 🐛 Debugging Steps

1. **Build with debug symbols**:
   ```bash
   npm run build:dev
   ```

2. **Serve your application** (e.g., `http-server` or similar)

3. **Open in Chrome** (not Firefox - DWARF debugging requires Chrome)

4. **Open Chrome DevTools** (F12)

5. **Go to Sources tab**

6. **Set breakpoints** in your Rust source files (they should appear in the file tree)

7. **Reload the page** - execution should pause at your breakpoints!

## 🎯 What You Get With DWARF Debugging

- ✅ **Step through original Rust source code** (not WASM assembly)
- ✅ **View variable values** by hovering over them
- ✅ **Inspect the call stack** with proper function names
- ✅ **Set breakpoints** in Rust source files
- ✅ **Inspect memory** with the Memory Inspector
- ✅ **Evaluate expressions** in the console

## 🔍 Troubleshooting

### Issue: "Could not load source" errors

**Root cause**: This happens when debug paths don't match your actual file structure.

**Solutions**:
1. **Use the Chrome extension settings**:
   - Go to `chrome://extensions/`
   - Find "C/C++ DevTools Support (DWARF)"
   - Click **"Details"** → **"Extension options"**
   - Map old paths to new paths

2. **Build and run on the same machine** (avoid Docker/containers if possible)

### Issue: Only seeing WebAssembly assembly

**Cause**: DWARF debug info not included in build.

**Fix**: 
- Ensure you're using `npm run build:dev` (not `npm run build`)
- Check that `dwarf-debug-info = true` is in your `Cargo.toml`
- Verify the Chrome extension is installed and enabled

### Issue: Breakpoints not working

**Fix**:
- Reload the page after setting breakpoints
- Make sure you're using Chrome (not Firefox)
- Ensure the DWARF extension is installed
- Check that DevTools experiments are enabled

## 📚 Advanced Features

### Memory Inspector
- Click the memory icon (🔧) next to variables in the Scope panel
- Inspect raw memory bytes of WebAssembly objects
- See highlighted memory regions for complex data structures

### Performance Profiling
For production debugging, use:
```bash
npm run build:profiling
```
This includes line-table debug info but with optimizations enabled.

## 🌐 References

- [Chrome DevTools WebAssembly Debugging](https://developer.chrome.com/docs/devtools/wasm)
- [DWARF for WebAssembly Specification](https://yurydelendik.github.io/webassembly-dwarf/)
- [Working Demo Repository](https://github.com/haraldreingruber-dedalus/rust-wasm-dwarf-debugging)

## 🆘 If All Else Fails

1. Try the [working demo](https://github.com/haraldreingruber-dedalus/rust-wasm-dwarf-debugging) to verify your setup
2. Check Chrome version (needs 88+)
3. Try Chrome Canary if you're having issues with stable Chrome
4. File bugs at https://bugs.chromium.org/p/chromium/issues/entry?template=DevTools+issue

---

Happy debugging! 🦀✨ 