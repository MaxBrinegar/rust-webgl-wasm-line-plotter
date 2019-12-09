use wasm_bindgen::prelude::*;

mod canvas;
use canvas::Canvas;

#[wasm_bindgen(start)]
pub fn start() {
    let canvas = Canvas::new("canvas", 750.0, 500.0);

    canvas.draw_line(300.0, 250.0, 400.0, 200.0, "black");
    canvas.draw_line(400.0, 200.0, 500.0, 400.0, "black");
    canvas.draw_line(500.0, 400.0, 100.0, 400.0, "black");
    canvas.draw_line(100.0, 400.0, 300.0, 350.0, "black");
    canvas.draw_line(300.0, 350.0, 300.0, 250.0, "black");

    canvas.draw_line(30.0, 30.0, 70.0, 30.0, "red");
    canvas.draw_line(30.0, 30.0, 30.0, 70.0, "red");
    canvas.draw_line(30.0, 70.0, 70.0, 70.0, "red");
    canvas.draw_line(70.0, 70.0, 70.0, 30.0, "red");
}
