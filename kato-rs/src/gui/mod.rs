mod stab;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use fltk::{
    app::{self, Sender},
    button::{Button, CheckButton},
    dialog,
    enums::{self, Align, Event, Font},
    frame::Frame,
    group::{Flex, Group, Pack, Tabs},
    image::JpegImage,
    input::Input,
    menu::Choice,
    output::Output,
    prelude::*,
    tree::{Tree, TreeItemDrawMode, TreeReason},
    window::{DoubleWindow, Window},
};
use kafka_lib::{BrokerPool, KafkaConfig};
use tracing::{info, warn};

use crate::config::{self, ServerConfig, WindowPreference};

macro_rules! clones {
    ($($field:expr),*) => {
        ($(
            $field.clone(),
        )*)
    };
}

/// Events to talk between callbacks and async functions
#[derive(Debug)]
pub(crate) enum Message {
    /// Message to save main window preferences. Contains coordinates x, y and sizes - width and heigth
    SaveWindowPreferences(i32, i32, i32, i32),
    /// Message to show error dialog
    ShowError(String),
    /// Message to show info/alert dialog
    ShowAlert(String),
    /// Add server to configuration database and tree control
    AddServer(String, ServerConfig),
    /// Add server tab 
    AddServerTab(String),
    /// Add server information to tree control
    FillServersTree(Metadata),
    /// Connect to server
    Connect(String),
}

#[derive(Debug, Clone)]
pub(crate) struct Metadata {
    pub(crate) server_name: String,
    pub(crate) brokers: Vec<String>,
    pub(crate) topics: Vec<String>,
}

#[derive(Debug)]
pub(crate) struct ServerInfo {
    pub(crate) pool: BrokerPool,
    pub(crate) tab_group: Group,
}

/// Gui configuration and states
pub struct KUI {
    // Configuration
    pub(crate) config: config::Config,
    pub(crate) servers: HashMap<String, ServerConfig>,
    pub(crate) tree: Option<Tree>,
    pub(crate) main_tab: Option<Tabs>,
    pub(crate) broker_pools: Arc<RwLock<HashMap<String, ServerInfo>>>,
    pub(crate) sender: Option<Sender<Message>>,
}

impl KUI {
    const WINDOW_NAME: &str = "KUI";
    const TAB_HEIGHT: i32 = 30;
    const MARGIN: i32 = 3;

    /// Create new KUI object with default values
    pub fn new(config: config::Config) -> Self {
        Self {
            config,
            servers: Default::default(),
            tree: None,
            main_tab: None,
            broker_pools: Default::default(),
            sender: None,
        }
    }

    /// Loads all kafka server onfigurations from database and fills server's tree interface component
    fn load_servers(&mut self, tree: &mut Tree) {
        tree.clear();
        self.config
            .read_servers()
            .into_iter()
            .for_each(|(name, server)| {
                info!("Load server: {}, {:?}", name, server);
                self.servers.insert(name.clone(), server);
                let item = tree.add(&name);
                if let Some(mut item) = item {
                    // item.set_label_color(Color::from_rgb(255, 0, 0));
                    let image = include_bytes!("images/red_circle.jpeg");
                    if let Some(mut img) = JpegImage::from_data(image).ok() {
                        img.scale(16, 16, true, false);
                        item.set_user_icon(Some(img));
                    }
                    item.set_label_size(16);
                }
                tree.redraw();
            });
    }

    /// Loads window preferences from database and move/resize main window
    fn load_window_preferences(&self, wnd: &mut DoubleWindow) {
        match self.config.find_window_preference(KUI::WINDOW_NAME) {
            Ok(wp) => {
                wnd.set_size(wp.width, wp.height);
                wnd.set_pos(wp.x, wp.y);
            }
            Err(err) => warn!("Error loading window preferences: {}", err),
        }
    }

