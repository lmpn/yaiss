use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver},
    SinkExt, StreamExt,
};
use ini::Ini;
use notify::{
    event::{DataChange, ModifyKind},
    Config, Event, RecommendedWatcher, RecursiveMode, Watcher,
};

pub struct Configuration {
    configuration: ini::Ini,
    watcher: UnboundedReceiver<notify::Result<Event>>,
    _w: Box<dyn Watcher>,
}

impl Configuration {
    pub fn new() -> Self {
        let path =
            std::env::var("INI_CONFIGURATION").expect("env variable INI_CONFIGURATION not set");
        let (mut tx, rx) = unbounded();

        let mut w: Box<dyn Watcher> = Box::new(
            RecommendedWatcher::new(
                move |res| {
                    futures::executor::block_on(async {
                        tx.send(res).await.unwrap();
                    })
                },
                Config::default(),
            )
            .expect("Error creating watcher"),
        );
        w.watch(path.as_ref(), RecursiveMode::NonRecursive)
            .expect("Error starting watcher");
        Self {
            configuration: Ini::load_from_file(&path).expect("Error loading configuration file"),
            watcher: rx,
            _w: w,
        }
    }

    pub(crate) fn database_url(&self) -> &str {
        self.configuration
            .get_from(Some("DATABASE"), "url")
            .expect("Invalid url")
    }

    pub(crate) fn migrations_path(&self) -> &str {
        self.configuration
            .get_from(Some("DATABASE"), "migrations_path")
            .expect("Invalid migrations path")
    }

    pub(crate) fn address(&self) -> ([u8; 4], u16) {
        let address: Vec<u8> = self
            .configuration
            .get_from(Some("SERVER"), "address")
            .unwrap()
            .split('.')
            .map(|e| e.parse::<u8>().unwrap())
            .collect();
        let port = self
            .configuration
            .get_from(Some("SERVER"), "port")
            .expect("Invalid port value")
            .parse::<u16>()
            .expect("Invalid port value");
        (address.try_into().expect("Invalid address value"), port)
    }

    pub async fn has_change(&mut self) -> Option<()> {
        loop {
            if let Some(Ok(Event {
                kind: notify::EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                ..
            })) = self.watcher.next().await
            {
                return Some(());
            }
        }
    }

    pub(crate) fn images_base_path(&self) -> &str {
        self.configuration
            .get_from(Some("IMAGE_SERVICE"), "base_path")
            .expect("Invalid base path")
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}
