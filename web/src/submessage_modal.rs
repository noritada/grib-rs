use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, ImageData};
use yew::prelude::*;

const SUBMESSAGE_MODAL_ID: &str = "submessage-modal";

#[derive(Properties, PartialEq)]
pub struct SubmessageModalProps {
    pub image_data: Option<ImageData>,
    pub on_click: Callback<MouseEvent>,
}

#[function_component(SubmessageModal)]
pub(crate) fn submessage_modal(
    SubmessageModalProps {
        image_data,
        on_click,
    }: &SubmessageModalProps,
) -> Html {
    let context = use_state(|| None);
    let context_ = context.clone();

    {
        let context_ = context.clone();
        use_effect_with(context, move |_| {
            context_.set(get_canvas_context());
        });
    }

    if let Some(context) = context_.as_ref() {
        if let Some(image_data) = image_data {
            let _ = context.put_image_data(&image_data, 0., 0.);
        }
    }

    html! {
        <div id={SUBMESSAGE_MODAL_ID} class="invisible" onclick={on_click}>
            <div id="submessage-details">
                <canvas id="grid-canvas"></canvas>
            </div>
        </div>
    }
}

pub(crate) fn display_submessage_modal() {
    if let Some(classes) = crate::utils::get_classes(SUBMESSAGE_MODAL_ID) {
        classes
            .remove_1("invisible")
            .expect("removing class 'invisible' failed")
    }
}

pub(crate) fn hide_submessage_modal() {
    if let Some(classes) = crate::utils::get_classes(SUBMESSAGE_MODAL_ID) {
        classes
            .add_1("invisible")
            .expect("adding class 'invisible' failed")
    }
}

fn get_canvas_context() -> Option<CanvasRenderingContext2d> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let canvas = document.get_element_by_id("grid-canvas")?;
    let canvas: web_sys::HtmlCanvasElement =
        canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()?;
    canvas
        .get_context("2d")
        .ok()??
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()
}
