use web_sys::{HtmlInputElement, Url, Element};
use yew::prelude::*;
use gloo_file::File;
use gloo_file::futures::read_as_bytes;
use wasm_bindgen::JsCast;

const BYTES_PER_ROW: usize = 16;
const ROW_HEIGHT: f64 = 29.6; // The measured or estimated height of a single table row in pixels.
const OVERSCAN_ROWS: usize = 10; // Render a few extra rows above and below the viewport.

pub enum Msg {
    LoadFile(File),
    FileLoaded(String, Vec<u8>),
    FileLoadError(String),
    UpdateByte(usize, String),
    SaveFile,
    Scrolled(Event),
}

pub struct HexEditor {
    file_name: String,
    file_data: Vec<u8>,
    error: Option<String>,
    scroll_top: f64,
    container_height: f64,
    scroll_container_ref: NodeRef,
}

impl Component for HexEditor {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            file_name: "no file loaded".to_string(),
            file_data: Vec::new(),
            error: None,
            scroll_top: 0.0,
            container_height: 500.0,
            scroll_container_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadFile(file) => {
                let file_name = file.name();
                _ctx.link().send_future(async move {
                    match read_as_bytes(&file).await {
                        Ok(data) => Msg::FileLoaded(file_name, data),
                        Err(e) => Msg::FileLoadError(e.to_string()),
                    }
                });
                false
            }
            Msg::FileLoaded(name, data) => {
                self.file_name = name;
                self.file_data = data;
                self.error = None;
                self.scroll_top = 0.0;
                true
            }
            Msg::FileLoadError(err_msg) => {
                self.error = Some(format!("Error loading file: {}", err_msg));
                self.file_data.clear();
                self.file_name = "no file loaded".to_string();
                true
            }
            Msg::UpdateByte(index, hex_value) => {
                if let Some(byte) = self.file_data.get_mut(index) {
                    if let Ok(new_byte) = u8::from_str_radix(&hex_value, 16) {
                        *byte = new_byte;
                        return true;
                    }
                }
                false
            }
            Msg::SaveFile => {
                if !self.file_data.is_empty() {
                    let blob = gloo_file::Blob::new_with_options(self.file_data.as_slice(), Some("application/octet-stream"));
                    let url = Url::create_object_url_with_blob(blob.as_ref()).unwrap();
                    let document = web_sys::window().unwrap().document().unwrap();
                    let a: web_sys::HtmlAnchorElement = document.create_element("a").unwrap().dyn_into().unwrap();
                    a.set_href(&url);
                    a.set_download(&self.file_name);
                    a.click();
                    Url::revoke_object_url(&url).unwrap();
                }
                false
            }
            Msg::Scrolled(e) => {
                let target: Element = e.target_unchecked_into();
                // FIX 1: Cast the i32 from scroll_top() into our f64 field.
                self.scroll_top = target.scroll_top() as f64;
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // FIX 2: Cast the NodeRef to an Element to get client_height.
            if let Some(element) = self.scroll_container_ref.cast::<web_sys::Element>() {
                self.container_height = element.client_height() as f64;
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let on_file_change = link.callback(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                if let Some(file) = files.get(0) {
                    return Msg::LoadFile(File::from(file));
                }
            }
            Msg::FileLoadError("No file selected.".to_string())
        });

        let total_rows = (self.file_data.len() as f64 / BYTES_PER_ROW as f64).ceil() as usize;
        let total_height = total_rows as f64 * ROW_HEIGHT;

        let first_visible_row = (self.scroll_top / ROW_HEIGHT).floor() as usize;
        let num_visible_rows = (self.container_height / ROW_HEIGHT).ceil() as usize;

        let start_row = first_visible_row.saturating_sub(OVERSCAN_ROWS / 2);
        let end_row = (first_visible_row + num_visible_rows + OVERSCAN_ROWS / 2).min(total_rows);

        let visible_data_slice = if self.file_data.is_empty() || start_row >= end_row {
            &[]
        } else {
            let start_byte = start_row * BYTES_PER_ROW;
            let end_byte = (end_row * BYTES_PER_ROW).min(self.file_data.len());
            &self.file_data[start_byte..end_byte]
        };

        let on_scroll = link.callback(Msg::Scrolled);

