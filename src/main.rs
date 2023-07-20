// NOTE: Bacause I don't have time to learn about dependency injection yet
// that why I decided use shaku
use shaku::{module, Component, HasComponent, Interface};
use std::{
    io,
    sync::{Arc, Mutex},
};

trait IApp: Interface {
    fn set_title(&self, title: &str);
    fn get_title(&self) -> String;
}

#[derive(Component)]
#[shaku(interface = IApp)]
struct AppImpl {
    title: Mutex<AppData>,
}

struct AppData {
    title: String,
}

impl Default for AppImpl {
    fn default() -> Self {
        AppImpl {
            title: Mutex::new(AppData {
                title: String::from("default"),
            }),
        }
    }
}

impl IApp for AppImpl {
    fn set_title(&self, _title: &str) {
        let mut app_data = self.title.lock().unwrap();
        app_data.title = String::from(_title);
    }
    fn get_title(&self) -> String {
        self.title.lock().unwrap().title.clone()
    }
}

// NOTE: Just imagine this is some kind of sql client
trait IClient: Interface {
    fn raw(&self) -> usize;
}

#[derive(Component)]
#[shaku(interface = IClient)]
struct ClientImpl {
    size: usize,
}

impl IClient for ClientImpl {
    fn raw(&self) -> usize {
        self.size
    }
}

impl Default for ClientImpl {
    fn default() -> Self {
        ClientImpl { size: 10 }
    }
}

module! {
    AppModule {
        components = [AppImpl, ClientImpl],
        providers  = []
    }
}

type App = Arc<AppModule>;

struct Server {
    func: Vec<Box<dyn FnMut(App)>>,
    module: Option<App>,
}

impl Server {
    fn new() -> Self {
        Server {
            func: vec![],
            module: None,
        }
    }
    fn add_system(&mut self, func: impl FnMut(App) + 'static) -> &mut Self {
        self.func.push(Box::new(func));
        self
    }
    fn add_module(&mut self, module: App) -> &mut Self {
        self.module = Some(module);
        self
    }
    fn run(&mut self) -> io::Result<()> {
        let module = self.module.as_mut().unwrap();
        for system in self.func.iter_mut() {
            system(module.clone());
        }
        Ok(())
    }
}

fn main() {
    let module = Arc::new(
        AppModule::builder()
            .with_component_override::<dyn IApp>(Box::new(AppImpl::default()))
            .with_component_override::<dyn IClient>(Box::new(ClientImpl::default()))
            .build(),
    );

    Server::new()
        .add_module(module.clone())
        .add_system(func_a)
        .add_system(func_b)
        .add_system(func_c)
        .run()
        .unwrap();
}

fn func_a(module: App) {
    let app: &dyn IApp = module.resolve_ref();

    let title = app.get_title();
    println!("{title}");
    app.set_title("hello world");
}

fn func_b(module: App) {
    let app: &dyn IApp = module.resolve_ref();

    let title = app.get_title();
    println!("{title}");
}

fn func_c(module: App) {
    let client: &dyn IClient = module.resolve_ref();

    let size = client.raw();
    println!("{size}");
}
