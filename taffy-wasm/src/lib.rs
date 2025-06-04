use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use taffy::{prelude::*, style::Style, TaffyTree as Taffy};
use wasm_bindgen::prelude::*;

// Re-export grid types for TypeScript generation
pub use taffy::style::{
    ConcreteGridPlacement, GridTrackRepetition, SimpleMaxTrackSizingFunction, SimpleMinTrackSizingFunction,
    SimpleNonRepeatedTrackSizingFunction, SimpleTrackSizingFunction,
};

// Console logging setup for different environments
#[cfg(feature = "browser-console")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// For Node.js with console output - provide console.log binding that client can implement
#[cfg(all(feature = "node-console", feature = "wasm-console"))]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Pure Node.js without console output
#[cfg(all(feature = "node-console", not(feature = "wasm-console")))]
fn log(s: &str) {
    // In Node.js environments, use println! which gets handled by wasm-bindgen properly
    println!("{}", s);
}

#[cfg(not(any(feature = "browser-console", feature = "node-console")))]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Set up console logging for WASM
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook to show panics in console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

// Macro to use our logging function - only active in debug builds
#[cfg(all(feature = "node-console", not(feature = "wasm-console")))]
macro_rules! wasm_log {
    ($($t:tt)*) => {
        #[cfg(debug_assertions)]
        println!("{}", format!($($t)*))
    }
}

#[cfg(not(all(feature = "node-console", not(feature = "wasm-console"))))]
macro_rules! wasm_log {
    ($($t:tt)*) => {
        #[cfg(debug_assertions)]
        log(&format!($($t)*))
    }
}

/// Thin, easily‑serialised copy of `Style`
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JsStyle(pub Style);

/// Context data for JavaScript - can hold any JS value
pub struct JsContext {
    data: JsValue,
}

#[wasm_bindgen]
pub struct TaffyTree {
    inner: RefCell<Taffy<JsContext>>,
}

