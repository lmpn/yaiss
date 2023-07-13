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
        let path = std::env::var("INI_CONFIGURATION").unwrap();
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
            .unwrap(),
        );
        w.watch(path.as_ref(), RecursiveMode::NonRecursive).unwrap();
        Self {
            configuration: Ini::load_from_file(&path).unwrap(),
            watcher: rx,
            _w: w,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn database_url(&self) -> &str {
        self.configuration
            .get_from(Some("DATABASE"), "url")
            .unwrap()
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
            .unwrap()
            .parse::<u16>()
            .unwrap();
        (address.try_into().unwrap(), port)
    }

    pub async fn has_change(&mut self) -> Option<()> {
        loop {
            match self.watcher.next().await {
                Some(Ok(Event {
                    kind: notify::EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                    ..
                })) => return Some(()),
                _ => {}
            }
        }
    }
}
