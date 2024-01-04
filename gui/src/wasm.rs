use std::num::NonZeroU32;

use cgmath::Vector2;
use wasm_bindgen::{closure::Closure, JsCast};

const CANVAS_ID: &'static str = "fractl";

pub fn window_size() -> Vector2<NonZeroU32> {
    let (default_x, default_y) = (640, 360);

    let size = if let Some(window) = web_sys::window() {
        Vector2::new(
            window.inner_width().unwrap().as_f64().unwrap() as u32,
            window.inner_height().unwrap().as_f64().unwrap() as u32,
        )
    } else {
        Vector2::new(default_x, default_y)
    };

    if let (Some(x), Some(y)) = (NonZeroU32::new(size.x), NonZeroU32::new(size.y)) {
        Vector2::new(x, y)
    } else {
        Vector2::new(NonZeroU32::new(default_x).unwrap(), NonZeroU32::new(default_y).unwrap())
    }
}

pub fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn document() -> web_sys::Document {
    window().document().expect("no global `document` exists")
}

pub fn body() -> web_sys::HtmlElement {
    document().body().expect("`document` has no body")
}

pub fn set_canvas_id() {
    let canvas_collection = document().get_elements_by_tag_name("canvas");

    if canvas_collection.length() == 1 {
        canvas_collection.item(0).unwrap().set_id(CANVAS_ID)
    } else {
        panic!("Too few or many `canvases` exist")
    }
}

pub fn get_element_by_id(id: &str) -> web_sys::HtmlElement {
    document()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("Element '{id:?}' does not exist"))
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap_or_else(|_| panic!("Could not dyn element '{id:?}' into HtmlElement"))
}

pub fn canvas_element() -> web_sys::HtmlCanvasElement {
    get_element_by_id(CANVAS_ID)
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .unwrap()
}

pub fn resize_canvas() {
    let size = window_size().map(|x| x.get());
    let canvas_element = canvas_element();

    canvas_element.set_width(size.x);
    canvas_element.set_height(size.y);
}

pub fn register_window_resize() {
    let closure = Closure::<dyn Fn()>::new(|| resize_canvas());

    window()
        .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
        .unwrap();

    closure.forget();
}
