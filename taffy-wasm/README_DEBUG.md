# Debug Output in Node.js/Vitest

To see detailed debug output from Taffy's layout algorithms in Node.js or Vitest environments:

## 1. Build taffy-wasm with debug features

```bash
pnpm build:dev
```

This builds with `taffy/debug` and `taffy/wasm-console` features enabled, which makes taffy core use console bindings instead of `println!` (which doesn't work in WASM). The `--debug` flag preserves DWARF debugging symbols.

## 2. Load WASM manually in Node.js

Since Node.js doesn't support `fetch()` with file:// URLs, you need to load the WASM manually:

```javascript
import init, { TaffyTree } from 'taffy-wasm';
import fs from 'fs';

async function test() {
  // Load WASM manually for Node.js
  const wasmBytes = fs.readFileSync('./pkg/taffy_wasm_bg.wasm');
  await init(wasmBytes);
  
  const tree = new TaffyTree();
  const node = tree.new_leaf({
    display: 'flex',
    width: { unit: 'points', value: 100 },
    height: { unit: 'points', value: 100 }
  });
  
  // This will show detailed debug output from taffy core
  tree.compute_layout(node, 200, 200);
}

test().catch(console.error);
```

## 3. Expected Output

You should see detailed debug output like:

```
NodeId(1): PerformLayout
NodeId(1): sizing_mode InherentSize
NodeId(1): known_dimensions Size { width: None, height: None }
NodeId(1): parent_size Size { width: Some(200.0), height: Some(200.0) }
NodeId(1): available_space Size { width: Definite(200.0), height: Definite(200.0) }
NodeId(1): FLEX
NodeId(1): LEAF
NodeId(1): RESULT Size { width: 0.0, height: 0.0 }
```

## Key Points

- **Both console output AND DWARF debugging work together** with `pnpm build:dev`
- **`println!` doesn't work in WASM** - the standard library discards the output
- **`taffy/wasm-console` feature** makes taffy core use console bindings instead
- **`--debug` flag** preserves DWARF debugging symbols for debugger support
- **Manual WASM loading** is required in Node.js since `fetch()` doesn't work with file:// URLs 