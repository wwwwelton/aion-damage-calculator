// Windows host: a tao window with a wry WebView2 surface that serves the embedded UI from an
// https://aion2.localhost origin and relays character and guide requests to the native services.
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use tokio::runtime::Runtime;
use wry::http::{Request, Response, Uri};
use wry::{WebContext, WebViewBuilder, WebViewBuilderExtWindows};

use crate::aionlabs::AionLabsClient;
use crate::guides::GuideClient;
use crate::official::OfficialClient;
use crate::profile_cache::ProfileCache;
use crate::profile_sync::ProfileSync;
use crate::{assets, protocol};

const HOME: &str = "https://aion2.localhost/index.html";
const HOST_BRIDGE: &str = include_str!("../host-bridge.js");

enum UserEvent {
    Reply(String),
}

pub fn run(runtime: Runtime) {
    let official = match OfficialClient::new() {
        Ok(client) => Arc::new(client),
        Err(error) => return show_error("Unable to open calculator", &error),
    };
    let guides = match GuideClient::new() {
        Ok(client) => Arc::new(client),
        Err(error) => return show_error("Unable to open calculator", &error),
    };
    let ai = match AionLabsClient::new() {
        Ok(client) => Arc::new(client),
        Err(error) => return show_error("Unable to open calculator", &error),
    };
    let profiles = match ProfileCache::open(data_folder().with_file_name("profiles.json")).and_then(|cache| ProfileSync::new(cache, Arc::clone(&official))) {
        Ok(profiles) => Arc::new(profiles),
        Err(error) => return show_error("Cannot open saved profiles", &error),
    };

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();
    let window = match WindowBuilder::new()
        .with_title("AION 2 — Damage Calculator")
        .with_inner_size(LogicalSize::new(1100.0, 820.0))
        .with_min_inner_size(LogicalSize::new(760.0, 600.0))
        .build(&event_loop)
    {
        Ok(window) => window,
        Err(error) => return show_error("Unable to open calculator", &error.to_string()),
    };

    let mut web_context = WebContext::new(Some(data_folder()));
    let handle = runtime.handle().clone();
    let background = Arc::clone(&profiles);
    let background_proxy = proxy.clone();
    handle.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last_sync = None::<std::time::Instant>;
        loop {
            interval.tick().await;
            let Ok(settings) = background.settings() else { continue };
            let seconds = settings["interval"].as_u64().unwrap_or(600);
            if settings["enabled"] == true && last_sync.is_none_or(|last| last.elapsed().as_secs() >= seconds) {
                background.refresh_all(|reply| { let _ = background_proxy.send_event(UserEvent::Reply(reply)); }).await;
                last_sync = Some(std::time::Instant::now());
            }
        }
    });
    let webview = match WebViewBuilder::new_with_web_context(&mut web_context)
        .with_https_scheme(true)
        .with_initialization_script(HOST_BRIDGE)
        .with_custom_protocol("aion2".to_string(), |_id, request| serve(&request))
        .with_navigation_handler(|url| url.starts_with("https://aion2.localhost/"))
        .with_ipc_handler(move |request: Request<String>| {
            if !is_app_origin(request.uri()) {
                return;
            }
            let message = request.into_body();
            let official = Arc::clone(&official);
            let guides = Arc::clone(&guides);
            let ai = Arc::clone(&ai);
            let profiles = Arc::clone(&profiles);
            let proxy = proxy.clone();
            handle.spawn(async move {
                if profiles.respond(&message, |reply| { let _ = proxy.send_event(UserEvent::Reply(reply)); }).await {
                    return;
                }
                if let Some(reply) = protocol::respond(&message, &official, &guides, &ai).await {
                    let _ = proxy.send_event(UserEvent::Reply(reply));
                }
            });
        })
        .with_url(HOME)
        .build(&window)
    {
        Ok(webview) => webview,
        Err(error) => {
            return show_error(
                "Unable to open calculator",
                &format!("Could not start WebView2. Install the Microsoft Edge WebView2 Runtime.\n\n{error}"),
            )
        }
    };

    // The event loop never returns, so the runtime and the WebView2 controller stay alive for the process.
    let _runtime = runtime;
    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::UserEvent(UserEvent::Reply(json)) => {
                let _ = webview.evaluate_script(&format!("aion2Host.receive({json})"));
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn serve(request: &Request<Vec<u8>>) -> Response<Cow<'static, [u8]>> {
    match assets::lookup(request.uri().path()) {
        Some(asset) => Response::builder()
            .status(200)
            .header("Content-Type", asset.content_type)
            .header("Cache-Control", asset.cache_control)
            .body(asset.data)
            .expect("static response headers are valid"),
        None => Response::builder()
            .status(404)
            .header("Content-Type", "text/plain")
            .body(Cow::Borrowed(&b"Not Found"[..]))
            .expect("static response headers are valid"),
    }
}

// Messages are accepted only from the embedded application page, never from navigated external sites.
fn is_app_origin(uri: &Uri) -> bool {
    uri.scheme_str() == Some("https") && uri.host() == Some(protocol::APP_ORIGIN_HOST)
}

// Same profile folder as the C# build, so WebView2 storage carries over.
fn data_folder() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("AION2-Calculadora")
        .join("WebView2")
}

fn show_error(title: &str, message: &str) {
    rfd::MessageDialog::new()
        .set_title(title)
        .set_description(message)
        .set_level(rfd::MessageLevel::Error)
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
}
