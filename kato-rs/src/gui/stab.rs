/// Module contains function to create and manipulate server tab
use fltk::{
    button::Button,
    enums::Align,
    frame::Frame,
    group::{Flex, Group, Pack, Tabs},
    prelude::{GroupExt, WidgetBase, WidgetExt},
};

use tracing::info;

use crate::gui::{Message, Metadata};

use super::Kui;

/// Implies server tab functions
impl Kui {
    fn resize_group(widget: &mut Group, x: i32, y: i32, w: i32, h: i32) {
        let children = widget.children();
        for idx in 0..children {
            if let Some(mut child) = widget.child(idx) {
                child.resize(
                    x + Kui::MARGIN,
                    y + Kui::TAB_HEIGHT,
                    w - x - Kui::MARGIN,
                    h - Kui::TAB_HEIGHT - y,
                );
            }
        }
    }

    /// Server tab creation
    pub(crate) fn create_server_tab(&mut self, server_name: &str) -> Option<Group> {
        if let Some(mt) = &mut self.main_tab.clone() {
            let server_group = Group::default()
                .with_size(200, 200)
                .with_label(format!("Server: {}", server_name).as_str());
            let mut tab = Tabs::new(0, Kui::MARGIN, 500, 450, "");
            let mut info_tab = Group::default()
                .with_size(200, 200)
                .with_label("Information");
            let _ = self.info_tab(server_name);
            info_tab.end();

            let grp2 = Group::new(0, Kui::TAB_HEIGHT, 500, 450 - Kui::TAB_HEIGHT, "Topics");
            grp2.end();

            tab.end();
            server_group.end();

            tab.resize_callback(Kui::tab_resize_callback);
            info_tab.resize_callback(Kui::resize_group);

            mt.add(&server_group);
            let _ = mt.set_value(&server_group);
            mt.resize(mt.x(), mt.y(), mt.w(), mt.h());
            mt.redraw();
            Some(server_group)
        } else {
            None
        }
    }

    fn key_value_label(key: &str, value: &str, label_width: i32) {
        let mut flex = Flex::default_fill().with_size(100, 30);
        let key_frame = Frame::default()
            .with_label(key)
            .with_align(Align::Inside | Align::Left);
        let _ = Frame::default()
            .with_label(value)
            .with_align(Align::Inside | Align::Left);
        flex.fixed(&key_frame, label_width);
        flex.end();
    }

    /// Infomation tab
    fn info_tab(&mut self, server_name: &str) -> Pack {
        let mut pack = Pack::default().with_size(100, 100);
        pack.set_spacing(10);

        if let Some(v) = self.servers.get(server_name) {
            Kui::key_value_label("Bootstrap:", v.bootstrap.join(",").as_str(), 80);
            Kui::key_value_label(
                "Protocol:",
                match v.protocol {
                    1 => "SSL",
                    0 => "PLAINTEXT",
                    _ => "UNKNOWN",
                },
                80,
            );
        }

        let mut flex = Flex::default_fill().with_size(100, 30);
        let mut connect = Button::default().with_size(0, 30).with_label("Connect");
        let mut disconnect = Button::default().with_size(0, 30).with_label("Disconnect");
        flex.fixed(&connect, 80);
        flex.fixed(&disconnect, 100);
        flex.end();

        if let Some(snd) = &self.sender {
            let sname = server_name.to_string();
            let snd = *snd;
            connect.set_callback(move |_| {
                let name = sname.to_string();
                snd.send(super::Message::Connect(name));
            });
        }

        if let Ok(bp) = self.broker_pools.read()
            && bp.contains_key(server_name)
        {
            connect.deactivate();
            disconnect.activate();
        }

        pack.end();
        pack
    }

    pub(crate) fn connect(&mut self, server_name: &str) {
        if let Some(snd) = self.sender {
            let sname = server_name.to_string();
            let broker_pools = self.broker_pools.clone();
            tokio::spawn(async move {
                let si = broker_pools
                    .write()
                    .ok()
                    .and_then(|mut bp| bp.remove(&sname));

                if let Some(mut si) = si {
                    match si.pool.init().await {
                        Ok(_) => {
                            info!("Successfully connected");
                            if let Some(mdr) = si.pool.metadata() {
                                let brokers = mdr
                                    .brokers
                                    .iter()
                                    .map(|bmd| format!("{}:{}", bmd.host, bmd.port))
                                    .collect();
                                let topics = mdr
                                    .topics
                                    .iter()
                                    .map(|topic| {
                                        topic.name.as_ref().map_or(String::new(), |n| n.to_string())
                                    })
                                    .collect();
                                snd.send(Message::FillServersTree(Metadata {
                                    server_name: sname.clone(),
                                    brokers,
                                    topics,
                                }));
                            }

                            if let Ok(mut bp) = broker_pools.write() {
                                bp.insert(sname, si);
                            }
                        }
                        Err(err) => {
                            snd.send(Message::ShowAlert(format!("Error connecting: {:?}", err)));
                        }
                    }
                }
            });

            // let _ = self.servers.get(server_name).and_then(|sc| {
            //     KafkaConfig::ctor(sc.bootstrap.clone(), sc.protocol, sc.verify_certs.clone(), sc.cert.clone(), sc.key.clone(), sc.ca_certs.clone()).ok()
            // }).and_then(|kc| {
            //     Some(BrokerPool::new(Arc::new(kc), Duration::from_secs(30), "kafka-tool"))
            // }).map(|mut broker_pool| {
            //     let broker_pools = self.broker_pools.clone();
            //     tokio::spawn(async move {
            //         match broker_pool.init().await {
            //             Ok(_) => {
            //                 info!("Successfully connected");
            //                 if let Some(mdr) = broker_pool.metadata() {
            //                     let brokers = mdr
            //                         .brokers
            //                         .iter()
            //                         .map(|(_, bmd)| {
            //                             format!("{}:{}", bmd.host, bmd.port)
            //                         })
            //                         .collect();
            //                     let topics = mdr
            //                         .topics
            //                         .iter()
            //                         .map(|(topic, _)| topic.to_string())
            //                         .collect();
            //                     snd.send(Message::FillServersTree(
            //                         Metadata {
            //                             server_name: sname.clone(),
            //                             brokers,
            //                             topics,
            //                         },
            //                     ));
            //                 }

            //                 if let Ok(mut bp) = broker_pools.write() {
            //                     if let Some(si) = bp.get_mut(&sname) {
            //                         info!("Already added broker pool in pools: {:?}", &si);
            //                         // bp.
            //                         // bp.insert(sname, ServerInfo {
            //                         //     pool: broker_pool,
            //                         //     tab_group: si.tab_group,
            //                         // });
            //                     }
            //                 }
            //             }
            //             Err(err) => {
            //                 snd.send(Message::ShowAlert(format!(
            //                     "Error connecting: {:?}",
            //                     err
            //                 )));
            //             }
            //         }
            //     });
            // });
        }
    }
}
