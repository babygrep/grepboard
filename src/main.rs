use leptos::ev; // import leptos event types
use leptos::html; // import leptos HTML module
use leptos::prelude::*; // import leptos names
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag}; // CowStr = Copy on write string
fn main() {
    leptos::mount::mount_to_body(App);
}

const CELL: i32 = 16; // constant cell type is i32, let CELL ∈ ℤ, CELL = 16
const MIN_W: i32 = 3; // minimum note width
const MIN_H: i32 = 2; // minimum note height

#[derive(Clone, Copy)] // clone and copy are an iconic duo in Rust, must go together
struct NoteData {
    id: u32,
    x: RwSignal<i32>, // RwSignal = read-write signal, both getter and setter in one!
    y: RwSignal<i32>,
    w: RwSignal<i32>,
    h: RwSignal<i32>,
    text: RwSignal<String>, // text type is read-write signal holding string of changeable text
}

impl NoteData {
    // implements functions that belong to NoteData
    fn new(id: u32, x: i32, y: i32, w: i32, h: i32, text: &str) -> Self {
        NoteData {
            id,
            x: RwSignal::new(x),
            y: RwSignal::new(y),
            w: RwSignal::new(w),
            h: RwSignal::new(h),
            text: RwSignal::new(text.to_string()), // turns passed in text into an owned copy and signal that's stored as the notes contents
        }
    }
}

#[component] // function turns into component
fn App() -> impl IntoView {
    let notes = RwSignal::new(vec![
        NoteData::new(1, 2, 2, 14, 8, "# hello\nfirst note"),
        NoteData::new(2, 18, 4, 12, 6, "- [ ] second"),
    ]);
    let draft = RwSignal::new(None::<(i32, i32, i32, i32)>);

    let start_draft = move |ev: ev::PointerEvent| {
        // note dragging event
        let col = cell_under(ev.client_x());
        let row = cell_under(ev.client_y());
        draft.set(Some((col, row, col, row)));
    };

    let grow_draft = move |ev: ev::PointerEvent| {
        if let Some((col, row, _, _)) = draft.get() {
            draft.set(Some((
                col,
                row,
                cell_under(ev.client_x()),
                cell_under(ev.client_y()),
            )));
        }
    };

    let finish_draft = move |_| {
        if let Some((a_col, a_row, b_col, b_row)) = draft.get() {
            let w = (a_col - b_col).abs() + 1;
            let h = (a_row - b_row).abs() + 1;
            if w >= MIN_W && h >= MIN_H {
                let id = notes.with(|list| list.iter().map(|n| n.id).max().unwrap_or(0) + 1);
                let note = NoteData::new(id, a_col.min(b_col), a_row.min(b_row), w, h, "");
                notes.update(|list| list.push(note));
            }
        }
        draft.set(None); // clears draft when drag is over
    };

    let draft_style = move || match draft.get() {
        Some((a_col, a_row, b_col, b_row)) => format!(
            "left: {}px; top: {}px; width: {}px; height: {}px;",
            a_col.min(b_col) * CELL,
            a_row.min(b_row) * CELL,
            ((a_col - b_col).abs() + 1) * CELL, // abs() = absolute value
            ((a_row - b_row).abs() + 1) * CELL,
        ),
        None => "display: none;".to_string(),
    };

    view! {
        <main
            class="board"
            style=format!("--cell: {CELL}px")
            on:pointerdown=start_draft
            on:pointermove=grow_draft
            on:pointerup=finish_draft
        >
            <div class="draft" style=draft_style></div>
            <For each=move || notes.get() key=|note| note.id let(note)>
                <Note note=note notes=notes />
            </For>
        </main>
    }
}

// url safety check for markdown
fn is_safe_url(url: &str) -> bool {
    // bool answers if safe (yes/no)
    let cleaned: String = url // chain check
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control()) // ! = not, && = and, so it keeps c if it's not whitespace and not invisible code char
        .collect();

    match cleaned.split_once(':') {
        // splits at the first ':'
        None => true, // no ':' means no scheme = safe
        Some((scheme, _)) if scheme.contains(['/', '?', '#']) => true, // no scheme, so safe
        Some((scheme, _)) => matches!(
            // yes scheme
            scheme.to_ascii_lowercase().as_str(), // turns Javascript into javascript
            "http" | "https" | "mailto"           // allow-list
        ),
    }
}

// rendering markdown, not a component so it don't need the little #[chud]
fn render_markdown(source: &str) -> String {
    let mut options = Options::empty(); // mut = mutable
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_TABLES);

    let events = Parser::new_ext(source, options).map(|event| match event {
        // parser with extra options (new_ext) reads markdown
        Event::Html(raw) | Event::InlineHtml(raw) => Event::Text(raw),
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) if !is_safe_url(&dest_url) => Event::Start(Tag::Link {
            link_type,
            dest_url: CowStr::Borrowed("#"),
            title,
            id,
        }),
        Event::Start(Tag::Image {
            link_type,
            dest_url,
            title,
            id,
        }) if !is_safe_url(&dest_url) => Event::Start(Tag::Image {
            link_type,
            dest_url: CowStr::Borrowed("#"),
            title,
            id,
        }),
        other => other,
    });

    let mut output = String::new();
    pulldown_cmark::html::push_html(&mut output, events); // push_html writes HTML for each item
    output
}

