use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow, Box as GtkBox, Button, Entry, Label, Orientation, ProgressBar, ScrolledWindow, ToggleButton};
use webkit::prelude::*;
use webkit::{CacheModel, HardwareAccelerationPolicy, LoadEvent, Settings, WebView};

const APP_ID: &str = "com.example.rust_multimedia_browser";
const HOME_PAGE: &str = "https://duckduckgo.com";
const WINDOW_TITLE: &str = "Rust Multimedia Browser";

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title(WINDOW_TITLE)
        .default_width(1280)
        .default_height(900)
        .build();

    let root = GtkBox::new(Orientation::Vertical, 6);
    root.set_margin_top(6);
    root.set_margin_bottom(6);
    root.set_margin_start(6);
    root.set_margin_end(6);

    let toolbar = GtkBox::new(Orientation::Horizontal, 6);
    let toolbar_2 = GtkBox::new(Orientation::Horizontal, 6);

    let back_button = Button::with_label("←");
    let forward_button = Button::with_label("→");
    let reload_button = Button::with_label("Reload");
    let stop_button = Button::with_label("Stop");
    let home_button = Button::with_label("Home");
    let go_button = Button::with_label("Go");
    let zoom_out_button = Button::with_label("A-");
    let zoom_reset_button = Button::with_label("A");
    let zoom_in_button = Button::with_label("A+");
    let mute_button = ToggleButton::with_label("Mute");

    back_button.set_sensitive(false);
    forward_button.set_sensitive(false);
    stop_button.set_sensitive(false);

    let address = Entry::new();
    address.set_hexpand(true);
    address.set_placeholder_text(Some("Enter a URL or search terms"));

    let progress = ProgressBar::new();
    progress.set_show_text(false);
    progress.set_fraction(0.0);
    progress.set_visible(false);

    let status = Label::new(Some("Ready"));
    status.set_xalign(0.0);

    let webview = WebView::new();
    let settings = configured_settings();
    webview.set_settings(&settings);

    if let Some(context) = webview.web_context() {
        context.set_cache_model(CacheModel::WebBrowser);
    }

    let scroller = ScrolledWindow::builder()
        .hexpand(true)
        .vexpand(true)
        .build();
    scroller.set_child(Some(&webview));

    toolbar.append(&back_button);
    toolbar.append(&forward_button);
    toolbar.append(&reload_button);
    toolbar.append(&stop_button);
    toolbar.append(&home_button);
    toolbar.append(&address);
    toolbar.append(&go_button);

    toolbar_2.append(&zoom_out_button);
    toolbar_2.append(&zoom_reset_button);
    toolbar_2.append(&zoom_in_button);
    toolbar_2.append(&mute_button);

    root.append(&toolbar);
    root.append(&toolbar_2);
    root.append(&progress);
    root.append(&scroller);
    root.append(&status);

    window.set_child(Some(&root));

    {
        let webview = webview.clone();
        address.connect_activate(move |entry| {
            if let Some(target) = normalize_target(entry.text().as_str()) {
                webview.load_uri(&target);
            }
        });
    }

    {
        let webview = webview.clone();
        let address = address.clone();
        go_button.connect_clicked(move |_| {
            if let Some(target) = normalize_target(address.text().as_str()) {
                webview.load_uri(&target);
            }
        });
    }

    {
        let webview = webview.clone();
        back_button.connect_clicked(move |_| {
            if webview.can_go_back() {
                webview.go_back();
            }
        });
    }

    {
        let webview = webview.clone();
        forward_button.connect_clicked(move |_| {
            if webview.can_go_forward() {
                webview.go_forward();
            }
        });
    }

    {
        let webview = webview.clone();
        reload_button.connect_clicked(move |_| {
            webview.reload();
        });
    }

    {
        let webview = webview.clone();
        stop_button.connect_clicked(move |_| {
            if webview.is_loading() {
                webview.stop_loading();
            }
        });
    }

    {
        let webview = webview.clone();
        let address = address.clone();
        home_button.connect_clicked(move |_| {
            address.set_text(HOME_PAGE);
            webview.load_uri(HOME_PAGE);
        });
    }

    {
        let webview = webview.clone();
        zoom_out_button.connect_clicked(move |_| {
            adjust_zoom(&webview, -0.10);
        });
    }

    {
        let webview = webview.clone();
        zoom_reset_button.connect_clicked(move |_| {
            webview.set_zoom_level(1.0);
        });
    }

    {
        let webview = webview.clone();
        zoom_in_button.connect_clicked(move |_| {
            adjust_zoom(&webview, 0.10);
        });
    }

    {
        let webview = webview.clone();
        mute_button.connect_toggled(move |button| {
            let muted = button.is_active();
            webview.set_is_muted(muted);
            if muted {
                button.set_label("Muted");
            } else {
                button.set_label("Mute");
            }
        });
    }

    {
        let address = address.clone();
        webview.connect_uri_notify(move |wv| {
            if let Some(uri) = wv.uri() {
                if address.text().as_str() != uri.as_str() {
                    address.set_text(uri.as_str());
                }
            }
        });
    }

    {
        let window = window.clone();
        webview.connect_title_notify(move |wv| {
            let title = wv
                .title()
                .map(|t| t.to_string())
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| WINDOW_TITLE.to_string());
            window.set_title(Some(&format!("{title} — {WINDOW_TITLE}")));
        });
    }

    {
        let progress = progress.clone();
        webview.connect_estimated_load_progress_notify(move |wv| {
            progress.set_fraction(wv.estimated_load_progress().clamp(0.0, 1.0));
        });
    }

    {
        let status = status.clone();
        webview.connect_is_playing_audio_notify(move |wv| {
            if wv.is_playing_audio() {
                status.set_text("Media is playing");
            } else if !wv.is_loading() {
                status.set_text("Ready");
            }
        });
    }

    {
        let status = status.clone();
        let mute_button = mute_button.clone();
        webview.connect_is_muted_notify(move |wv| {
            let muted = wv.is_muted();
            if mute_button.is_active() != muted {
                mute_button.set_active(muted);
            }
            if muted {
                status.set_text("Audio muted");
            }
        });
    }

    {
        let back_button = back_button.clone();
        let forward_button = forward_button.clone();
        let reload_button = reload_button.clone();
        let stop_button = stop_button.clone();
        let progress = progress.clone();
        webview.connect_is_loading_notify(move |wv| {
            sync_chrome(
                wv,
                &back_button,
                &forward_button,
                &reload_button,
                &stop_button,
                &progress,
            );
        });
    }

    {
        let back_button = back_button.clone();
        let forward_button = forward_button.clone();
        let reload_button = reload_button.clone();
        let stop_button = stop_button.clone();
        let progress = progress.clone();
        webview.connect_uri_notify(move |wv| {
            sync_chrome(
                wv,
                &back_button,
                &forward_button,
                &reload_button,
                &stop_button,
                &progress,
            );
        });
    }

    {
        let status = status.clone();
        let progress = progress.clone();
        webview.connect_load_changed(move |wv, event| {
            match event {
                LoadEvent::Started => {
                    progress.set_visible(true);
                    progress.set_fraction(0.0);
                    status.set_text("Loading...");
                }
                LoadEvent::Redirected => {
                    status.set_text("Redirecting...");
                }
                LoadEvent::Committed => {
                    if let Some(uri) = wv.uri() {
                        status.set_text(&format!("Opened {}", uri.as_str()));
                    }
                }
                LoadEvent::Finished => {
                    progress.set_fraction(1.0);
                    progress.set_visible(false);
                    if wv.is_playing_audio() {
                        status.set_text("Finished loading • media is playing");
                    } else {
                        status.set_text("Finished loading");
                    }
                }
                _ => {}
            }
        });
    }

    {
        let status = status.clone();
        let progress = progress.clone();
        webview.connect_load_failed(move |_wv, _event, uri, error| {
            progress.set_visible(false);
            status.set_text(&format!("Load failed: {uri} ({error})"));
            false
        });
    }

    {
        let status = status.clone();
        webview.connect_permission_request(move |_wv, request| {
            request.deny();
            status.set_text("Blocked permission request (camera/microphone/location not enabled in this build)");
            true
        });
    }

    webview.load_uri(HOME_PAGE);
    address.set_text(HOME_PAGE);
    window.present();
}

