use leptos::html; // import leptos HTML module
use leptos::prelude::*; // import leptos names
use pulldown_cmark::{CowStr, Event, Options, Parser, Tag}; // CowStr = Copy on write string
fn main() {
    leptos::mount::mount_to_body(App);
}

#[component] // function turns into component
fn App() -> impl IntoView {
    // impl = implement
    view! { // view! turns HTML-esque code into rust at compile time, baby
        <main class="board">
            <Note text="What the fuck did you just fucking say about me, you little bitch?"/> // humorous placeholder text
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

#[component]
fn Note(text: &'static str) -> impl IntoView {
    // &'static str = borrowed string literal, in program file
    let (content, set_content) = signal(text.to_string()); // let = creates a variable, signal (leptos container) tuple (getter, setter)
    let (editing, set_editing) = signal(false); // bool start false (the sims 1 cheat was my first bool, good times)
    let textarea_ref: NodeRef<html::Textarea> = NodeRef::new(); // NodeRef = slot for page element

    Effect::new(move || {
        if let Some(el) = textarea_ref.get() {
            // el = element
            let _ = el.focus();
        }
    });

    view! {
        <div class="note" on:click=move |_| set_editing.set(true)>
            <Show
                when=move || editing.get() // when when is true
                fallback=move || view! {
                    <div class="rendered" inner_html=move || render_markdown(&content.get())></div>
                } // fallback = when when is false
            >
                <textarea
                    node_ref=textarea_ref
                    prop:value=move || content.get()
                    on:input=move |ev| set_content.set(event_target_value(&ev)) // ev = event
                    on:blur=move |_| set_editing.set(false)
                />
            </Show>
        </div>
    }
}
