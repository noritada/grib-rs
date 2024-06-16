pub(crate) fn get_classes(element_id: &str) -> Option<web_sys::DomTokenList> {
    let window = web_sys::window()?;
    let document = window.document()?;
    let element = document.get_element_by_id(element_id)?;
    let class_list = element.class_list();
    Some(class_list)
}
