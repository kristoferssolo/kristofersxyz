#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> Result<(), kristofersxyz::errors::ApplicationError> {
    use kristofersxyz::{
        admin_cli::{self, AdminCliError},
        configuration::Settings,
        domain::Username,
        startup::{Application, ApplicationState},
        telemetry::{get_subscriber, init_subscriber},
    };
    use leptos::logging::log;
    use std::path::Path;

    dotenvy::dotenv().ok();

    let subscriber = get_subscriber("info", std::io::stdout);
    init_subscriber(subscriber);

    let settings = Settings::from_env()?;

    let mut arguments = std::env::args().skip(1);
    if let Some(command) = arguments.next() {
        return match command.as_str() {
            "set-password" => {
                let username = Username::new(arguments.next().ok_or(AdminCliError::Usage)?)
                    .map_err(AdminCliError::from)?;
                let password = admin_cli::read_new_password()?;
                admin_cli::set_password(&settings, &username, &password).await?;
                log!("password set for '{username}'");
                Ok(())
            }
            "backup" => {
                let destination = arguments.next().ok_or(AdminCliError::Usage)?;
                let destination = Path::new(&destination);
                admin_cli::back_up(&settings, destination).await?;
                log!("backup written to '{}'", destination.display());
                Ok(())
            }
            "verify-restore" => {
                let database = arguments.next().ok_or(AdminCliError::Usage)?;
                let database = Path::new(&database);
                let report = admin_cli::verify_restore(database).await?;
                log!(
                    "'{}' passed its integrity check, {} session(s) revoked",
                    database.display(),
                    report.revoked_sessions
                );
                Ok(())
            }
            other => Err(AdminCliError::UnknownCommand(other.to_owned()).into()),
        };
    }

    let app = ApplicationState::new(&settings).await?;

    let addr = app.leptos_options.site_addr;
    let application = Application::build(app).await?;
    log!("listening on http://{}", &addr);

    application.run_until_stopped().await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