fn configured_settings() -> Settings {
    let settings = Settings::builder().build();
    settings.set_enable_javascript(true);
    settings.set_enable_javascript_markup(true);
    settings.set_enable_media(true);
    settings.set_enable_media_capabilities(true);
    settings.set_enable_mediasource(true);
    settings.set_enable_webaudio(true);
    settings.set_enable_webgl(true);
    settings.set_enable_webrtc(true);
    settings.set_enable_encrypted_media(true);
    settings.set_enable_fullscreen(true);
    settings.set_enable_html5_database(true);
    settings.set_enable_html5_local_storage(true);
    settings.set_enable_page_cache(true);
    settings.set_enable_site_specific_quirks(true);
    settings.set_enable_smooth_scrolling(true);
    settings.set_enable_developer_extras(true);
    settings.set_javascript_can_open_windows_automatically(false);
    settings.set_hardware_acceleration_policy(HardwareAccelerationPolicy::Always);
    settings.set_user_agent_with_application_details(
        Some("Rust Multimedia Browser"),
        Some(env!("CARGO_PKG_VERSION")),
    );
    settings
}

fn normalize_target(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("file://")
        || lower.starts_with("about:")
        || lower.starts_with("data:")
    {
        return Some(trimmed.to_string());
    }

    if trimmed.contains(char::is_whitespace) {
        return Some(format!(
            "https://duckduckgo.com/?q={}",
            urlencoding::encode(trimmed)
        ));
    }

    Some(format!("https://{trimmed}"))
}

fn adjust_zoom(webview: &WebView, delta: f64) {
    let new_zoom = (webview.zoom_level() + delta).clamp(0.30, 3.00);
    webview.set_zoom_level(new_zoom);
}

fn sync_chrome(
    webview: &WebView,
    back_button: &Button,
    forward_button: &Button,
    reload_button: &Button,
    stop_button: &Button,
    progress: &ProgressBar,
) {
    let loading = webview.is_loading();
    back_button.set_sensitive(webview.can_go_back());
    forward_button.set_sensitive(webview.can_go_forward());
    reload_button.set_sensitive(!loading);
    stop_button.set_sensitive(loading);
    progress.set_visible(loading);
}
