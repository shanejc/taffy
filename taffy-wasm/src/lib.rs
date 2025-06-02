use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use taffy::{TaffyTree as Taffy, prelude::*, style::Style};
use wasm_bindgen::prelude::*;

// Import console.log for browser debugging
use web_sys::console;

// Set up console logging for WASM
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook to show panics in console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

// Simple logging function that works in both Node.js and browser
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Macro to use our WASM console.log function
macro_rules! wasm_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
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
                // Log the error and use default style instead of panicking
                web_sys::console::error_1(&format!("Style decode error in new_leaf: {}", e).into());
                web_sys::console::error_1(&"Using default style instead".into());
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
        let rs: JsStyle = match serde_wasm_bindgen::from_value(style) {
            Ok(style) => style,
            Err(e) => {
                web_sys::console::error_1(&format!("Style decode error: {}", e).into());
                return;
            }
        };
        let node = NodeId::from(node_id as u64);
        if let Err(e) = self.inner.borrow_mut().set_style(node, rs.0) {
            web_sys::console::error_1(&format!("Set style error: {:?}", e).into());
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
                    web_sys::console::error_1(&format!("Failed to serialize constraints: {}", e).into());
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
