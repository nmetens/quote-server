mod quote;

use leptos::prelude::*;

pub fn main() {
    use tracing_subscriber::fmt;
    use tracing_subscriber_wasm::MakeConsoleWriter;

    fmt()
        .with_writer(MakeConsoleWriter::default().map_trace_level_to(tracing::Level::DEBUG))
        .without_time()
        .init();

    console_error_panic_hook::set_once();

    mount_to_body(fetch_quote);
}

fn fetch_quote() -> impl IntoView {
    // Signal to store the endpoint string
    let (endpoint, set_endpoint) = signal::<String>("random-quote".to_string());
    // Signal to store the input for theme
    let (theme_input, set_theme_input) = signal("".to_string());
    // Create a LocalResource to fetch the quote
    let quote = LocalResource::new(move || quote::fetch(endpoint.get()));

    // Error fallback
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
        <div class="container">
            <h1>"Quote"</h1>

            <Transition fallback=|| view! { <div>"Loading..."</div> }>
                <ErrorBoundary fallback=error_fallback>
                    {move || Suspend::new(async move {
                        quote.map(|q| {
                            let q = q.as_ref().unwrap();
                            view! {
                                <>
                                    <div class="quote">
                                        <span>{q.quote.clone()}</span><br/>
                                    </div>
                                    <div class="info">
                                        <span class="source">{"Author: "}{q.author.clone()}</span><br/>
                                    </div>
                                </>
                            }
                        })
                    })}
                </ErrorBoundary>
            </Transition>

            <form on:submit=move |ev| {
                ev.prevent_default(); // prevent page reload
                let theme = theme_input.get();
                if theme.trim().is_empty() {
                    set_endpoint.set("random-quote".to_string());
                } else if theme.chars().all(|c| c.is_ascii_digit()) {
                        // It's a number → quote by ID
                        set_endpoint.set(format!("quote/{}", theme));
                } else {
                    // Not a number → treat as tag(s)
                    let tags = theme
                        .split(',')
                        .map(|s| s.trim())
                        .collect::<Vec<_>>()
                        .join(",");
                    set_endpoint.set(format!("tagged-quote?tags={}", tags));
                }
            }>
                <label>"Select a theme, quote id, or leave blank for a random theme:"</label><br/>
                <input
                    type="text"
                    prop:value=theme_input
                    on:input=move |ev| {
                        set_theme_input.set(event_target_value(&ev));
                    }
                /><br/>
                <button type="submit">"New Quote"</button>
            </form>
        </div>
    }
}
