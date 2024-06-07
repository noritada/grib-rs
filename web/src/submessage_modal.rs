use yew::prelude::*;

const SUBMESSAGE_MODAL_ID: &str = "submessage-modal";

#[derive(Properties, PartialEq)]
pub struct SubmessageModalProps {
    pub on_click: Callback<MouseEvent>,
}

#[function_component(SubmessageModal)]
pub(crate) fn submessage_modal(SubmessageModalProps { on_click }: &SubmessageModalProps) -> Html {
    html! {
        <div id={SUBMESSAGE_MODAL_ID} class="invisible" onclick={on_click}>
            <div id="submessage-details"></div>
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
