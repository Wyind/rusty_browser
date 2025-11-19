// --- IMPORTS ---
use gtk::prelude::*;
use gtk::glib; 
use gtk::{
    Application, ApplicationWindow, Box, Orientation, Entry, Button, 
    Label, Dialog, ResponseType, Switch, Separator, LinkButton,
    ScrolledWindow, PolicyType, CssProvider, ProgressBar, DropDown, StringList,
    Image, Window, Align
};
use gtk::gdk_pixbuf::PixbufLoader;
use gtk::gdk;
use webkit6::prelude::*;
use webkit6::{WebView, HardwareAccelerationPolicy, WebContext, UserContentManager, UserStyleSheet, UserContentInjectedFrames, UserStyleLevel};
use std::env;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

// --- CONFIGURATION STRUCTS ---

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AppConfig {
    homepage: String,
    use_hw_accel: bool,
    enable_adblock: bool,
    amnesia_mode: bool,
    show_home_button: bool,
    search_engine_url: String,
    search_engine_index: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            homepage: "https://duckduckgo.com".to_string(),
            use_hw_accel: true, 
            enable_adblock: true,
            amnesia_mode: false,
            show_home_button: true,
            search_engine_url: "https://duckduckgo.com/?q=".to_string(),
            search_engine_index: 0,
        }
    }
}

// --- SAVE/LOAD HELPERS ---
fn get_config_path() -> PathBuf {
    let mut path = glib::user_config_dir();
    path.push("rusty_browser");
    std::fs::create_dir_all(&path).unwrap_or_default();
    path.push("settings.json");
    path
}

fn load_config() -> AppConfig {
    let path = get_config_path();
    if let Ok(contents) = fs::read_to_string(path) {
        if let Ok(config) = serde_json::from_str(&contents) {
            return config;
        }
    }
    AppConfig::default()
}

fn save_config(config: &AppConfig) {
    let path = get_config_path();
    if let Ok(json) = serde_json::to_string_pretty(config) {
        let _ = fs::write(path, json);
    }
}

