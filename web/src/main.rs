use std::ops::Deref;

use gloo_file::{futures::read_as_bytes, Blob};
use grib::codetables::{CodeTable4_2, CodeTable4_3, Lookup};
use yew::prelude::*;
mod drop_area;
use drop_area::FileDropArea;

#[function_component(App)]
fn app() -> Html {
    let first_time = use_state(|| true);
    let dropped_file = use_state(|| None);
    let grib_context = use_state(|| None);

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

    {
        let grib_context = grib_context.clone();
        let file = dropped_file.clone();
        use_effect_with(dropped_file, move |_| {
            if let Some(file) = file.as_ref() {
                let blob = Blob::from(file.deref().clone());
                wasm_bindgen_futures::spawn_local(async move {
                    let result = read_as_bytes(&blob).await;
                    if let Ok(bytes_) = result {
                        let grib = grib::from_reader(std::io::Cursor::new(bytes_));
                        grib_context.set(grib.ok())
                    }
                });
            }
        });
    }

    let listing = if let Some(context) = grib_context.as_ref() {
        let submessages = context.submessages();
        let (len, _) = submessages.size_hint();
        let submessages_html = submessages
            .map(|(i, submessage)| {
                let id = format!("{}.{}", i.0, i.1);
                let prod_def = submessage.prod_def();
                let category = prod_def
                    .parameter_category()
                    .zip(prod_def.parameter_number())
                    .map(|(c, n)| {
                        CodeTable4_2::new(submessage.indicator().discipline, c)
                            .lookup(usize::from(n))
                            .to_string()
                    })
                    .unwrap_or_default();
                let generating_process = prod_def
                    .generating_process()
                    .map(|v| CodeTable4_3.lookup(usize::from(v)).to_string())
                    .unwrap_or_default();
                let forecast_time = prod_def
                    .forecast_time()
                    .map(|ft| ft.to_string())
                    .unwrap_or_default();
                let surfaces = prod_def
                    .fixed_surfaces()
                    .map(|(first, second)| (first.value().to_string(), second.value().to_string()))
                    .unwrap_or((String::new(), String::new()));
                let num_grid_points = submessage.grid_def().num_points();
                let num_points_represented = submessage.repr_def().num_points();
                html! {
                    <tr>
                        <td>{id}</td>
                        <td>{category}</td>
                        <td>{generating_process}</td>
                        <td>{forecast_time}</td>
                        <td>{surfaces.0}</td>
                        <td>{surfaces.1}</td>
                        <td>{num_grid_points - num_points_represented}</td>
                        <td>{num_grid_points}</td>
                    </tr>
                }
            })
            .collect::<Html>();
        html! {
            <>
                <div>{format!("{} submessage(s)", len)}</div>
                <div id="submessage_list">
                    <table>
                        <thead>
                            <tr>
                                <th>{"#"}</th>
                                <th>{"parameter"}</th>
                                <th>{"generating process"}</th>
                                <th>{"forecast time"}</th>
                                <th>{"1st fixed surface"}</th>
                                <th>{"2nd fixed surface"}</th>
                                <th>{"#points (nan)"}</th>
                                <th>{"#points (total)"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {submessages_html}
                        </tbody>
                    </table>
                </div>
            </>
        }
    } else {
        html! {}
    };

    html! {
        <>
            <div id="main" ondragover={ on_drag_over }>
                <h1>{ "GRIB2 Data Viewer" }</h1>
                <div>{ file_name }</div>
                { listing }
            </div>
            <FileDropArea first_time={*first_time} on_drop={on_file_drop} />
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
