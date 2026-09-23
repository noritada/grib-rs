use std::{borrow::Cow, ops::Deref};

use gloo_file::{Blob, futures::read_as_bytes};
use grib::{
    GridShortName,
    codetables::{CodeTable4_2, CodeTable4_3, Lookup},
};
use web_sys::ImageData;
use yew::prelude::*;
mod drop_area;
use drop_area::FileDropArea;
mod submessage_modal;
use submessage_modal::SubmessageModal;
mod palette;
mod utils;

#[function_component(App)]
fn app() -> Html {
    let first_time = use_state(|| true);
    let dropped_file = use_state(|| None);
    let grib_context = use_state(|| None);
    let image_data = use_state(|| None);
    let rendering_error = use_state(|| None::<String>);

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

    let on_click_submessage_modal = {
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            submessage_modal::hide_submessage_modal();
        })
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
                        grib_context.set(grib.ok());
                        submessage_modal::hide_submessage_modal();
                    }
                });
            }
        });
    }

    let listing = if let Some(context) = grib_context.as_ref() {
        let submessages = context.submessages();
        let (len, _) = submessages.size_hint();
        let submessages_html = submessages
            .enumerate()
            .map(|(index, (i, submessage))| {
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
                    .map(|(first, second)| (format_surface(&first), format_surface(&second)))
                    .unwrap_or((String::new(), String::new()));
                let grid_def = submessage.grid_def();
                let num_grid_points = grid_def.num_points();
                let num_points_represented = submessage.repr_def().num_points();
                let num_missing = num_grid_points
                    .checked_sub(num_points_represented)
                    .map(|count| count.to_string())
                    .unwrap_or_else(|| "invalid".to_owned());
                let grid_type = grib::GridDefinitionTemplateValues::try_from(grid_def)
                    .map(|def| Cow::from(def.short_name()))
                    .unwrap_or_else(|_| {
                        Cow::from(format!("unknown (template {})", grid_def.grid_tmpl_num()))
                    });

                let grib_context_ = grib_context.clone();
                let image_data_ = image_data.clone();
                let rendering_error_ = rendering_error.clone();
                let on_click_submessage_row = {
                    Callback::from(move |e: MouseEvent| {
                        e.prevent_default();
                        if let Some(grib) = grib_context_.as_ref() {
                            let result = grib
                                .submessages()
                                .nth(index)
                                .ok_or_else(|| "it is no longer available".to_owned())
                                .and_then(|(_i, submessage)| {
                                    let (w, h) = submessage
                                        .grid_shape()
                                        .map_err(|error| error.to_string())?;
                                    let decoder = grib::Grib2SubmessageDecoder::from(submessage)
                                        .map_err(|error| error.to_string())?;
                                    let values =
                                        decoder.dispatch().map_err(|error| error.to_string())?;
                                    let pixel_bytes = values
                                        .flat_map(palette::jma_amedas_temperature)
                                        .collect::<Vec<_>>();
                                    ImageData::new_with_u8_clamped_array_and_sh(
                                        wasm_bindgen::Clamped(pixel_bytes.as_slice()),
                                        w as u32,
                                        h as u32,
                                    )
                                    .map_err(|error| {
                                        format!(
                                            "the browser could not create image data: {error:?}"
                                        )
                                    })
                                });
                            match result {
                                Ok(image_data) => {
                                    image_data_.set(Some(image_data));
                                    rendering_error_.set(None);
                                }
                                Err(error) => {
                                    image_data_.set(None);
                                    rendering_error_.set(Some(format!(
                                        "Unable to render this submessage: {error}"
                                    )));
                                }
                            };
                        }
                        submessage_modal::display_submessage_modal();
                    })
                };

                html! {
                    <tr onclick={on_click_submessage_row}>
                        <td>{id}</td>
                        <td>{category}</td>
                        <td>{generating_process}</td>
                        <td>{forecast_time}</td>
                        <td>{surfaces.0}</td>
                        <td>{surfaces.1}</td>
                        <td>{num_missing}</td>
                        <td>{num_grid_points}</td>
                        <td>{grid_type}</td>
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
                                <th>{"grid type"}</th>
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

    let image_data = image_data.as_ref().map(|i| i.clone());
    let rendering_error = rendering_error.as_ref().map(String::clone);

    html! {
        <>
            <div id="main" ondragover={ on_drag_over.clone() }>
                <h1>{ "GRIB2 Data Viewer" }</h1>
                <div>{ file_name }</div>
                { listing }
            </div>
            <SubmessageModal
                image_data={image_data}
                rendering_error={rendering_error}
                on_click={on_click_submessage_modal}
                on_drag_over={on_drag_over}
            />
            <FileDropArea first_time={*first_time} on_drop={on_file_drop} />
        </>
    }
}

fn format_surface(surface: &grib::FixedSurface) -> String {
    let value = surface.value();
    let unit = surface
        .unit()
        .map(|s| format!(" [{s}]"))
        .unwrap_or_default();
    format!("{value}{unit}")
}

fn main() {
    yew::Renderer::<App>::new().render();
}