impl Default for TaffyTree {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl TaffyTree {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: RefCell::new(Taffy::new()) }
    }

    /// Create a leaf from a JS object `{display:"flex", flexDirection:"row", …}`
    #[wasm_bindgen]
    pub fn new_leaf(&self, style: JsValue) -> u32 {
        let rs: JsStyle = match serde_wasm_bindgen::from_value(style) {
            Ok(style) => style,
            Err(e) => {
                wasm_log!("🚀 WASM: Style decode error in new_leaf: {}", e);
                JsStyle(Style::default())
            }
        };
        let node = self.inner.borrow_mut().new_leaf(rs.0).unwrap();
        u64::from(node) as u32
    }

    #[wasm_bindgen]
    pub fn add_child(&self, parent: u32, child: u32) {
        let parent = NodeId::from(parent as u64);
        let child = NodeId::from(child as u64);
        self.inner.borrow_mut().add_child(parent, child).unwrap();
    }

    #[wasm_bindgen]
    pub fn update_style(&self, node_id: u32, style: JsValue) {
        // Add explicit console logging for debugging
        #[cfg(feature = "node-console")]
        web_sys::console::log_1(&format!("🚀 WASM: update_style called for node {}", node_id).into());

        let rs: JsStyle = match serde_wasm_bindgen::from_value(style) {
            Ok(style) => {
                #[cfg(feature = "node-console")]
                web_sys::console::log_1(&"✅ WASM: Style deserialized successfully".into());
                style
            }
            Err(e) => {
                // Use both wasm_log! and direct console logging
                wasm_log!("🚀 WASM: Style decode error in update_style: {}", e);
                #[cfg(feature = "node-console")]
                web_sys::console::error_1(&format!("❌ WASM: Style decode error in update_style: {}", e).into());
                return;
            }
        };
        let node = NodeId::from(node_id as u64);

        #[cfg(feature = "node-console")]
        web_sys::console::log_1(&format!("🚀 WASM: About to call set_style for node {}", node_id).into());

        if let Err(e) = self.inner.borrow_mut().set_style(node, rs.0) {
            wasm_log!("🚀 WASM: Set style error: {}", e);
            #[cfg(feature = "node-console")]
            web_sys::console::error_1(&format!("❌ WASM: Set style error: {}", e).into());
        } else {
            #[cfg(feature = "node-console")]
            web_sys::console::log_1(&"✅ WASM: set_style completed successfully".into());
        }
    }

    #[wasm_bindgen]
    pub fn compute_layout(&self, node_id: u32, width: f32, height: f32) {
        wasm_log!("🚀 WASM: Starting compute_layout for node {} with size {}x{}", node_id, width, height);
        let node = NodeId::from(node_id as u64);
        let available_space = Size { width: AvailableSpace::Definite(width), height: AvailableSpace::Definite(height) };
        self.inner.borrow_mut().compute_layout(node, available_space).unwrap();
        wasm_log!("✅ WASM: Finished compute_layout for node {}", node_id);
    }

    #[wasm_bindgen]
    pub fn compute_layout_with_measure(&self, node_id: u32, width: f32, height: f32, measure_func: &js_sys::Function) {
        let node = NodeId::from(node_id as u64);
        let available_space = Size { width: AvailableSpace::Definite(width), height: AvailableSpace::Definite(height) };

        let measure_function = |known_dimensions: Size<Option<f32>>,
                                available_space: Size<AvailableSpace>,
                                _node_id: NodeId,
                                node_context: Option<&mut JsContext>,
                                _style: &Style|
         -> Size<f32> {
            // Get the context data (or null if no context)
            let null_value = JsValue::NULL;
            let context_data = node_context.map(|ctx| &ctx.data).unwrap_or(&null_value);

            // Calculate the effective available space for measure function
            // If we have a known dimension, use Definite with that value
            // Otherwise, use the available space as provided by the parent
            let effective_width = known_dimensions.width.map(AvailableSpace::Definite).unwrap_or(available_space.width);
            let effective_height =
                known_dimensions.height.map(AvailableSpace::Definite).unwrap_or(available_space.height);

            // Create the constraints using Size<AvailableSpace>
            let constraints = Size { width: effective_width, height: effective_height };

            wasm_log!(
                "🚀 WASM: Measuring with constraints: width={:?}, height={:?}",
                constraints.width,
                constraints.height
            );

            // Serialize constraints to JsValue for passing to JavaScript
            let constraints_js = match serde_wasm_bindgen::to_value(&constraints) {
                Ok(value) => value,
                Err(e) => {
                    wasm_log!("🚀 WASM: Failed to serialize constraints: {}", e);
                    return Size::ZERO;
                }
            };

            // Call the JavaScript function with (contextData, constraints)
            match measure_func.call2(&JsValue::NULL, context_data, &constraints_js) {
                Ok(result) => {
                    // Try to parse the result as {width: number, height: number}
                    if result.is_object() {
                        let width_prop = js_sys::Reflect::get(&result, &"width".into()).unwrap_or(JsValue::from(0.0));
                        let height_prop = js_sys::Reflect::get(&result, &"height".into()).unwrap_or(JsValue::from(0.0));

                        let width = width_prop.as_f64().unwrap_or(0.0) as f32;
                        let height = height_prop.as_f64().unwrap_or(0.0) as f32;

                        Size { width, height }
                    } else {
                        // Fallback: try to parse as array [width, height]
                        if js_sys::Array::is_array(&result) {
                            let array = js_sys::Array::from(&result);
                            let width = array.get(0).as_f64().unwrap_or(0.0) as f32;
                            let height = array.get(1).as_f64().unwrap_or(0.0) as f32;
                            Size { width, height }
                        } else {
                            Size { width: 0.0, height: 0.0 }
                        }
                    }
                }
                Err(_) => Size { width: 0.0, height: 0.0 },
            }
        };

        self.inner.borrow_mut().compute_layout_with_measure(node, available_space, measure_function).unwrap();
    }

    #[wasm_bindgen]
    pub fn set_node_context(&self, node_id: u32, data: &JsValue) {
        let node = NodeId::from(node_id as u64);
        let context = JsContext { data: data.clone() };
        self.inner.borrow_mut().set_node_context(node, Some(context)).unwrap();
    }

    #[wasm_bindgen]
    pub fn remove_node_context(&self, node_id: u32) {
        let node = NodeId::from(node_id as u64);
        self.inner.borrow_mut().set_node_context(node, None).unwrap();
    }

    #[wasm_bindgen]
    pub fn layout_left(&self, node_id: u32) -> f32 {
        self.inner.borrow().layout(NodeId::from(node_id as u64)).unwrap().location.x
    }

    #[wasm_bindgen]
    pub fn layout_top(&self, node_id: u32) -> f32 {
        self.inner.borrow().layout(NodeId::from(node_id as u64)).unwrap().location.y
    }

    #[wasm_bindgen]
    pub fn layout_width(&self, node_id: u32) -> f32 {
        self.inner.borrow().layout(NodeId::from(node_id as u64)).unwrap().size.width
    }

    #[wasm_bindgen]
    pub fn layout_height(&self, node_id: u32) -> f32 {
        self.inner.borrow().layout(NodeId::from(node_id as u64)).unwrap().size.height
    }

    // …add other helpers you need (top, width, height, etc.)
}

