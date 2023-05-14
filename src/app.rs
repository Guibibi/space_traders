use dotenv::dotenv;
use space_traders_api::{apis::configuration::Configuration, models::Agent};
use std::{env, sync::mpsc::{self, Sender}};
use tokio::runtime;

const PPP: f32 = 1.25;

enum Messages {
    Agent(Box<Agent>),
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state

pub struct TemplateApp {
    #[serde(skip)]
    client: Configuration,

    #[serde(skip)]
    rt: runtime::Runtime,

    #[serde(skip)]
    sender: mpsc::Sender<Messages>, 

    #[serde(skip)]
    receiver: mpsc::Receiver<Messages>,

    #[serde(skip)]
    agent: Option<Box<Agent>>,
}



impl Default for TemplateApp {
    fn default() -> Self {
        dotenv().ok(); // Loads the .env file
        let user_token = env::var("ACCOUNT_TOKEN").expect("ACCOUNT_TOKEN environement variable not set.");
        let mut client: space_traders_api::apis::configuration::Configuration = space_traders_api::apis::configuration::Configuration::new();
        client.bearer_access_token = Option::Some(user_token);

        let (sender, receiver) = mpsc::channel();
        Self {
            client,
            rt: runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap(),
            sender,
            receiver,
            agent: None,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        cc.egui_ctx.set_pixels_per_point(PPP);
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let Self { client, rt, sender, receiver, agent} = self;
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        match receiver.try_recv() {
            Ok(Messages::Agent(new_agent)) => {
                *agent = Some(new_agent);
            },
            Err(_) => {},
        }

        //NOTE: Example for a menu bar
        // #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     // The top panel is often a good place for a menu bar:
        //     egui::menu::bar(ui, |ui| {
        //         ui.menu_button("File", |ui| {
        //             if ui.button("Quit").clicked() {
        //                 _frame.close();
        //             }
        //         });
        //     });
        // });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Agent Information");

            // TODO: Reactivate this
            // if ui.button("Get agent").clicked() {
            //     let new_client = client.clone();
            //     rt.spawn(get_my_agent(new_client, sender.clone()));                
            // }

            if ui.button("Get Agent").clicked() {
                let new_client = client.clone();
                let new_sender: Sender<Messages> = sender.clone();
                rt.spawn(async move {
                    //let request = space_traders_api::apis::systems_api::get_waypoint(configuration, system_symbol, waypoint_symbol)
                    //let request = space_traders_api::apis::locations_api::get_locations(&new_client);
                    let request = space_traders_api::apis::agents_api::get_my_agent(&new_client);
                    let response = request.await;
                    println!("{:?}", response);
                    let new_agent = response.unwrap();
                    match new_sender.send(Messages::Agent(new_agent.data)) {
                        Ok(_) => println!("Sent"),
                        Err(_) => {},
                    }
                });
            }

            ui.vertical(|ui| {
                ui.heading("Agent");
                match agent {
                    Some(agent) => {
                        ui.label(format!("Username: {}", agent.symbol));
                        ui.label(format!("Credits: {}", agent.credits));
                        ui.label(format!("Headquarters: {}", agent.headquarters));
                    },
                    None => {
                        ui.label("Please get your agent.");
                    }
                }
            });

            // Layout example:
            // ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            //     ui.horizontal(|ui| {
            //         ui.spacing_mut().item_spacing.x = 0.0;
            //         ui.label("powered by ");
            //         ui.hyperlink_to("egui", "https://github.com/emilk/egui");
            //         ui.label(" and ");
            //         ui.hyperlink_to(
            //             "eframe",
            //             "https://github.com/emilk/egui/tree/master/crates/eframe",
            //         );
            //         ui.label(".");
            //     });
            // });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.heading("eframe template");
            // TODO: Fix this deferencing error.
            ui.hyperlink("https://github.com/emilk/eframe_template");
            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/master/",
                "Source code."
            ));
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }
    }
}
