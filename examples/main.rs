mod joke;

use std::collections::HashSet;
use leptos::prelude::*;

// Gemini wrote this part initially.
#[component]
pub fn EnterInput(set_endpoint: WriteSignal<String>) -> impl IntoView {
    // Create a signal to store the current input value
    let (input_text, set_input_text) = signal("".to_string());

    // Define the action to be performed on Enter
    let handle_enter_action = move |_| {
        // This closure needs to capture 'input_text' and 'set_submitted_text'
        // to read the current input and update the submitted text.
        let current_input = input_text.get(); // Get the current value from the signal
        if !current_input.trim().is_empty() {
            set_endpoint.set(format!("joke/{}", current_input));
        }
    };

    view! {
        <div>
            "Find a joke: " <input
                type="text"
                // Bind the input's value to the signal
                prop:value=input_text
                // Update the signal when the input changes
                on:input=move |ev| {
                    set_input_text.set(event_target_value(&ev));
                }
                // Listen for keydown events
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    if ev.key() == "Enter" {
                        handle_enter_action(ev);
                    }
                }
                placeholder="Joke ID"
            />
        </div>
    }
}

fn format_tags(tags: &HashSet<String>) -> String {
    let taglist: Vec<&str> = tags.iter().map(String::as_ref).collect();
    taglist.join(", ")
}

fn fetch_joke() -> impl IntoView {
    let (endpoint, set_endpoint) = signal::<String>("random-joke".to_string());
    let joke = LocalResource::new(move || joke::fetch(endpoint.get()));

    let error_fallback = move |errors: ArcRwSignal<Errors>| {
        let error_list = move || {
            errors.with(|errors| {
                errors
                    .iter()
                    .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                    .collect::<Vec<_>>()
            })
        };

        view! {
            <div>
                <h2>"Error"</h2>
                <span class="error">{error_list}</span>
            </div>
        }
    };

    view! {
        <div><Transition fallback=|| view! { <div>"Loading..."</div> }>
            <ErrorBoundary fallback=error_fallback>
                {move || Suspend::new( async move {
                    joke.map(|j| {
                        // XXX Don't know how to fix this unwrap() yet.
                        let j = j.as_ref().unwrap();
                        view! {
                            <div class="joke">
                                <span class="teller">{"Knock-Knock!"}</span><br/>
                                <span class="tellee">{"Who's there?"}</span><br/>
                                <span class="teller">{j.whos_there.clone()}</span><br/>
                                <span class="tellee">{format!("{} who?", j.whos_there)}</span><br/>
                                <span class="teller">{j.answer_who.clone()}</span>
                            </div>
                            <span class="annotation">
                                {format!(
                                    "[id: {}; tags: {}; source: {}]",
                                    j.id,
                                    format_tags(&j.tags),
                                    j.source,
                                )}
                            </span>
                        }
                    })
                })}
            </ErrorBoundary>
        </Transition></div>
        <div>
            <button on:click=move |_| {
                let ep = "random-joke".to_string();
                set_endpoint.set(ep)
            }>Tell me another!</button>
            <EnterInput set_endpoint=set_endpoint/>
        </div>
    }
}

pub fn main() {
    use tracing_subscriber::fmt;
    use tracing_subscriber_wasm::MakeConsoleWriter;

    fmt()
        .with_writer(
            // To avoid trace events in the browser from showing their
            // JS backtrace, which is very annoying, in my opinion
            MakeConsoleWriter::default()
                .map_trace_level_to(tracing::Level::DEBUG),
        )
        // For some reason, if we don't do this in the browser, we get
        // a runtime error.
        .without_time()
        .init();
    console_error_panic_hook::set_once();
    mount_to_body(fetch_joke)
}
