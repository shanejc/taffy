use std::path::Path;
use taffy::style::{AvailableSpace, Style};
use ts_rs::TS;

fn main() {
    // Use ts-rs's built-in export functionality to generate TypeScript definitions
    // This will export Style and all its dependencies to the pkg directory
    Style::export_all_to("pkg").unwrap();

    // Also explicitly export AvailableSpace since we use it in our measure function interface
    AvailableSpace::export_all_to("pkg").unwrap();

    // Generate CompactLength TypeScript declaration with helpers
    let compact_length_ts = r#"
// CompactLength TypeScript declaration and helpers
export type CompactLength = bigint;

export const CompactLength = {
  // Tag constants
  LENGTH_TAG: 0b0000_0001n,
  PERCENT_TAG: 0b0000_0010n,
  AUTO_TAG: 0b0000_0011n,
  FR_TAG: 0b0000_0100n,
  MIN_CONTENT_TAG: 0b0000_0111n,
  MAX_CONTENT_TAG: 0b0000_1111n,
  FIT_CONTENT_PX_TAG: 0b0001_0111n,
  FIT_CONTENT_PERCENT_TAG: 0b0001_1111n,

  // Helper to convert f32 to its bit representation
  _f32ToBits: (value: number): number => {
    const buffer = new ArrayBuffer(4);
    const view = new DataView(buffer);
    view.setFloat32(0, value, true); // little-endian
    return view.getUint32(0, true);
  },

  // Helper functions to create tagged values
  length: (value: number): CompactLength => {
    // Create the tagged value the same way Rust does: (value_bits << 32) | tag
    const valueBits = BigInt(CompactLength._f32ToBits(value)) << 32n;
    const taggedValue = valueBits | CompactLength.LENGTH_TAG;
    // Then rotate left 32 bits for serialization (matching Rust's .rotate_left(32))
    // For 64-bit rotate_left(32): take low 32 bits and move to high, take high 32 bits and move to low
    const low32 = taggedValue & 0xFFFFFFFFn;
    const high32 = (taggedValue >> 32n) & 0xFFFFFFFFn;
    return (low32 << 32n) | high32;
  },

  percent: (value: number): CompactLength => {
    const valueBits = BigInt(CompactLength._f32ToBits(value)) << 32n;
    const taggedValue = valueBits | CompactLength.PERCENT_TAG;
    const low32 = taggedValue & 0xFFFFFFFFn;
    const high32 = (taggedValue >> 32n) & 0xFFFFFFFFn;
    return (low32 << 32n) | high32;
  },

  auto: (): CompactLength => {
    const taggedValue = BigInt(CompactLength.AUTO_TAG);
    const low32 = taggedValue & 0xFFFFFFFFn;
    const high32 = (taggedValue >> 32n) & 0xFFFFFFFFn;
    return (low32 << 32n) | high32;
  },

  fr: (value: number): CompactLength => {
    const valueBits = BigInt(CompactLength._f32ToBits(value)) << 32n;
    const taggedValue = valueBits | CompactLength.FR_TAG;
    const low32 = taggedValue & 0xFFFFFFFFn;
    const high32 = (taggedValue >> 32n) & 0xFFFFFFFFn;
    return (low32 << 32n) | high32;
  },

  minContent: (): CompactLength => {
    const taggedValue = BigInt(CompactLength.MIN_CONTENT_TAG);
    const low32 = taggedValue & 0xFFFFFFFFn;
    const high32 = (taggedValue >> 32n) & 0xFFFFFFFFn;
    return (low32 << 32n) | high32;
  },
  
  maxContent: (): CompactLength => {
    const taggedValue = BigInt(CompactLength.MAX_CONTENT_TAG);
    const low32 = taggedValue & 0xFFFFFFFFn;
    const high32 = (taggedValue >> 32n) & 0xFFFFFFFFn;
    return (low32 << 32n) | high32;
  },

  fitContentPx: (value: number): CompactLength => {
    const valueBits = BigInt(CompactLength._f32ToBits(value)) << 32n;
    const taggedValue = valueBits | CompactLength.FIT_CONTENT_PX_TAG;
    const low32 = taggedValue & 0xFFFFFFFFn;
    const high32 = (taggedValue >> 32n) & 0xFFFFFFFFn;
    return (low32 << 32n) | high32;
  },

  fitContentPercent: (value: number): CompactLength => {
    const valueBits = BigInt(CompactLength._f32ToBits(value)) << 32n;
    const taggedValue = valueBits | CompactLength.FIT_CONTENT_PERCENT_TAG;
    const low32 = taggedValue & 0xFFFFFFFFn;
    const high32 = (taggedValue >> 32n) & 0xFFFFFFFFn;
    return (low32 << 32n) | high32;
  },

  // Helper to extract tag from CompactLength (after rotating right to undo serialization)
  getTag: (value: CompactLength): bigint => {
    // Rotate right 32 to undo the serialization rotation
    const low32 = value & 0xFFFFFFFFn;
    const high32 = (value >> 32n) & 0xFFFFFFFFn;
    const unrotated = (high32 << 32n) | low32;
    return unrotated & 0xFFn;
  },

  // Helper to extract numeric value from CompactLength (after rotating right to undo serialization)
  getValue: (value: CompactLength): number => {
    // Rotate right 32 to undo the serialization rotation
    const low32 = value & 0xFFFFFFFFn;
    const high32 = (value >> 32n) & 0xFFFFFFFFn;
    const unrotated = (high32 << 32n) | low32;
    const valueBits = Number(unrotated >> 32n);
    const view = new DataView(new ArrayBuffer(4));
    view.setUint32(0, valueBits, true); // little-endian
    return view.getFloat32(0, true);
  }
};
"#;

    std::fs::write("pkg/CompactLength.ts", compact_length_ts).expect("failed to write CompactLength.ts");

    // Post-process generated TypeScript files to add .js extensions to relative imports
    fix_import_extensions().expect("failed to fix import extensions");
}

fn fix_import_extensions() -> Result<(), Box<dyn std::error::Error>> {
    use regex::Regex;
    use std::fs;

    let pkg_dir = Path::new("pkg");
    if !pkg_dir.exists() {
        return Ok(());
    }

    // Regex to match relative imports without extensions
    // Matches: import ... from "./Something" or import ... from "../Something"
    // But not: import ... from "./Something.js" or import ... from "external-package"
    let import_regex = Regex::new(r#"(import\s+(?:.*?\s+from\s+)?["'])(\./[^"'/]*|../[^"'/]*)(["'])"#)?;

    // Process all .ts files in the pkg directory
    for entry in fs::read_dir(pkg_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("ts")
            && path.file_name().and_then(|s| s.to_str()) != Some("CompactLength.ts")
        {
            let content = fs::read_to_string(&path)?;

            // Replace relative imports to add .js extension
            let fixed_content = import_regex.replace_all(&content, |caps: &regex::Captures| {
                let prefix = &caps[1];
                let import_path = &caps[2];
                let suffix = &caps[3];

                // Only add .js if the path doesn't already have an extension
                if import_path
                    .rfind('.')
                    .map_or(false, |dot_idx| import_path.rfind('/').map_or(true, |slash_idx| dot_idx > slash_idx))
                {
                    // Path already has an extension, don't modify
                    format!("{}{}{}", prefix, import_path, suffix)
                } else {
                    // Add .js extension
                    format!("{}{}.js{}", prefix, import_path, suffix)
                }
            });

            // Write back if content changed
            if fixed_content != content {
                fs::write(&path, fixed_content.as_ref())?;
            }
        }
    }

    Ok(())
}
