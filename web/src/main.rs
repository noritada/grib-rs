use yew::prelude::*;
mod drop_area;
use drop_area::FileDropArea;

#[function_component(App)]
fn app() -> Html {
    let first_time = use_state(|| true);
    let dropped_file = use_state(|| None);

    let first_time_ = first_time.clone();
    let on_file_drop = {
        let dropped_file = dropped_file.clone();
        Callback::from(move |file: web_sys::File| {
            dropped_file.set(Some(file));
            first_time_.set(false);
        })
    };

    let file_name = if let Some(file) = dropped_file.as_ref() {
        file.name()
    } else {
        String::new()
    };

    let on_drag_over = {
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            drop_area::display_drop_zone();
        })
    };

    html! {
        <>
            <div id="main" ondragover={ on_drag_over }>
                <h1>{ "GRIB2 Data Viewer" }</h1>
                <div>{ file_name }</div>
            </div>
            <FileDropArea first_time={*first_time} on_drop={on_file_drop} />
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