fn main() {
    // --- CONFIGURATION ---
    env::set_var("GDK_BACKEND", "x11");
    env::set_var("GST_GL_WINDOW", "x11");
    env::set_var("LIBVA_DRIVER_NAME", "nvidia");
    env::set_var("WEBKIT_DISABLE_SANDBOX_GPU_PROCESS", "1");

    let app = Application::builder()
        .application_id("com.titan.rustybrowser")
        .build();

    app.connect_startup(|_| {
        let provider = CssProvider::new();
        provider.load_from_data("
            /* Global Dark Theme */
            window, scrolledwindow, notebook, dialog, popover { background-color: #1e1e20; color: #ececec; }
            
            /* Toolbar */
            .toolbar { background-color: #1e1e20; border-bottom: 1px solid #000000; padding: 6px 12px; min-height: 36px; }
            
            /* Inputs */
            entry { background-color: #2a2a2c; color: white; border: 1px solid #3a3a3c; border-radius: 12px; padding: 2px 12px; margin: 0 10px; caret-color: #3daee9; min-height: 28px; box-shadow: none; }
            entry:focus { background-color: #323234; border-color: #3daee9; }
            
            /* Buttons */
            button { background-color: transparent; color: #b0b0b0; border: none; border-radius: 6px; margin: 0 2px; padding: 2px; min-height: 32px; min-width: 32px; box-shadow: none; }
            button:hover { background-color: rgba(255, 255, 255, 0.1); color: white; }
            button:active { background-color: rgba(61, 174, 233, 0.3); color: #3daee9; }
            
            .incognito-btn { color: #d4af37; }
            .incognito-btn:hover { background-color: rgba(212, 175, 55, 0.2); }

            .flat-button { padding: 5px 10px; border-radius: 5px; background-color: rgba(255, 255, 255, 0.05); color: #ececec; }
            .flat-button:hover { background-color: rgba(255, 255, 255, 0.1); }

            /* About Window Styling */
            .about-title { font-size: 24px; font-weight: bold; margin-bottom: 5px; }
            .about-version { color: #808080; margin-bottom: 20px; }
            .about-box { padding: 30px; }

            /* Progress Bar */
            progressbar trough { min-height: 2px; background: transparent; border: none; }
            progressbar progress { background-color: #3daee9; min-height: 2px; border-radius: 0; }
            
            /* Tabs */
            notebook header { background-color: #151516; padding: 0; min-height: 28px; }
            tab { background-color: transparent; border: none; padding: 2px 8px; color: #808080; font-size: 12px; margin-right: 1px; }
            tab:checked { background-color: #1e1e20; color: white; border-top: 2px solid #3daee9; }

            /* Tab Close Button */
            .tab-close-btn {
                min-width: 16px;
                min-height: 16px;
                padding: 0;
                margin-left: 8px;
                background-color: transparent;
                color: #808080;
                border-radius: 100%;
            }
            .tab-close-btn:hover {
                background-color: rgba(255, 80, 80, 0.2);
                color: #ff5f56;
            }
        ");

        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
        }
    });

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // LOAD SETTINGS FROM DISK
    let loaded_config = load_config();
    let app_state = Rc::new(RefCell::new(loaded_config));

    // --- PERSISTENCE ---
    let persistent_context = WebContext::default().unwrap();
    let shared_persistent_context = Rc::new(persistent_context);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Rusty Browser")
        .default_width(1200)
        .default_height(800)
        .build();

    let main_box = Box::new(Orientation::Vertical, 0);
    window.set_child(Some(&main_box));

    // --- TOOLBAR ---
    let toolbar = Box::new(Orientation::Horizontal, 0);
    toolbar.add_css_class("toolbar"); 
    
    let back_btn = Button::builder().icon_name("go-previous-symbolic").tooltip_text("Go Back").build();
    let forward_btn = Button::builder().icon_name("go-next-symbolic").tooltip_text("Go Forward").build();
    let refresh_btn = Button::builder().icon_name("view-refresh-symbolic").tooltip_text("Reload").build();
    let home_btn = Button::builder().icon_name("go-home-symbolic").tooltip_text("Home").build();
    
    // Apply loaded visibility state
    home_btn.set_visible(app_state.borrow().show_home_button);

    let url_bar = Entry::new();
    url_bar.set_hexpand(true);
    url_bar.set_placeholder_text(Some("Search or enter URL"));
    
    let new_tab_btn = Button::builder().icon_name("tab-new-symbolic").tooltip_text("New Tab").build();
    let incognito_btn = Button::builder().icon_name("weather-clear-night-symbolic").tooltip_text("Incognito").build();
    incognito_btn.add_css_class("incognito-btn");
    let settings_btn = Button::builder().icon_name("emblem-system-symbolic").tooltip_text("Settings").build();

    toolbar.append(&back_btn);
    toolbar.append(&forward_btn);
    toolbar.append(&refresh_btn);
    toolbar.append(&home_btn);
    toolbar.append(&url_bar);
    toolbar.append(&new_tab_btn);
    toolbar.append(&incognito_btn);
    toolbar.append(&settings_btn);
    main_box.append(&toolbar);

    let progress_bar = ProgressBar::new();
    progress_bar.set_visible(false);
    main_box.append(&progress_bar);

    let notebook = gtk::Notebook::new();
    notebook.set_scrollable(true);
    notebook.set_vexpand(true);
    main_box.append(&notebook);

    // --- TAB LOGIC ---
    let notebook_weak = notebook.downgrade();
    let url_bar_weak = url_bar.downgrade();
    let window_weak = window.downgrade();
    let progress_bar_weak = progress_bar.downgrade();
    let state_clone = app_state.clone();
    let persistent_ctx_clone = shared_persistent_context.clone();

    let create_tab = Rc::new(move |url: &str, is_incognito: bool| {
        let notebook = match notebook_weak.upgrade() {
            Some(n) => n,
            None => return,
        };

        let user_manager = UserContentManager::new();

        if state_clone.borrow().enable_adblock {
            let adblock_css = "iframe[src*='ads'], div[class*='ad-'], div[id*='google_ads'], .adsbygoogle, .ad-banner { display: none !important; }";
            let style = UserStyleSheet::new(adblock_css, UserContentInjectedFrames::AllFrames, UserStyleLevel::User, &[], &[]);
            user_manager.add_style_sheet(&style);
        }

        let webview: WebView;
        if is_incognito || state_clone.borrow().amnesia_mode {
            let ephemeral_ctx = WebContext::new(); 
            webview = glib::Object::builder().property("web-context", &ephemeral_ctx).property("user-content-manager", &user_manager).build();
        } else {
            webview = glib::Object::builder().property("web-context", persistent_ctx_clone.as_ref()).property("user-content-manager", &user_manager).build();
        }
        
        let use_accel = state_clone.borrow().use_hw_accel;

        if let Some(settings) = WebViewExt::settings(&webview) {
            if use_accel {
                settings.set_hardware_acceleration_policy(HardwareAccelerationPolicy::Always);
                settings.set_enable_webgl(true);
            } else {
                settings.set_hardware_acceleration_policy(HardwareAccelerationPolicy::Never);
                settings.set_enable_webgl(false);
            }
            settings.set_enable_media_stream(true);
            settings.set_enable_mediasource(true);
            settings.set_enable_smooth_scrolling(false); 
            settings.set_enable_developer_extras(true);
            settings.set_user_agent(Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));
        }

        webview.load_uri(url);

        let tab_box = Box::new(Orientation::Horizontal, 0);
        tab_box.set_valign(Align::Center);
        
        let label_text = if is_incognito { "🕵️ Loading..." } else { "Loading..." };
        let tab_label = Label::new(Some(label_text));
        
        let close_btn = Button::builder().icon_name("window-close-symbolic").build();
        close_btn.add_css_class("tab-close-btn");

        tab_box.append(&tab_label);
        tab_box.append(&close_btn);
        tab_box.show();

        let page_idx = notebook.append_page(&webview, Some(&tab_box));
        notebook.set_tab_reorderable(&webview, true);
        notebook.set_current_page(Some(page_idx));

        let nb_close = notebook.clone();
        let wv_close = webview.clone();
        close_btn.connect_clicked(move |_| {
            if let Some(idx) = nb_close.page_num(&wv_close) {
                nb_close.remove_page(Some(idx));
            }
        });

        let url_bar_weak = url_bar_weak.clone();
        webview.connect_uri_notify(move |wv| {
            if let Some(u) = url_bar_weak.upgrade() { if let Some(uri) = wv.uri() { u.set_text(uri.as_str()); } }
        });

        let window_weak_title = window_weak.clone();
        let notebook_weak_title = notebook.downgrade();
        let label_clone = tab_label.clone();

        webview.connect_title_notify(move |wv| {
             if let Some(title) = wv.title() {
                let short_title = if is_incognito { 
                    format!("🕵️ {}", &title.as_str()[0..std::cmp::min(15, title.as_str().len())]) 
                } else { 
                    format!("{}", &title.as_str()[0..std::cmp::min(15, title.as_str().len())]) 
                };
                label_clone.set_text(&short_title);

                if let (Some(win), Some(nb)) = (window_weak_title.upgrade(), notebook_weak_title.upgrade()) {
                    if let Some(page_idx) = nb.page_num(wv) {
                        if Some(page_idx) == nb.current_page() {
                            win.set_title(Some(&format!("{} - Rusty Browser", title.as_str())));
                        }
                    }
                }
             }
        });

        let progress_weak = progress_bar_weak.clone();
        webview.connect_estimated_load_progress_notify(move |wv| {
            if let Some(bar) = progress_weak.upgrade() {
                let progress = wv.estimated_load_progress();
                bar.set_fraction(progress);
                if progress >= 1.0 { bar.set_visible(false); } else { bar.set_visible(true); }
            }
        });

        notebook.show();
    });

    (create_tab)(&app_state.borrow().homepage, false);

    // --- ACTIONS ---
    let create_tab_clone = create_tab.clone();
    let state_clone_new = app_state.clone();
    new_tab_btn.connect_clicked(move |_| { (create_tab_clone)(&state_clone_new.borrow().homepage, false); });

    let create_tab_incog = create_tab.clone();
    let state_clone_incog = app_state.clone();
    incognito_btn.connect_clicked(move |_| { (create_tab_incog)(&state_clone_incog.borrow().homepage, true); });

    let notebook_clone = notebook.clone();
    let state_clone_search = app_state.clone();
    url_bar.connect_activate(move |entry| {
        if let Some(page) = notebook_clone.nth_page(notebook_clone.current_page()) {
            if let Ok(webview) = page.downcast::<WebView>() {
                let input = entry.text().to_string();
                let target_url = if input.contains("://") { input } 
                else if !input.contains('.') || input.contains(' ') {
                     let engine = &state_clone_search.borrow().search_engine_url;
                     format!("{}{}", engine, input)
                } else { format!("https://{}", input) };
                webview.load_uri(&target_url);
            }
        }
    });

    let notebook_clone = notebook.clone();
    back_btn.connect_clicked(move |_| {
        if let Some(page) = notebook_clone.nth_page(notebook_clone.current_page()) {
            if let Ok(webview) = page.downcast::<WebView>() { if webview.can_go_back() { webview.go_back(); } }
        }
    });

    let notebook_clone = notebook.clone();
    forward_btn.connect_clicked(move |_| {
        if let Some(page) = notebook_clone.nth_page(notebook_clone.current_page()) {
            if let Ok(webview) = page.downcast::<WebView>() { if webview.can_go_forward() { webview.go_forward(); } }
        }
    });

    let notebook_clone = notebook.clone();
    refresh_btn.connect_clicked(move |_| {
        if let Some(page) = notebook_clone.nth_page(notebook_clone.current_page()) {
            if let Ok(webview) = page.downcast::<WebView>() { webview.reload(); }
        }
    });

    let notebook_clone = notebook.clone();
    let state_clone_home = app_state.clone();
    home_btn.connect_clicked(move |_| {
        if let Some(page) = notebook_clone.nth_page(notebook_clone.current_page()) {
            if let Ok(webview) = page.downcast::<WebView>() { webview.load_uri(&state_clone_home.borrow().homepage); }
        }
    });

    let notebook_clone = notebook.clone();
    let url_bar_clone = url_bar.clone();
    let window_clone = window.clone();
    notebook.connect_switch_page(move |_, widget, _| {
        if let Ok(webview) = widget.clone().downcast::<WebView>() {
            if let Some(uri) = webview.uri() { url_bar_clone.set_text(&uri); }
            if let Some(title) = webview.title() { window_clone.set_title(Some(&format!("{} - Rusty Browser", title))); }
            else { window_clone.set_title(Some("Rusty Browser")); }
        }
    });

    // --- SETTINGS ---
    let window_clone = window.clone();
    let state_clone_settings = app_state.clone();
    let home_btn_clone = home_btn.clone();

    settings_btn.connect_clicked(move |_| {
        let dialog = Dialog::builder().transient_for(&window_clone).modal(true).title("Settings").build();
        dialog.add_button("Close", ResponseType::Close);
        let content_area = dialog.content_area(); 
        let scroll = ScrolledWindow::builder().hscrollbar_policy(PolicyType::Never).min_content_width(400).min_content_height(400).build();
        let vbox = Box::new(Orientation::Vertical, 10);
        vbox.set_margin_top(20); vbox.set_margin_bottom(20); vbox.set_margin_start(20); vbox.set_margin_end(20);

        let label_gen = Label::new(None); label_gen.set_markup("<b>General</b>"); label_gen.set_halign(gtk::Align::Start); vbox.append(&label_gen);
        let home_entry = Entry::new(); home_entry.set_text(&state_clone_settings.borrow().homepage); vbox.append(&home_entry);
        let show_home_switch = Switch::new(); show_home_switch.set_active(state_clone_settings.borrow().show_home_button);
        let show_home_box = Box::new(Orientation::Horizontal, 10); show_home_box.append(&show_home_switch); show_home_box.append(&Label::new(Some("Show Home Button"))); vbox.append(&show_home_box);

        vbox.append(&Separator::new(Orientation::Horizontal));
        let label_search = Label::new(None); label_search.set_markup("<b>Search Engine</b>"); label_search.set_halign(gtk::Align::Start); vbox.append(&label_search);
        let engines = StringList::new(&["DuckDuckGo", "Google", "Bing", "Brave"]);
        let dropdown = DropDown::new(Some(engines), Option::<gtk::Expression>::None);
        dropdown.set_selected(state_clone_settings.borrow().search_engine_index);
        vbox.append(&dropdown);

        vbox.append(&Separator::new(Orientation::Horizontal));
        let label_perf = Label::new(None); label_perf.set_markup("<b>Performance &amp; Privacy</b>"); label_perf.set_halign(gtk::Align::Start); vbox.append(&label_perf);
        let hw_switch = Switch::new(); hw_switch.set_active(state_clone_settings.borrow().use_hw_accel);
        let hw_box = Box::new(Orientation::Horizontal, 10); hw_box.append(&hw_switch); hw_box.append(&Label::new(Some("Hardware Acceleration"))); vbox.append(&hw_box);
        let ad_switch = Switch::new(); ad_switch.set_active(state_clone_settings.borrow().enable_adblock);
        let ad_box = Box::new(Orientation::Horizontal, 10); ad_box.append(&ad_switch); ad_box.append(&Label::new(Some("AdBlock"))); vbox.append(&ad_box);
        let amnesia_switch = Switch::new(); amnesia_switch.set_active(state_clone_settings.borrow().amnesia_mode);
        let amnesia_box = Box::new(Orientation::Horizontal, 10); amnesia_box.append(&amnesia_switch); amnesia_box.append(&Label::new(Some("Amnesia Mode"))); vbox.append(&amnesia_box);
        let warn_lbl = Label::new(None); warn_lbl.set_markup("<i>(Changes require opening a new tab)</i>"); warn_lbl.set_halign(gtk::Align::Start); warn_lbl.set_sensitive(false); vbox.append(&warn_lbl);

        vbox.append(&Separator::new(Orientation::Horizontal));
        let label_about = Label::new(None); label_about.set_markup("<b>About</b>"); label_about.set_halign(gtk::Align::Start); vbox.append(&label_about);
        let about_btn = Button::builder().label("About Rusty Browser").build();
        about_btn.add_css_class("flat-button");
        vbox.append(&about_btn);
        vbox.append(&LinkButton::with_label("https://github.com/wyind/rusty_browser", "Source Code"));
        vbox.append(&LinkButton::with_label("https://ko-fi.com/wyind", "Donate"));

        scroll.set_child(Some(&vbox));
        content_area.append(&scroll);

        let window_weak_for_about = window_clone.downgrade();
        about_btn.connect_clicked(move |_| {
            if let Some(parent_window) = window_weak_for_about.upgrade() {
                let logo_bytes = include_bytes!("logo.png");
                let loader = PixbufLoader::new();
                loader.write(logo_bytes).unwrap();
                loader.close().unwrap();
                let pixbuf = loader.pixbuf().unwrap();
                let texture = gdk::Texture::for_pixbuf(&pixbuf);

                let about_window = Window::builder().transient_for(&parent_window).modal(true).title("About").default_width(350).default_height(400).resizable(false).decorated(true).build();
                let vbox = Box::new(Orientation::Vertical, 15);
                vbox.add_css_class("about-box");
                vbox.set_margin_top(20); vbox.set_margin_bottom(20); vbox.set_margin_start(20); vbox.set_margin_end(20);

                let img = Image::from_paintable(Some(&texture));
                img.set_pixel_size(128);
                vbox.append(&img);
                let title = Label::new(None); title.set_markup("<span size='xx-large' weight='bold'>Rusty Browser</span>"); vbox.append(&title);
                let version = Label::new(Some("Version 1.0.0 (Titan Edition)")); version.add_css_class("about-version"); vbox.append(&version);
                let desc = Label::new(Some("A privacy-focused, high-performance browser.\nBuilt with Rust, GTK4 & WebKit."));
                desc.set_wrap(true); desc.set_justify(gtk::Justification::Center); vbox.append(&desc);
                vbox.append(&Separator::new(Orientation::Horizontal));
                vbox.append(&LinkButton::with_label("https://wyind.dev", "Website"));
                vbox.append(&LinkButton::with_label("https://github.com/wyind/rusty_browser", "Source Code"));
                vbox.append(&LinkButton::with_label("https://ko-fi.com/wyind", "Buy Me A Coffee"));
                vbox.append(&Label::new(Some("© 2025 wyind.dev")));

                about_window.set_child(Some(&vbox));
                about_window.present();
            }
        });

        let state_clone = state_clone_settings.clone();
        let home_btn_action = home_btn_clone.clone();

        dialog.connect_response(move |d, _| {
            state_clone.borrow_mut().homepage = home_entry.text().to_string();
            state_clone.borrow_mut().use_hw_accel = hw_switch.is_active();
            state_clone.borrow_mut().enable_adblock = ad_switch.is_active();
            state_clone.borrow_mut().amnesia_mode = amnesia_switch.is_active();
            
            let show_home = show_home_switch.is_active();
            state_clone.borrow_mut().show_home_button = show_home;
            home_btn_action.set_visible(show_home);

            let idx = dropdown.selected();
            state_clone.borrow_mut().search_engine_index = idx;
            match idx {
                0 => state_clone.borrow_mut().search_engine_url = "https://duckduckgo.com/?q=".to_string(),
                1 => state_clone.borrow_mut().search_engine_url = "https://www.google.com/search?q=".to_string(),
                2 => state_clone.borrow_mut().search_engine_url = "https://www.bing.com/search?q=".to_string(),
                3 => state_clone.borrow_mut().search_engine_url = "https://search.brave.com/search?q=".to_string(),
                _ => {}
            }
            // SAVE CONFIG
            save_config(&state_clone.borrow());
            d.close();
        });
        dialog.show();
    });

    window.present();
}