fn snap(px: i32) -> i32 {
    (px as f64 / CELL as f64).round() as i32
} // snap grid by turning distance (in pixels) into nearest whole number

fn cell_under(px: i32) -> i32 {
    px.div_euclid(CELL)
}

#[derive(Clone, Copy)]
enum Gesture {
    Move {
        start_x: i32,
        start_y: i32,
        from_x: i32,
        from_y: i32,
    },
    Resize {
        start_x: i32,
        start_y: i32,
        from_w: i32,
        from_h: i32,
    },
}

#[component]
fn Note(note: NoteData, notes: RwSignal<Vec<NoteData>>) -> impl IntoView {
    let NoteData {
        id,
        x,
        y,
        w,
        h,
        text,
    } = note;
    let (editing, set_editing) = signal(false); // bool start false (the sims 1 cheat was my first bool, good times)
    let gesture = RwSignal::new(None::<Gesture>);
    let moved = RwSignal::new(false);
    let note_ref: NodeRef<html::Div> = NodeRef::new();
    let textarea_ref: NodeRef<html::Textarea> = NodeRef::new(); // NodeRef = slot for page element

    let style = move || {
        // ! after a word, like 'format!' means it's a macro which runs before program compiles, function runs while program runs
        format!(
            "left: {}px; top: {}px; width: {}px; height: {}px",
            x.get() * CELL,
            y.get() * CELL,
            w.get() * CELL,
            h.get() * CELL,
        )
    };

    Effect::new(move || {
        if let Some(el) = textarea_ref.get() {
            // el = element
            let _ = el.focus();
        }
    });

    let begin = move |ev: &ev::PointerEvent, kind: Gesture| {
        if let Some(el) = note_ref.get() {
            let _ = el.set_pointer_capture(ev.pointer_id());
        }
        gesture.set(Some(kind));
        moved.set(false);
    };

    let start_move = move |ev: ev::PointerEvent| {
        ev.stop_propagation();
        if editing.get() {
            return;
        }
        let kind = Gesture::Move {
            start_x: ev.client_x(),
            start_y: ev.client_y(),
            from_x: x.get(),
            from_y: y.get(),
        };
        begin(&ev, kind);
    };

    let start_resize = move |ev: ev::PointerEvent| {
        ev.stop_propagation();
        let kind = Gesture::Resize {
            start_x: ev.client_x(),
            start_y: ev.client_y(),
            from_w: w.get(),
            from_h: h.get(),
        };
        begin(&ev, kind);
    };

    let during_gesture = move |ev: ev::PointerEvent| {
        let px = ev.client_x();
        let py = ev.client_y();
        match gesture.get() {
            Some(Gesture::Move {
                start_x,
                start_y,
                from_x,
                from_y,
            }) => {
                let new_x = (from_x + snap(px - start_x)).max(0);
                let new_y = (from_y + snap(py - start_y)).max(0);
                if new_x != x.get() || new_y != y.get() {
                    x.set(new_x);
                    y.set(new_y);
                    moved.set(true);
                }
            }
            Some(Gesture::Resize {
                start_x,
                start_y,
                from_w,
                from_h,
            }) => {
                let new_w = (from_w + snap(px - start_x)).max(MIN_W);
                let new_h = (from_h + snap(py - start_y)).max(MIN_H);
                if new_w != w.get() || new_h != h.get() {
                    w.set(new_w);
                    h.set(new_h);
                    moved.set(true);
                }
            }
            None => {}
        }
    };

    let end_gesture = move |ev: ev::PointerEvent| {
        ev.stop_propagation();
        gesture.set(None);
    };

    let open_editor = move |_: ev::MouseEvent| {
        if moved.get() {
            moved.set(false);
        } else {
            set_editing.set(true);
        }
    };

    let delete = move |ev: ev::MouseEvent| {
        ev.stop_propagation();
        notes.update(|list| list.retain(|n| n.id != id));
    };

    view! {
        <div
            class="note"
            node_ref=note_ref
            style=style
            on:pointerdown=start_move
            on:pointermove=during_gesture
            on:pointerup=end_gesture
            on:click=open_editor
        >
            <Show
                when=move || editing.get() // when when is true
                fallback=move || view! {
                    <div class="rendered" inner_html=move || render_markdown(&text.get())></div>
                } // fallback = when when is false
            >
                <textarea
                    node_ref=textarea_ref
                    prop:value=move || text.get()
                    on:input=move |ev| text.set(event_target_value(&ev)) // ev = event
                    on:blur=move |_| set_editing.set(false)
                />
            </Show>
            <button
                class="delete"
                title="Delete note"
                on:pointerdown=|ev| ev.stop_propagation()
                on:click=delete
            >
                "x"
            </button>
            <div class="resize" on:pointerdown=start_resize></div>
        </div>
    }
}
