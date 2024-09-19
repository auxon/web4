use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use warp::Filter;
use warp::http::header::{HeaderMap, HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_HEADERS};
use crate::engine::BrowserEngine;
use tao::{
    event::{Event, StartCause, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    keyboard::{KeyCode, ModifiersState},
    window::{WindowBuilder, Window},
    dpi::LogicalSize,
};
use wry::{WebView, WebViewBuilder, WebContext};

mod engine;
mod llm;
mod network;

#[derive(Debug)]
enum UserEvent {
    LoadUrl(String),
    CreateNewTab,
    SwitchTab(usize),
}

struct Tab {
    webview: WebView,
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Arc::new(BrowserEngine::new());

    let engine_filter = warp::any().map(move || Arc::clone(&engine));

    let cors = warp::reply::with::headers(HeaderMap::from_iter(vec![
        (ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*")),
        (ACCESS_CONTROL_ALLOW_METHODS, HeaderValue::from_static("GET, POST, OPTIONS")),
        (ACCESS_CONTROL_ALLOW_HEADERS, HeaderValue::from_static("Content-Type")),
    ]));

    // Handle GET /load?url=...
    let load_url_get = warp::get()
        .and(warp::path("load"))
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .and(engine_filter.clone())
        .and_then(handle_load_url_get)
        .with(cors.clone());

    // Handle GET /
    let index = warp::get()
        .and(warp::path::end())
        .map(|| warp::reply::html(include_str!("index.html")))
        .with(cors.clone());

    let routes = load_url_get.or(index);

    let server_ready = Arc::new(AtomicBool::new(false));
    let server_ready_clone = server_ready.clone();

    // Start the warp server in a separate thread
    std::thread::spawn(move || {
        let run_server = async move {
            let (_addr, server) = warp::serve(routes)
                .bind_with_graceful_shutdown(([127, 0, 0, 1], 3030), async {
                    tokio::signal::ctrl_c().await.ok();
                });
            server_ready_clone.store(true, Ordering::SeqCst);
            server.await;
        };
        tokio::runtime::Runtime::new().unwrap().block_on(run_server);
    });

    // Wait for the server to be ready
    while !server_ready.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(100));
    }
    println!("Server is ready at http://localhost:3030");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let window = WindowBuilder::new()
        .with_title("Web4")
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop)?;

    let event_loop_proxy = event_loop.create_proxy();

    let mut web_context = WebContext::new(None);
    let mut tabs = vec![];
    let mut current_tab = 0;

    // Create initial tab
    let initial_tab = create_tab(&window, &mut web_context, "https://www.google.com", &event_loop_proxy)?;
    tabs.push(initial_tab);

    let mut modifiers = ModifiersState::default();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => println!("Web4 is initializing..."),
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { event, .. },
                ..
            } => {
                if let ElementState::Pressed = event.state {
                    match event.physical_key {
                        KeyCode::Enter => {
                            if let Some(url) = get_url_from_address_bar(&tabs[current_tab].webview) {
                                load_url_in_current_tab(&mut tabs[current_tab], &url);
                            }
                        },
                        KeyCode::KeyT => {
                            // Create a new tab (Ctrl+T)
                            if modifiers.control_key() {
                                if let Ok(new_tab) = create_tab(&window, &mut web_context, "http://localhost:3030", &event_loop_proxy) {
                                    tabs.push(new_tab);
                                    current_tab = tabs.len() - 1;
                                    update_tab_ui(&tabs, current_tab);
                                }
                            }
                        },
                        KeyCode::Tab => {
                            // Switch tabs (Ctrl+Tab)
                            if modifiers.control_key() {
                                current_tab = (current_tab + 1) % tabs.len();
                                update_tab_ui(&tabs, current_tab);
                            }
                        },
                        _ => (),
                    }
                }
            },
            Event::WindowEvent { event: WindowEvent::ModifiersChanged(new_modifiers), .. } => {
                modifiers = new_modifiers;
            },
            Event::UserEvent(UserEvent::LoadUrl(url)) => {
                load_url_in_current_tab(&mut tabs[current_tab], &url);
                update_tab_ui(&tabs, current_tab);
            },
            Event::UserEvent(UserEvent::CreateNewTab) => {
                if let Ok(new_tab) = create_tab(&window, &mut web_context, "http://localhost:3030", &event_loop_proxy) {
                    tabs.push(new_tab);
                    current_tab = tabs.len() - 1;
                    update_tab_ui(&tabs, current_tab);
                }
            },
            Event::UserEvent(UserEvent::SwitchTab(index)) => {
                if index < tabs.len() {
                    current_tab = index;
                    update_tab_ui(&tabs, current_tab);
                }
            },
            _ => (),
        }
    })
}

