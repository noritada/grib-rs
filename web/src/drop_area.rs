use wasm_bindgen::JsCast;
use web_sys::Element;
use yew::prelude::*;

use crate::utils;

const DROP_ZONE_ID: &str = "drop-zone";

#[derive(Properties, PartialEq)]
pub struct FileDropAreaProps {
    pub first_time: bool,
    pub on_drop: Callback<web_sys::File>,
}

#[function_component(FileDropArea)]
pub(crate) fn file_drop_area(
    FileDropAreaProps {
        first_time,
        on_drop,
    }: &FileDropAreaProps,
) -> Html {
    let on_drag_over = {
        Callback::from(move |e: DragEvent| {
            e.prevent_default();

            if let Some(target) = e.target() {
                let element = target.unchecked_into::<Element>();
                element
                    .class_list()
                    .add_1("dragover")
                    .expect("adding class 'dragover' failed")
            }
        })
    };
    let first_time_ = *first_time;
    let on_drag_leave = {
        Callback::from(move |e: DragEvent| {
            e.prevent_default();

            if let Some(target) = e.target() {
                let element = target.unchecked_into::<Element>();
                element
                    .class_list()
                    .remove_1("dragover")
                    .expect("removing class 'dragover' failed")
            }

            if !first_time_ {
                hide_drop_zone();
            }
        })
    };
    let on_file_drop = {
        let on_drop = on_drop.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();

            if let Some(target) = e.target() {
                let element = target.unchecked_into::<Element>();
                element
                    .class_list()
                    .remove_1("dragover")
                    .expect("removing class 'dragover' failed")
            }

            hide_drop_zone();

            let item = e
                .data_transfer()
                .and_then(|transfer| transfer.files())
                .and_then(|files| files.item(0));
            if let Some(item) = item {
                on_drop.emit(item)
            }
        })
    };

    html! {
        <div id={ "drop-zone" } ondragover={on_drag_over} ondragleave={on_drag_leave} ondrop={on_file_drop}>
            <div id="drop-zone-content">
                <h1>{ "GRIB2 Data Viewer" }</h1>
                { "Drag and drop file here" }
                <div class="note">{ "Data is never transferred over the Internet." }</div>
            </div>
        </div>
    }
}

pub(crate) fn display_drop_zone() {
    if let Some(classes) = utils::get_classes(DROP_ZONE_ID) {
        classes
            .remove_1("invisible")
            .expect("removing class 'invisible' failed")
    }
}

pub(crate) fn hide_drop_zone() {
    if let Some(classes) = utils::get_classes(DROP_ZONE_ID) {
        classes
            .add_1("invisible")
            .expect("adding class 'invisible' failed")
    }
}
