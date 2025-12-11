use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

// Console logging macro
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

// Fractal viewport state
#[derive(Clone, Copy)]
struct Viewport {
    center_x: f64,
    center_y: f64,
    range: f64,   // vertical range; horizontal range = range * aspect
    width: u32,
    height: u32,
    max_iter: u32,
}

thread_local! {
    static VIEWPORT: std::cell::RefCell<Viewport> = std::cell::RefCell::new(Viewport {
        center_x: -0.5,   // center Mandelbrot
        center_y: 0.0,
        range: 3.0,       // initial visible range
        width: 800,
        height: 600,
        max_iter: 256,
    });
    static CTX: std::cell::RefCell<Option<CanvasRenderingContext2d>> = std::cell::RefCell::new(None);
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    log!("WASM initialized");
}

#[wasm_bindgen]
pub fn init(canvas_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("no document"))?;

    let element = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str("canvas not found"))?;

    let canvas: HtmlCanvasElement = element.dyn_into()?;

    let ctx = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("no 2D context"))?
        .dyn_into::<CanvasRenderingContext2d>()?;

    VIEWPORT.with(|v| {
        let mut vp = v.borrow_mut();
        vp.width = canvas.width();
        vp.height = canvas.height();
    });

    CTX.with(|c| *c.borrow_mut() = Some(ctx));

    render()
}

#[wasm_bindgen]
pub fn render() -> Result<(), JsValue> {
    let (viewport, ctx) = get_state_and_ctx()?;

    let width = viewport.width as usize;
    let height = viewport.height as usize;

    let mut data = vec![0u8; width * height * 4];

    for py in 0..height {
        for px in 0..width {
            let (re, im) = map_pixel_to_complex(px as f64, py as f64, &viewport);
            let iter = calculate_mandelbrot(re, im, viewport.max_iter);
            let (r, g, b) = get_color(iter, viewport.max_iter);

            let idx = (py * width + px) * 4;
            data[idx] = r;
            data[idx + 1] = g;
            data[idx + 2] = b;
            data[idx + 3] = 255;
        }
    }

    let clamped = wasm_bindgen::Clamped(&data[..]);
    let image_data = ImageData::new_with_u8_clamped_array_and_sh(
        clamped,
        viewport.width,
        viewport.height,
    )?;

    ctx.put_image_data(&image_data, 0.0, 0.0)?;
    Ok(())
}

#[wasm_bindgen]
pub fn zoom_at(x: f64, y: f64, zoom_factor: f64) -> Result<(), JsValue> {
    VIEWPORT.with(|v| {
        let mut vp = v.borrow_mut();
        let (re, im) = map_pixel_to_complex(x, y, &vp);
        vp.center_x = re;
        vp.center_y = im;
        vp.range /= zoom_factor;
    });

    render()
}

fn get_state_and_ctx() -> Result<(Viewport, CanvasRenderingContext2d), JsValue> {
    let viewport = VIEWPORT.with(|v| *v.borrow());
    let ctx = CTX.with(|c| {
        c.borrow()
            .as_ref()
            .ok_or_else(|| JsValue::from_str("context not initialized"))
            .cloned()
    })?;
    Ok((viewport, ctx))
}

fn map_pixel_to_complex(px: f64, py: f64, vp: &Viewport) -> (f64, f64) {
    let w = vp.width as f64;
    let h = vp.height as f64;
    let aspect = w / h;

    let range_y = vp.range;
    let range_x = vp.range * aspect;

    let x = (px / w - 0.5) * range_x + vp.center_x;
    let y = (0.5 - py / h) * range_y + vp.center_y;

    (x, y)
}

fn calculate_mandelbrot(re0: f64, im0: f64, max_iter: u32) -> u32 {
    let mut re = 0.0;
    let mut im = 0.0;
    for n in 0..max_iter {
        let re2 = re * re;
        let im2 = im * im;

        if re2 + im2 > 4.0 {
            return n;
        }

        im = 2.0 * re * im + im0;
        re = re2 - im2 + re0;
    }
    max_iter
}

fn get_color(iter: u32, max_iter: u32) -> (u8, u8, u8) {
    if iter == max_iter {
        return (0, 0, 0);
    }

    let t = (iter as f64 / max_iter as f64).powf(0.5);

    let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u8;
    let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u8;
    let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u8;

    (r, g, b)
}
