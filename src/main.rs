use web_sys::{HtmlInputElement, Url, Element, HtmlSelectElement};
use yew::prelude::*;
use gloo_file::File;
use gloo_file::futures::read_as_bytes;
use wasm_bindgen::JsCast;

const BYTES_PER_ROW: usize = 16;
const ROW_HEIGHT: f64 = 29.6;
const OVERSCAN_ROWS: usize = 10;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum SearchMode {
    Hex,
    Ascii,
}

pub enum Msg {
    LoadFile(File),
    FileLoaded(String, Vec<u8>),
    FileLoadError(String),
    UpdateByte(usize, String),
    SaveFile,
    Scrolled(Event),
    UpdateSearchTerm(String),
    UpdateSearchMode(SearchMode),
    ExecuteSearch,
    FindNext,
    FindPrevious,
}

pub struct HexEditor {
    file_name: String,
    file_data: Vec<u8>,
    error: Option<String>,
    scroll_top: f64,
    container_height: f64,
    scroll_container_ref: NodeRef,
    search_term: String,
    search_mode: SearchMode,
    search_bytes: Vec<u8>,
    search_results: Vec<usize>,
    current_match_index: Option<usize>,
    search_status: String,
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
            search_term: String::new(),
            search_mode: SearchMode::Ascii,
            search_bytes: Vec::new(),
            search_results: Vec::new(),
            current_match_index: None,
            search_status: String::new(),
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
                self.search_results.clear();
                self.current_match_index = None;
                self.search_status.clear();
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
                self.scroll_top = target.scroll_top() as f64;
                true
            }
            Msg::UpdateSearchTerm(term) => {
                self.search_term = term;
                true
            }
            Msg::UpdateSearchMode(mode) => {
                self.search_mode = mode;
                true
            }
            Msg::ExecuteSearch => {
                self.search_results.clear();
                self.current_match_index = None;
                let search_bytes = match self.search_mode {
                    SearchMode::Ascii => self.search_term.as_bytes().to_vec(),
                    SearchMode::Hex => {
                        let cleaned: String = self.search_term.chars().filter(|c| !c.is_whitespace()).collect();
                        match hex::decode(cleaned) {
                            Ok(bytes) => bytes,
                            Err(_) => {
                                self.search_status = "Invalid Hex sequence.".to_string();
                                return true;
                            }
                        }
                    }
                };
                if search_bytes.is_empty() {
                    self.search_status = "".to_string();
                    return true;
                }
                self.search_bytes = search_bytes.clone();
                self.search_results = self.file_data
                    .windows(search_bytes.len())
                    .enumerate()
                    .filter_map(|(i, window)| if window == search_bytes.as_slice() { Some(i) } else { None })
                    .collect();
                if self.search_results.is_empty() {
                    self.search_status = "Not found.".to_string();
                } else {
                    let count = self.search_results.len();
                    self.search_status = format!("Found {} match(es).", count);
                    self.jump_to_match(0);
                }
                true
            }
            Msg::FindNext => {
                if !self.search_results.is_empty() {
                    let next_index = self.current_match_index.map_or(0, |i| (i + 1) % self.search_results.len());
                    self.jump_to_match(next_index);
                }
                true
            }
            Msg::FindPrevious => {
                if !self.search_results.is_empty() {
                    let total = self.search_results.len();
                    let prev_index = self.current_match_index.map_or(total - 1, |i| (i + total - 1) % total);
                    self.jump_to_match(prev_index);
                }
                true
            }
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
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
        let on_search_input = link.callback(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            Msg::UpdateSearchTerm(input.value())
        });
        
        let on_search_mode_change = link.callback(|e: Event| {
            let select: HtmlSelectElement = e.target_unchecked_into();
            match select.value().as_str() {
                "hex" => Msg::UpdateSearchMode(SearchMode::Hex),
                _ => Msg::UpdateSearchMode(SearchMode::Ascii),
            }
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
                <div class="card bg-dark-subtle mb-3">
                    <div class="card-body">
                        <div class="d-flex gap-2 align-items-center">
                            <select class="form-select" style="width: 100px;" onchange={on_search_mode_change}>
                                <option value="ascii" selected={self.search_mode == SearchMode::Ascii}>{"ASCII"}</option>
                                <option value="hex" selected={self.search_mode == SearchMode::Hex}>{"Hex"}</option>
                            </select>
                            <input type="text" class="form-control" placeholder="Enter search term..." value={self.search_term.clone()} oninput={on_search_input} />
                            <button class="btn btn-secondary" onclick={link.callback(|_| Msg::ExecuteSearch)}>{"Search"}</button>
                            <div class="btn-group">
                                <button class="btn btn-outline-secondary" onclick={link.callback(|_| Msg::FindPrevious)} disabled={self.search_results.is_empty()}>{"<"}</button>
                                <button class="btn btn-outline-secondary" onclick={link.callback(|_| Msg::FindNext)} disabled={self.search_results.is_empty()}>{">"}</button>
                            </div>
                        </div>
                        <div class="form-text mt-1">{ &self.search_status }</div>
                    </div>
                </div>
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

// --- HELPER METHODS IMPL BLOCK ---
// All helper methods are now correctly placed here.
impl HexEditor {
    fn jump_to_match(&mut self, result_index: usize) {
        if let Some(&match_start_byte) = self.search_results.get(result_index) {
            self.current_match_index = Some(result_index);
            let target_row = (match_start_byte / BYTES_PER_ROW) as f64;
            let scroll_pos = target_row * ROW_HEIGHT - (self.container_height / 2.0);
            
            // This logic was missing from the previous attempt.
            // It scrolls the view to the new position.
            if let Some(element) = self.scroll_container_ref.cast::<web_sys::Element>() {
                element.set_scroll_top(scroll_pos.max(0.0) as i32);
            }
        }
    }

    fn view_row(&self, ctx: &Context<Self>, row_idx: usize, bytes: &[u8]) -> Html {
        let offset = row_idx * BYTES_PER_ROW;
        let link = ctx.link();
        let current_match_range = self.current_match_index.and_then(|idx| {
            self.search_results.get(idx).map(|&start| start..(start + self.search_bytes.len()))
        });
        html! {
            <tr>
                <td class="text-secondary">{ format!("{:08X}", offset) }</td>
                { for bytes.iter().enumerate().map(|(i, byte)| {
                    let byte_idx = offset + i;
                    let on_hex_change = link.callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Msg::UpdateByte(byte_idx, input.value())
                    });
                    let is_highlighted = current_match_range.as_ref().map_or(false, |range| range.contains(&byte_idx));
                    let mut class = "hex-input".to_string();
                    if is_highlighted {
                        class.push_str(" highlight");
                    }
                    html!{
                        <td class="text-center">
                            <input type="text" {class} value={format!("{:02X}", byte)} onchange={on_hex_change} maxlength="2" />
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