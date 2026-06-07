use tap::messages::MessageParse;

async fn validate(cli: &mut tap::cli::Cli) {
    match &cli.stage {
        tap::cli::CliStage::EnteringAddress => {
            cli.player.client.addr = cli.state.consume();
            cli.state.log(tap::cli::Message::Info(format!(
                "attempting to connect to '{}'",
                cli.player.client.addr,
            )));
            cli.stage = tap::cli::CliStage::WaitingConnection;
            cli.waiter.begin();
        }
        tap::cli::CliStage::EnteringUsername => {
            cli.player.username = cli.state.consume();
            cli.state.log(tap::cli::Message::Info(format!(
                "try to authenticate with username '{}'",
                cli.player.username,
            )));
            match cli.player.client.write_message(&tap::messages::Message::Command(tap::messages::Command {
                kind: tap::messages::CommandKind::Connect,
                payload: tap::messages::Payload::new(&[
                    tap::messages::PayloadKind::String(cli.player.username.clone()),
                ]),
            })).await {
                Ok(_) => {
                    cli.stage = tap::cli::CliStage::WaitingAuth;
                    cli.waiter.begin();
                },
                Err(e) => cli.state.log(tap::cli::Message::error(e)),
            };
        }
        tap::cli::CliStage::EnteringCommand => {
            let input = cli.state.consume();
            cli.state.log(tap::cli::Message::Outgoing(input.clone()));
            match tap::messages::Message::from_string(&input) {
                Ok(message) => match cli.player.client.write_message(&message).await {
                    Ok(_) => {
                        cli.stage = tap::cli::CliStage::WaitingResponse;
                        cli.waiter.begin();
                    },
                    Err(e) => cli.state.log(tap::cli::Message::error(e)),
                },
                Err(_) => cli.state.log(tap::cli::Message::Error("invalid command".to_string())),
            }
        }
        _ => (),
    }
}

async fn receive(cli: &mut tap::cli::Cli, message: tap::messages::Message) {
    if match (&cli.stage, &message) {
        (tap::cli::CliStage::WaitingGreeting, tap::messages::Message::Response(message)) if message.payload.matches(&[
            tap::messages::PayloadPattern::String(Some("hello".to_string())),
            tap::messages::PayloadPattern::KeyValue(Some("proto".to_string())),
        ]) => {
            cli.stage = tap::cli::CliStage::EnteringUsername;
            true
        }
        (tap::cli::CliStage::WaitingAuth, message) if matches!(
            message,
            tap::messages::Message::Response(_) |
            tap::messages::Message::Error(_)
        ) => {
            if matches!(message, tap::messages::Message::Error(_)) {
                cli.stage = tap::cli::CliStage::EnteringUsername;
                true
            } else if let tap::messages::Message::Response(response) = message && response.payload.matches(&[
                tap::messages::PayloadPattern::String(Some("connected".to_string())),
            ]) {
                cli.stage = tap::cli::CliStage::EnteringCommand;
                true
            } else {
                false
            }
        }
        (tap::cli::CliStage::WaitingResponse, message) if matches!(
            message,
            tap::messages::Message::Response(_) |
            tap::messages::Message::Error(_)
        ) => {
            cli.stage = tap::cli::CliStage::EnteringCommand;
            true
        }
        (tap::cli::CliStage::EnteringCommand, message) if matches!(
            message,
            tap::messages::Message::Event(_)
        ) => {
            true
        }
        (_, _) => false,
    } {
        cli.state.log(tap::cli::Message::Incoming(message.to_string()));
        cli.waiter.end();
        return;
    };
    cli.state.log(tap::cli::Message::Error(format!("unexpected message received from the server: {message}")));
    cli.player.client.close();
    cli.waiter.end();
}

#[tokio::main]
async fn main() {
    match tap::cli::raw::run(validate, receive).await {
        Err(e) => eprintln!("Error: {e}"),
        _ => (),
    }
}