        html! {
            <div class="container mt-4">
                <header class="d-flex justify-content-between align-items-center mb-4 p-3 bg-dark-subtle rounded">
                    <h1>
                        <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="currentColor" class="bi bi-file-earmark-binary-fill me-2" viewBox="0 0 16 16"><path d="M5.526 6.382a.5.5 0 0 0-.526.992l1.102 4.41a.5.5 0 0 0 .992-.248L6.094 6.63a.5.5 0 0 0-.568-.248M7.34 6.227a.5.5 0 0 0-.564.833l1.32 1.319a.5.5 0 0 0 .707-.707l-1.32-1.319a.5.5 0 0 0-.143-.126m2.28-.283a.5.5 0 0 0-.707.707l.883.884a.5.5 0 0 0 .708-.707l-.884-.884Z"/><path d="M4 0h5.293A1 1 0 0 1 10 .293L13.707 4A1 1 0 0 1 14 5.293V14a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V2a2 2 0 0 1 2-2m5.5 1.5v2a1 1 0 0 0 1 1h2zM3.604 10.331a.5.5 0 0 0-.992.138l-1.102 4.41A.5.5 0 0 0 2 15h1.17a.5.5 0 0 0 .495-.368l.53-2.122h2.51l.53 2.122a.5.5 0 0 0 .495.368H8.5a.5.5 0 0 0 .495-.368l-1.102-4.41a.5.5 0 0 0-.992-.138l-.434 1.735H4.04zm1.06 2.122h1.67l-.835-3.34h-.002zM10.151 6.132a.5.5 0 0 0-.89.445l.428 2.146a.5.5 0 0 0 .973-.193zM12 15a.5.5 0 0 0 .5-.5v-4a.5.5 0 0 0-1 0v4a.5.5 0 0 0 .5.5"/></svg>
                        { "Hex Editor" }
                    </h1>
                    <div>
                        <span class="badge bg-secondary me-2">{ format!("{} bytes", self.file_data.len()) }</span>
                        <span class="badge bg-info">{ &self.file_name }</span>
                    </div>
                </header>

                if let Some(err) = &self.error {
                    <div class="alert alert-danger">{ err }</div>
                }

                <div class="mb-3 d-flex gap-2">
                    <input class="form-control" type="file" onchange={on_file_change} />
                    <button class="btn btn-primary" onclick={link.callback(|_| Msg::SaveFile)} disabled={self.file_data.is_empty()}>
                        { "Save" }
                    </button>
                </div>

                // FIX 3: Use explicit `onscroll={...}` instead of shorthand `{...}`.
                <div ref={self.scroll_container_ref.clone()} class="scroll-container" onscroll={on_scroll}>
                    <div class="scroll-sizer" style={format!("height: {}px;", total_height)}>
                        <table class="table table-dark table-sm table-hover mb-0 virtual-table" style={format!("transform: translateY({}px);", start_row as f64 * ROW_HEIGHT)}>
                            <thead>
                                <tr>
                                    <th class="text-secondary">{ "Offset" }</th>
                                    { for (0..BYTES_PER_ROW).map(|i| html!{ <th class="text-secondary text-center">{ format!("{:02X}", i) }</th> }) }
                                    <th class="text-secondary">{ "ASCII" }</th>
                                </tr>
                            </thead>
                            <tbody>
                                { for visible_data_slice.chunks(BYTES_PER_ROW).enumerate().map(|(i, bytes)| self.view_row(ctx, start_row + i, bytes)) }
                            </tbody>
                        </table>
                    </div>
                </div>

                <footer class="text-center text-muted mt-4">
                    <p>{ "Built by " }<a href="https://www.andydixon.com" target="_blank">{"Andy Dixon"}</a> { " with Rust." }</p>
                </footer>
            </div>
        }
    }
}

impl HexEditor {

fn view_row(&self, ctx: &Context<Self>, row_idx: usize, bytes: &[u8]) -> Html {
    let offset = row_idx * BYTES_PER_ROW;
    let link = ctx.link();

    html! {
        <tr>
            <td class="text-secondary">{ format!("{:08X}", offset) }</td>
            { for bytes.iter().enumerate().map(|(i, byte)| {
                let byte_idx = offset + i;
                // The callback now expects a generic `Event` from `onchange`.
                let on_hex_change = link.callback(move |e: Event| { // <-- 1. Type changed to Event
                    let input: HtmlInputElement = e.target_unchecked_into();
                    Msg::UpdateByte(byte_idx, input.value())
                });
                html!{
                    <td class="text-center">
                        <input
                            type="text"
                            class="hex-input"
                            value={format!("{:02X}", byte)}
                            onchange={on_hex_change} // <-- 2. Event changed to onchange
                            maxlength="2"
                        />
                    </td>
                }
            }) }
            { for (0..(BYTES_PER_ROW - bytes.len())).map(|_| html!{ <td></td> }) }
            <td class="ascii-char">{
                bytes.iter().map(|&b| if (32..=126).contains(&b) { b as char } else { '.' }).collect::<String>()
            }</td>
        </tr>
    }
}

}

fn main() {
    yew::Renderer::<HexEditor>::new().render();
}
