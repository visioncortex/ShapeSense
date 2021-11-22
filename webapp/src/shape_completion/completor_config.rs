use super::{draw::DisplaySelector, CurveIntrapolatorConfig};
use wasm_bindgen::prelude::*;

/// Configuration to ShapeCompletor
#[wasm_bindgen]
pub struct ShapeCompletorAPIConfig {
    // Create a ShapeCompletor
    canvas_id: String,
    pub display_selector: DisplaySelector,
    pub display_tangents: bool,
    pub display_control_points: bool,
    pub hole_left: usize,
    pub hole_top: usize,
    pub hole_width: usize,
    pub hole_height: usize,

    // Simplify path segments
    pub simplify_tolerance: f64,

    // Curve intrrpolator
    pub curve_intrapolator_config: CurveIntrapolatorConfig,
}

impl Default for ShapeCompletorAPIConfig {
    fn default() -> Self {
        Self {
            canvas_id: "canvas_id".to_owned(),
            display_selector: DisplaySelector::None,
            display_tangents: false,
            display_control_points: false,
            hole_left: 0,
            hole_top: 0,
            hole_width: 15,
            hole_height: 15,
            simplify_tolerance: 2.0,
            curve_intrapolator_config: Default::default(),
        }
    }
}

// WASM API
#[wasm_bindgen]
#[allow(non_snake_case)]
impl ShapeCompletorAPIConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Self {
        Self::default().canvasId(canvas_id)
    }

    pub fn canvasId(mut self, value: &str) -> Self {
        self.canvas_id = value.to_owned();
        self
    }

    pub fn displaySelector(mut self, value: DisplaySelector) -> Self {
        self.display_selector = value;
        self
    }

    pub fn displayTangents(mut self, value: bool) -> Self {
        self.display_tangents = value;
        self
    }

    pub fn displayControlPoints(mut self, value: bool) -> Self {
        self.display_control_points = value;
        self
    }

    pub fn holeLeft(mut self, value: usize) -> Self {
        self.hole_left = value;
        self
    }

    pub fn holeTop(mut self, value: usize) -> Self {
        self.hole_top = value;
        self
    }

    pub fn holeWidth(mut self, value: usize) -> Self {
        self.hole_width = value;
        self
    }

    pub fn holeHeight(mut self, value: usize) -> Self {
        self.hole_height = value;
        self
    }

    pub fn holeRect(mut self, left: usize, top: usize, width: usize, height: usize) -> Self {
        self.hole_left = left;
        self.hole_top = top;
        self.hole_width = width;
        self.hole_height = height;
        self
    }

    pub fn pathSimplifyTolerance(mut self, value: f64) -> Self {
        self.simplify_tolerance = value;
        self
    }

    // CurveInterpolatorConfig

    pub fn curveOutsetRatio(mut self, value: f64) -> Self {
        self.curve_intrapolator_config.outset_ratio = value;
        self
    }

    pub fn curveMinSegmentLength(mut self, value: f64) -> Self {
        self.curve_intrapolator_config.min_segment_length = value;
        self
    }

    pub fn curveSmoothMaxIterations(mut self, value: usize) -> Self {
        self.curve_intrapolator_config.smooth_max_iterations = value;
        self
    }

    pub fn curveCornerThreshold(mut self, value: f64) -> Self {
        self.curve_intrapolator_config.corner_threshold = value;
        self
    }

    pub fn curveTailTangentNumPoints(mut self, value: usize) -> Self {
        self.curve_intrapolator_config.tail_tangent_num_points = value;
        self
    }

    pub fn curveTailWeightMultiplier(mut self, value: f64) -> Self {
        self.curve_intrapolator_config.tail_weight_multiplier = value;
        self
    }

    pub fn curveControlPointsRetractRatio(mut self, value: f64) -> Self {
        self.curve_intrapolator_config.control_points_retract_ratio = value;
        self
    }
}

// API
impl ShapeCompletorAPIConfig {
    pub fn get_canvas_id(&self) -> &str {
        &self.canvas_id
    }
}

// Helper functions
impl ShapeCompletorAPIConfig {}