fn create_tab(window: &Window, context: &mut WebContext, url: &str, event_loop_proxy: &EventLoopProxy<UserEvent>) -> Result<Tab, Box<dyn std::error::Error>> {
    let event_loop_proxy = event_loop_proxy.clone();
    println!("Creating new tab with URL: {}", url);
    let html_content = include_str!("index.html");
    println!("HTML content length: {}", html_content.len());
    let webview = WebViewBuilder::new(window)
        .with_web_context(context)
        .with_html(html_content)
        .with_initialization_script(&format!("{}\n{}", 
            include_str!("tab.js"),
            format!("console.log('Initialization script running...'); initializePage('{}');", url)
        ))
        .with_ipc_handler(move |request| {
            println!("IPC request received: {:?}", request);
            let request_body = request.body();
            if let Some(url) = request_body.strip_prefix("loadUrl:") {
                event_loop_proxy.send_event(UserEvent::LoadUrl(url.to_string())).ok();
            } else if request_body == "createNewTab" {
                event_loop_proxy.send_event(UserEvent::CreateNewTab).ok();
            } else if let Some(tab_index) = request_body.strip_prefix("switchTab:") {
                if let Ok(index) = tab_index.parse::<usize>() {
                    event_loop_proxy.send_event(UserEvent::SwitchTab(index)).ok();
                }
            }
        })
        .build()?;
    
    println!("Tab created successfully");
    Ok(Tab { webview, url: url.to_string() })
}

fn load_url_in_current_tab(tab: &mut Tab, url: &str) {
    println!("Loading URL in current tab: {}", url);
    let script = format!("loadUrl('{}');", url);
    match tab.webview.evaluate_script(&script) {
        Ok(_) => {
            tab.url = url.to_string();
            println!("Successfully loaded URL: {}", url);
        },
        Err(e) => {
            println!("Error loading URL {}: {:?}", url, e);
        }
    }
}

fn get_url_from_address_bar(webview: &WebView) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    
    // The JavaScript function to get the URL
    let script = "document.getElementById('url-input').value";
    
    webview.evaluate_script_with_callback(script, move |result| {
        tx.send(result.clone()).unwrap();
    }).ok()?;
    
    rx.recv().ok().and_then(|s| {
        serde_json::from_str(&s).ok()
    }).map(|s: String| s.trim().to_string())
}

fn update_tab_ui(tabs: &[Tab], current_tab: usize) {
    let tab_list = tabs.iter().enumerate().map(|(i, tab)| {
        format!("<div class='tab{}' onclick='switchTab({})'>Tab {}</div>",
            if i == current_tab { " active" } else { "" },
            i,
            i + 1
        )
    }).collect::<Vec<_>>().join("");
    
    let script = format!(
        r#"
        document.getElementById('tab-bar').innerHTML = "{}";
        document.getElementById('url-input').value = "{}";
        "#,
        tab_list,
        tabs[current_tab].url
    );

    if let Err(e) = tabs[current_tab].webview.evaluate_script(&script) {
        println!("Error updating tab UI: {:?}", e);
    }
}

async fn handle_load_url_get(
    params: std::collections::HashMap<String, String>,
    engine: Arc<BrowserEngine>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let url = params.get("url").cloned().unwrap_or_default();
    println!("Handling load URL (GET) request for: {}", url);
    match engine.load_url(&url).await {
        Ok(mut result) => {
            // Check if the URL has changed (due to redirects)
            if result.url != url {
                println!("Redirect detected: {} -> {}", url, result.url);
                // Load the new URL
                match engine.load_url(&result.url).await {
                    Ok(new_result) => result = new_result,
                    Err(e) => {
                        println!("Error following redirect: {:?}", e);
                        // Return an error response
                        let error_response = serde_json::json!({
                            "error": format!("Error following redirect: {}", e)
                        });
                        return Ok(warp::reply::with_header(
                            warp::reply::json(&error_response),
                            "Content-Type",
                            "application/json"
                        ));
                    }
                }
            }

            let response = serde_json::json!({
                "url": result.url,
                "summary": result.summary,
                "analysis": result.analysis,
                "content": result.content,
                "baseUrl": result.url  // Use the final URL after potential redirects
            });
            println!("Server response (GET): {:?}", response);
            Ok(warp::reply::with_header(
                warp::reply::json(&response),
                "Content-Type",
                "application/json"
            ))
        },
        Err(e) => {
            let error_response = serde_json::json!({
                "error": format!("Error loading URL: {}", e)
            });
            println!("Server error response (GET): {:?}", error_response);
            Ok(warp::reply::with_header(
                warp::reply::json(&error_response),
                "Content-Type",
                "application/json"
            ))
        },
    }
}