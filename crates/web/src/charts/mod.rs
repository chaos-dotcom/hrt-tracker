#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ViewZoom {
    pub x_min: Option<f64>,
    pub x_max: Option<f64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChartTooltip {
    pub text: String,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct DragState {
    pub start_x: f64,
    pub start_min: f64,
    pub start_max: f64,
}

pub(crate) const CHART_MARGIN: f64 = 18.0;
pub(crate) const CHART_X_LABEL: f64 = 42.0;
pub(crate) const CHART_Y_LABEL: f64 = 52.0;

pub fn chart_padding() -> (f64, f64, f64, f64) {
    (
        CHART_MARGIN + CHART_Y_LABEL,
        CHART_MARGIN,
        CHART_MARGIN,
        CHART_MARGIN + CHART_X_LABEL,
    )
}

pub fn clamp_zoom(domain_min: f64, domain_max: f64, new_min: f64, new_max: f64) -> ViewZoom {
    let full_span = domain_max - domain_min;
    let span = (new_max - new_min).max(1.0);
    if span >= full_span * 0.98 {
        return ViewZoom::default();
    }
    let mut min_val = new_min;
    let mut max_val = new_min + span;
    if min_val < domain_min {
        min_val = domain_min;
        max_val = domain_min + span;
    }
    if max_val > domain_max {
        max_val = domain_max;
        min_val = domain_max - span;
    }
    ViewZoom {
        x_min: Some(min_val),
        x_max: Some(max_val),
    }
}

pub fn compute_chart_bounds(
    client_width: f64,
    client_height: f64,
    padding: (f64, f64, f64, f64),
    domain_min: f64,
    domain_max: f64,
    y_min: f64,
    y_max: f64,
) -> (f64, f64, f64, f64) {
    let (left, top, right, bottom) = padding;
    let width = (client_width - left - right).max(1.0);
    let height = (client_height - top - bottom).max(1.0);
    let domain_span = (domain_max - domain_min).abs().max(1.0);
    let y_span = (y_max - y_min).abs().max(1.0);
    (width, height, domain_span, y_span)
}

pub(crate) fn data_to_canvas_x(
    x: f64,
    domain_min: f64,
    domain_span: f64,
    width: f64,
    left: f64,
) -> f64 {
    left + ((x - domain_min) / domain_span) * width
}

pub(crate) fn data_to_canvas_y(y: f64, y_min: f64, y_span: f64, height: f64, top: f64) -> f64 {
    top + height - ((y - y_min) / y_span) * height
}

pub mod estrannaise;
pub mod view;