    /// Save window preferences into configuration database
    fn save_window_preference(&self, pref: WindowPreference) -> anyhow::Result<()> {
        self.config.add_window_preference(KUI::WINDOW_NAME, &pref)
    }

    /// Main UI loop. This function processes all messages from UI callbacks or from asynchronous functions.
    /// All the messages sending and receiving using crossbeam channel created inside fltk library.
    /// Access to channel allowed by calling [`fltk::app::channel`](fn@fltk::app::channel)
    /// Messages defined in [`Message`](enum@crate::Message)
    pub fn run(&mut self) -> anyhow::Result<()> {
        let app = app::App::default().with_scheme(app::Scheme::Base);
        let (sender, receiver) = app::channel::<Message>();
        self.sender = Some(sender.clone());
        let mut wind = self.make_window(sender.clone());

        // Process messages
        while app.wait() {
            match receiver.recv() {
                // Store window preferences and exit
                Some(Message::SaveWindowPreferences(x, y, w, h)) => {
                    warn!("Receive save preference message");
                    match self.save_window_preference(WindowPreference {
                        x,
                        y,
                        width: w,
                        height: h,
                    }) {
                        Ok(_) => info!(
                            "Successfully stored window preferences: {}/{}/{}/{}",
                            x, y, w, h
                        ),
                        Err(err) => warn!("Error storing window preferences: {:?}", err),
                    }
                    // Close main window and exit program
                    wind.hide();
                }
                // Show information dialog
                Some(Message::ShowAlert(alert)) => dialog::message_default(&alert),
                // Show error dialog
                Some(Message::ShowError(error)) => dialog::message_default(&error),
                // Add server to configuration database and redraw tree control
                Some(Message::AddServer(name, server)) => {
                    match self.config.add_server(name, server) {
                        Ok(_) => {
                            dialog::message_default("Server added");
                            if self.tree.is_some() {
                                let mut tree = self.tree.clone().unwrap();
                                self.load_servers(&mut tree);
                            }
                        }
                        Err(err) => dialog::message_default(
                            format!("Error adding server: {}", err).as_str(),
                        ),
                    }
                }
                // Fill server tree node with brokers and topics children
                Some(Message::FillServersTree(md)) => {
                    if let Some(tree) = &mut self.tree.clone() {
                        tree.add(format!("{}/brokers", &md.server_name).as_str());
                        tree.add(format!("{}/topics", &md.server_name).as_str());

                        md.brokers.iter().for_each(|b| {
                            tree.add(format!("{}/brokers/{}", &md.server_name, b).as_str());
                        });
                        md.topics.iter().for_each(|b| {
                            tree.add(format!("{}/topics/{}", &md.server_name, b).as_str());
                        });

                        tree.redraw();
                    }
                }
                // Create or activate server's tab with information of brokers and tab control
                Some(Message::AddServerTab(server_name)) => {
                    // Check if tab and broker pool already created
                    if let Some(group) = self
                        .broker_pools
                        .read()
                        .ok()
                        .and_then(|bp| bp.get(&server_name).map(|v| v.tab_group.clone()))
                    {
                        self.main_tab.iter_mut().for_each(|mt| {
                            // Activate current server's tab
                            let _ = mt.set_value(&group);
                        });
                    } else if let Some(server_group) = self.create_server_tab(&server_name) {
                        // New server's tab created
                        if let Some(server) = self.servers.get(&server_name) {
                            match KafkaConfig::ctor(
                                server.bootstrap.clone(),
                                server.protocol,
                                server.verify_certs,
                                server.cert.clone(),
                                server.key.clone(),
                                server.ca_certs.clone(),
                            ) {
                                Ok(cfg) => {
                                    info!("Successfully create config");
                                    let broker_pool = BrokerPool::new(
                                        Arc::new(cfg),
                                        Duration::from_secs(30),
                                        "kafka-tool",
                                    );

                                    if let Ok(mut bp) = self.broker_pools.write() {
                                        bp.insert(
                                            server_name,
                                            ServerInfo {
                                                pool: broker_pool,
                                                tab_group: server_group,
                                            },
                                        );
                                    }
                                }

                                Err(err) => dialog::message_default(
                                    format!("Error creating kafka config: {:?}", err).as_str(),
                                ),
                            }
                        } else {
                            dialog::message_default(
                                format!("Server {} not found", &server_name).as_str(),
                            );
                        }
                    }
                }
                Some(Message::Connect(server)) => {
                    self.connect(&server);
                }

                // Unknown message
                None => {}
            }
        }

        
        // Save preference after main loop has been interrupted
        match receiver.recv() {
            // Store window preferences and exit
            Some(Message::SaveWindowPreferences(x, y, w, h)) => {
                warn!("Receive save preference message");
                match self.save_window_preference(WindowPreference {
                    x,
                    y,
                    width: w,
                    height: h,
                }) {
                    Ok(_) => info!(
                        "Successfully stored window preferences: {}/{}/{}/{}",
                        x, y, w, h
                    ),
                    Err(err) => warn!("Error storing window preferences: {:?}", err),
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Function handle tree selection event and send message to channel depends on tree item position
    fn handle_tree_selection(&mut self, sender: Sender<Message>) {
        if let Some(mut tree) = self.tree.clone() {
            tree.set_callback_reason(TreeReason::Selected);
            tree.set_callback(move |t| {
                if let Some(items) = t.get_selected_items() {
                    // Only one item must be selectd
                    if let Some(item) = items.first() {
                        let path = item.label().unwrap_or_default();
                        if item.parent().map(|p| p.is_root()).unwrap_or(true) {
                            // Add closable server tab
                            sender.send(Message::AddServerTab(path));
                        } else if path.contains("/brokers/") {
                            println!("Select broker {}", &path);
                        }
                    }
                }
            });
        }
    }

    /// Make main window. All callbacks using 'static lifetime, therefore we are using channels to send messages
    /// from callback to main loop
    fn make_window(&mut self, sender: Sender<Message>) -> DoubleWindow {
        let mut wind = Window::default()
            .with_size(1024, 800)
            .center_screen()
            .with_label("Kafka tool rust edition");

        self.load_window_preferences(&mut wind);
        wind.make_resizable(true);
        let snd = sender.clone();
        wind.handle(move |wnd, ev| {
            if ev.bits() == Event::Close.bits() || ev.bits() == Event::Hide.bits() {
                info!("Handle window close event");
                snd.send(Message::SaveWindowPreferences(
                    wnd.x(),
                    wnd.y(),
                    wnd.w(),
                    wnd.h(),
                ));
                true
            } else {
                false
            }
        });

        // Flex layout
        let mut rows = Flex::default_fill().row();

        let mut tree = Tree::default().with_align(enums::Align::Inside | enums::Align::Right);
        self.tree = Some(tree.clone());
        self.handle_tree_selection(sender.clone());

        tree.set_show_root(false);
        tree.set_connector_style(fltk::tree::TreeConnectorStyle::Dotted);
        tree.set_item_draw_mode(TreeItemDrawMode::LabelAndWidget);

        rows.set_size(&tree, 300);

        let mut tab = Tabs::new(0, 0, 500, 450, "");
        self.main_tab = Some(tab.clone());

        let grp2 = Group::default_fill().with_label("Welcome");

        KUI::draw_welcome_page(&sender);

        grp2.end();

        tab.resize_callback(KUI::tab_resize_callback);

        tab.end();

        self.load_servers(&mut tree);

        rows.end();

        wind.resizable(&rows);
        wind.end();
        wind.show();
        wind
    }

    pub(crate) fn tab_resize_callback(widget: &mut impl GroupExt, x: i32, y: i32, w: i32, h: i32) {
        let children = widget.children();
        for idx in 0..children {
            if let Some(mut child) = widget.child(idx) {
                child.resize(x + KUI::MARGIN, y + KUI::TAB_HEIGHT, w - x - KUI::MARGIN, h - KUI::TAB_HEIGHT);
            }
        }
    }

    /// Create file browser widget with browse button
    fn browse_widget(width: i32, title: &str, pattern: &str, activated: bool) -> Output {
        let mut flex = Flex::default().with_size(width, KUI::TAB_HEIGHT);
        let mut file_pem = Output::default()
            .with_size(0, 30)
            .with_label(title)
            .with_align(Align::Left);
        if !activated {
            file_pem.deactivate();
        }

        let mut browse_button = Button::default()
            .with_size(100, KUI::TAB_HEIGHT)
            .with_label("@fileopen");
        flex.set_size(&browse_button, 30);
        flex.end();

        let title_s = title.to_string();
        let pattern_s = pattern.to_string();
        let mut fpem = file_pem.clone();
        browse_button.set_callback(move |_| {
            let file = dialog::file_chooser(title_s.as_str(), pattern_s.as_str(), ".", true);
            if let Some(filename) = file {
                fpem.set_value(filename.as_str());
            }
        });

        file_pem
    }

    /// Draw welcome page. Page used to add new kafka server connection
    fn draw_welcome_page(sender: &Sender<Message>) {
        let mut pack = Pack::default().with_size(300, 200).center_of_parent();
        pack.set_spacing(10);

        let mut _fmt =
            Frame::new(0, 0, 0, 0, "Add new kafka cluster").with_align(Align::Inside | Align::Left);
        _fmt.set_label_size(24);
        _fmt.set_label_font(Font::Courier);

        let mut name = Input::default()
            .with_size(0, KUI::TAB_HEIGHT)
            .with_label("Server name:")
            .with_align(Align::Left);
        let mut bootstrap = Input::default()
            .with_size(0, KUI::TAB_HEIGHT)
            .with_label("Bootstrap:")
            .with_align(Align::Left);
        let mut protocol = Choice::default()
            .with_size(0, KUI::TAB_HEIGHT)
            .with_label("Protocol:")
            .with_align(Align::Left);
        protocol.add_choice("plaintext");
        protocol.add_choice("ssl");

        let cert = KUI::browse_widget(100, "Certificate (PEM):", "PEM Files (*.pem)", false);
        let key = KUI::browse_widget(100, "Key (cer):", "Key Files (*.cer)", false);
        let ca_cert = KUI::browse_widget(100, "CA certs (PEM):", "PEM Files (*.pem)", false);

        let mut check_certs = CheckButton::default()
            .with_size(0, 30)
            .with_label("Validate server certificate");
        check_certs.deactivate();

        let mut buttons = Flex::new(200, 0, 100, KUI::TAB_HEIGHT, "");
        let space = Frame::new(0, 0, 200, 0, "");
        let mut test_button = Button::default()
            .with_size(100, KUI::TAB_HEIGHT)
            .with_label("Test connect");
        test_button.deactivate();
        let mut save_button = Button::default()
            .with_size(100, KUI::TAB_HEIGHT)
            .with_label("Save");
        save_button.deactivate();

        buttons.set_size(&space, 200);
        buttons.end();

        // Callback to enable/disable buttons
        let (c_name, c_bootstrap, c_protocol) = clones!(name, bootstrap, protocol);
        let (mut c_test_button, mut c_save_button) = clones!(test_button, save_button);
        name.set_callback(move |_| {
            if !c_name.value().is_empty()
                && !c_bootstrap.value().is_empty()
                && c_protocol.value() >= 0
            {
                c_test_button.activate();
                c_save_button.activate();
            } else {
                c_test_button.deactivate();
                c_save_button.deactivate();
            }
        });

        let (c_name, c_bootstrap, c_protocol) = clones!(name, bootstrap, protocol);
        let (mut c_test_button, mut c_save_button) = clones!(test_button, save_button);
        bootstrap.set_callback(move |_| {
            if !c_name.value().is_empty()
                && !c_bootstrap.value().is_empty()
                && c_protocol.value() >= 0
            {
                c_test_button.activate();
                c_save_button.activate();
            } else {
                c_test_button.deactivate();
                c_save_button.deactivate();
            }
        });

        let (c_name, c_bootstrap, c_protocol) = clones!(name, bootstrap, protocol);
        let (mut c_test_button, mut c_save_button, mut c_cert, mut c_key, mut c_ca_cert) =
            clones!(test_button, save_button, cert, key, ca_cert);
        protocol.set_callback(move |_| {
            if !c_name.value().is_empty()
                && !c_bootstrap.value().is_empty()
                && c_protocol.value() >= 0
            {
                c_test_button.activate();
                c_save_button.activate();
            } else {
                c_test_button.deactivate();
                c_save_button.deactivate();
            }

            if c_protocol.value() == 1 {
                c_cert.activate();
                c_key.activate();
                c_ca_cert.activate();
            } else {
                c_cert.deactivate();
                c_key.deactivate();
                c_ca_cert.deactivate();
            }
        });

        let (c_bootstrap, c_protocol, c_check_certs, c_cert, c_key, c_ca_cert) =
            clones!(bootstrap, protocol, check_certs, cert, key, ca_cert);
        let snd = sender.clone();
        test_button.set_callback(move |_| {
            let bs = c_bootstrap.value();
            let proto = match c_protocol.value() {
                1 => "ssl".to_string(),
                _ => "plaintext".to_string(),
            };
            let varify_certs = c_check_certs.value();
            let certificate = PathBuf::from(&c_cert.value())
                .canonicalize()
                .ok()
                .and_then(|p| p.into_os_string().into_string().ok());
            let key = PathBuf::from(&c_key.value())
                .canonicalize()
                .ok()
                .and_then(|p| p.into_os_string().into_string().ok());
            let ca = PathBuf::from(&c_ca_cert.value())
                .canonicalize()
                .ok()
                .and_then(|p| p.into_os_string().into_string().ok());
            match KafkaConfig::new(bs, proto, varify_certs, certificate, key, ca) {
                Ok(cfg) => {
                    let snd = snd.clone();
                    tokio::spawn(async move {
                        match cfg.connect().await {
                            Ok(_) => {
                                snd.send(Message::ShowAlert("Successfully connected".to_string()))
                            }
                            Err(err) => {
                                snd.send(Message::ShowError(format!("Error connecting: {:?}", err)))
                            }
                        }
                    });
                }
                Err(err) => {
                    dialog::message_default(format!("Error connecting: {:?}", err).as_str())
                }
            }
        });

        let (c_name, c_bootstrap, c_protocol, c_check_certs, c_cert, c_key, c_ca_cert) =
            clones!(name, bootstrap, protocol, check_certs, cert, key, ca_cert);
        let snd = sender.clone();
        save_button.set_callback(move |_| {
            let server_name = c_name.value();
            let bs = c_bootstrap.value();
            let proto = c_protocol.value();
            let verify_certs = c_check_certs.value();
            let certificate = PathBuf::from(&c_cert.value())
                .canonicalize()
                .ok()
                .and_then(|p| p.into_os_string().into_string().ok());
            let key = PathBuf::from(&c_key.value())
                .canonicalize()
                .ok()
                .and_then(|p| p.into_os_string().into_string().ok());
            let ca = PathBuf::from(&c_ca_cert.value())
                .canonicalize()
                .ok()
                .and_then(|p| p.into_os_string().into_string().ok());
            let cfg = ServerConfig {
                bootstrap: bs.split(',').map(|s| s.to_string()).collect(),
                cert: certificate,
                key,
                ca_certs: ca,
                protocol: proto as i16,
                verify_certs,
            };
            snd.send(Message::AddServer(server_name, cfg));
        });

        pack.end();
    }
}
