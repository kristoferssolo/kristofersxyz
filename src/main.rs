#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), kristofersxyz::errors::ApplicationError> {
    use kristofersxyz::{
        configuration::Settings,
        startup::{App, Application},
        telemetry::{get_subscriber, init_subscriber},
    };
    use leptos::logging::log;

    dotenvy::dotenv().ok();

    let subscriber = get_subscriber("kristofersxyz", "info", std::io::stdout);
    init_subscriber(subscriber);

    let settings = Settings::from_env()?;

    let app = App::new(&settings).await?;

    let addr = app.leptos_options.site_addr;
    let application = Application::build(app).await?;
    log!("listening on http://{}", &addr);

    application.run_until_stopped().await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
