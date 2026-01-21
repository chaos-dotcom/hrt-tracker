use leptos::window;
use chrono::{Local, TimeZone};
use plotters::element::DashedPathElement;
use plotters::prelude::*;
use plotters_canvas::CanvasBackend;
use wasm_bindgen::JsCast;
use web_sys::HtmlCanvasElement;

use crate::charts::{
    data_to_canvas_x, data_to_canvas_y, ChartTooltip, ViewZoom, CHART_MARGIN, CHART_X_LABEL,
    CHART_Y_LABEL,
};

#[derive(Clone, Debug, PartialEq)]
pub struct EstrannaisePoint {
    pub x: f64,
    pub y: f64,
    pub label: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EstrannaiseSeries {
    pub blended: Vec<EstrannaisePoint>,
    pub stepped: Vec<EstrannaisePoint>,
    pub blood: Vec<EstrannaisePoint>,
    pub forecast: Option<(f64, f64)>,
    pub step_split_x: Option<f64>,
    pub domain_min: f64,
    pub domain_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub x_label: String,
    pub y_label: String,
    pub first_dose: Option<i64>,
    pub use_days: bool,
}

pub fn find_nearest_estrannaise_point(
    points: &[EstrannaisePoint],
    domain_min: f64,
    domain_span: f64,
    y_min: f64,
    y_span: f64,
    width: f64,
    height: f64,
    padding: (f64, f64, f64, f64),
    cursor_x: f64,
    cursor_y: f64,
) -> Option<(ChartTooltip, f64)> {
    let (left, top, _, _) = padding;
    let mut best: Option<(f64, &EstrannaisePoint, f64, f64)> = None;
    for point in points {
        let px = data_to_canvas_x(point.x, domain_min, domain_span, width, left);
        let py = data_to_canvas_y(point.y, y_min, y_span, height, top);
        let dx = px - cursor_x;
        let dy = py - cursor_y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < 18.0 {
            match best {
                Some((best_dist, _, _, _)) if dist >= best_dist => {}
                _ => best = Some((dist, point, px, py)),
            }
        }
    }
    best.map(|(dist, point, px, py)| {
        (
            ChartTooltip {
                text: point.label.clone(),
                x: px,
                y: py,
            },
            dist,
        )
    })
}

pub fn draw_estrannaise_chart(canvas_id: &str, series: &EstrannaiseSeries, zoom: ViewZoom) {
    let Some(canvas) = window()
        .document()
        .and_then(|doc| doc.get_element_by_id(canvas_id))
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
    else {
        return;
    };
    let rect = canvas.get_bounding_client_rect();
    let width = rect.width().max(320.0);
    let height = rect.height().max(280.0);
    let dpr = window().device_pixel_ratio();
    canvas.set_width((width * dpr) as u32);
    canvas.set_height((height * dpr) as u32);

    let backend = CanvasBackend::with_canvas_object(canvas)
        .expect("canvas backend")
        .into_drawing_area();
    backend.fill(&RGBColor(15, 17, 26)).ok();

    let x_min = zoom.x_min.unwrap_or(series.domain_min);
    let x_max = zoom.x_max.unwrap_or(series.domain_max);
    let mut chart = match ChartBuilder::on(&backend)
        .margin(CHART_MARGIN as i32)
        .x_label_area_size((CHART_X_LABEL * 1.1) as i32)
        .y_label_area_size((CHART_Y_LABEL * 1.1) as i32)
        .build_cartesian_2d(x_min..x_max, series.y_min..series.y_max)
    {
        Ok(chart) => chart,
        Err(_) => return,
    };

    let label_style = ("Quicksand", 24)
        .into_font()
        .color(&RGBColor(200, 188, 214));
    let axis_desc_style = ("Quicksand", 24)
        .into_font()
        .color(&RGBColor(200, 188, 214));
    let bold_grid = ShapeStyle::from(&RGBColor(64, 58, 86)).stroke_width(1);
    let light_grid = ShapeStyle::from(&RGBColor(42, 38, 60)).stroke_width(1);
    let use_days = series.use_days;
    let x_label_formatter = move |value: &f64| {
        if use_days {
            format!("Day {}", value.round() as i64)
        } else {
            Local
                .timestamp_millis_opt(*value as i64)
                .single()
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| format!("{:.0}", value))
        }
    };

    chart
        .configure_mesh()
        .x_labels(6)
        .y_labels(6)
        .x_label_formatter(&x_label_formatter)
        .label_style(label_style)
        .axis_desc_style(axis_desc_style)
        .axis_style(&RGBColor(96, 86, 120))
        .bold_line_style(bold_grid)
        .light_line_style(light_grid)
        .x_desc(series.x_label.clone())
        .y_desc(series.y_label.clone())
        .draw()
        .ok();

    if let Some((start, end)) = series.forecast {
        chart
            .draw_series(std::iter::once(Rectangle::new(
                [(start, series.y_min), (end, series.y_max)],
                RGBAColor(246, 193, 119, 0.12).filled(),
            )))
            .ok();
    }

    if !series.blended.is_empty() {
        let blended_style = ShapeStyle::from(&RGBColor(46, 134, 171)).stroke_width(2);
        if let Some((split_x, _)) = series.forecast {
            let mut historical = Vec::new();
            let mut forecast = Vec::new();
            for point in &series.blended {
                if point.x < split_x {
                    historical.push((point.x, point.y));
                } else {
                    forecast.push((point.x, point.y));
                }
            }
            if historical.len() > 1 {
                chart
                    .draw_series(LineSeries::new(historical, blended_style))
                    .ok();
            }
            if forecast.len() > 1 {
                chart
                    .draw_series(LineSeries::new(forecast, blended_style))
                    .ok();
            }
        } else {
            let line = series.blended.iter().map(|p| (p.x, p.y));
            chart
                .draw_series(LineSeries::new(line, blended_style))
                .ok();
        }
    }
    if !series.stepped.is_empty() {
        let step_style = ShapeStyle::from(&RGBColor(162, 59, 114)).stroke_width(2);
        if let Some(split_x) = series.step_split_x {
            let mut historical = Vec::new();
            let mut forecast = Vec::new();
            for point in &series.stepped {
                if point.x < split_x {
                    historical.push((point.x, point.y));
                } else {
                    forecast.push((point.x, point.y));
                }
            }
            if historical.len() > 1 {
                chart
                    .draw_series(std::iter::once(DashedPathElement::new(
                        historical,
                        6,
                        4,
                        step_style,
                    )))
                    .ok();
            }
            if forecast.len() > 1 {
                chart
                    .draw_series(std::iter::once(DashedPathElement::new(
                        forecast,
                        2,
                        6,
                        step_style,
                    )))
                    .ok();
            }
        } else {
            let line = series.stepped.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>();
            if line.len() > 1 {
                chart
                    .draw_series(std::iter::once(DashedPathElement::new(
                        line,
                        6,
                        4,
                        step_style,
                    )))
                    .ok();
            }
        }
    }
    for point in &series.blood {
        chart
            .draw_series(std::iter::once(Circle::new(
                (point.x, point.y),
                4,
                RGBColor(255, 165, 0).filled(),
            )))
            .ok();
    }

    backend.present().ok();
}