// Force TypeScript generation of grid types by including them in public API
// These functions are never called but ensure the types get exported

/// Get the default TrackSizingFunction for TypeScript export
#[wasm_bindgen]
pub fn get_default_track_sizing_function() -> JsValue {
    use taffy::style::TrackSizingFunction;
    let default_track = TrackSizingFunction::AUTO;
    serde_wasm_bindgen::to_value(&default_track).unwrap_or(JsValue::NULL)
}

/// Get the default NonRepeatedTrackSizingFunction for TypeScript export  
#[wasm_bindgen]
pub fn get_default_non_repeated_track_sizing_function() -> JsValue {
    use taffy::style::NonRepeatedTrackSizingFunction;
    let default_track = NonRepeatedTrackSizingFunction::AUTO;
    serde_wasm_bindgen::to_value(&default_track).unwrap_or(JsValue::NULL)
}

/// Get the default GridPlacement for TypeScript export
#[wasm_bindgen]
pub fn get_default_grid_placement() -> JsValue {
    use taffy::style::GridPlacement;
    let default_placement = GridPlacement::Auto;
    serde_wasm_bindgen::to_value(&default_placement).unwrap_or(JsValue::NULL)
}

/// Get the default MinTrackSizingFunction for TypeScript export
#[wasm_bindgen]
pub fn get_default_min_track_sizing_function() -> JsValue {
    use taffy::style::MinTrackSizingFunction;
    let default_min = MinTrackSizingFunction::AUTO;
    serde_wasm_bindgen::to_value(&default_min).unwrap_or(JsValue::NULL)
}

/// Get the default MaxTrackSizingFunction for TypeScript export
#[wasm_bindgen]
pub fn get_default_max_track_sizing_function() -> JsValue {
    use taffy::style::MaxTrackSizingFunction;
    let default_max = MaxTrackSizingFunction::AUTO;
    serde_wasm_bindgen::to_value(&default_max).unwrap_or(JsValue::NULL)
}

/// Get the default GridTrackRepetition for TypeScript export
#[wasm_bindgen]
pub fn get_default_grid_track_repetition() -> JsValue {
    use taffy::style::GridTrackRepetition;
    let default_repetition = GridTrackRepetition::AutoFill;
    serde_wasm_bindgen::to_value(&default_repetition).unwrap_or(JsValue::NULL)
}
