#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), kristofersxyz::errors::ApplicationError> {
    use kristofersxyz::{
        admin_cli::{self, AdminCliError},
        configuration::Settings,
        startup::{App, Application},
        telemetry::{get_subscriber, init_subscriber},
    };
    use leptos::logging::log;

    dotenvy::dotenv().ok();

    let subscriber = get_subscriber("kristofersxyz", "info", std::io::stdout);
    init_subscriber(subscriber);

    let settings = Settings::from_env()?;

    // No subcommand means serve, which is how cargo-leptos runs this binary.
    // A subcommand runs the tool and exits.
    let mut arguments = std::env::args().skip(1);
    if let Some(command) = arguments.next() {
        return match command.as_str() {
            "set-password" => {
                let username = arguments.next().ok_or(AdminCliError::Usage)?;
                let password = admin_cli::read_new_password()?;
                admin_cli::set_password(&settings, &username, &password).await?;
                log!("password set for '{username}'");
                Ok(())
            }
            other => Err(AdminCliError::UnknownCommand(other.to_owned()).into()),
        };
    }

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
