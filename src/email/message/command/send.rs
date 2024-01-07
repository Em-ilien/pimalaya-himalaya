use anyhow::Result;
use clap::Parser;
#[cfg(feature = "sendmail")]
use email::message::send::sendmail::SendMessageSendmail;
#[cfg(feature = "smtp")]
use email::message::send::smtp::SendMessageSmtp;
use log::info;
use std::io::{self, BufRead, IsTerminal};

#[cfg(feature = "sync")]
use crate::cache::arg::disable::CacheDisableFlag;
use crate::{
    account::arg::name::AccountNameFlag,
    backend::{Backend, BackendKind},
    config::TomlConfig,
    message::arg::MessageRawArg,
    printer::Printer,
};

/// Send a message.
///
/// This command allows you to send a raw message and to save a copy
/// to your send folder.
#[derive(Debug, Parser)]
pub struct MessageSendCommand {
    #[command(flatten)]
    pub message: MessageRawArg,

    #[cfg(feature = "sync")]
    #[command(flatten)]
    pub cache: CacheDisableFlag,

    #[command(flatten)]
    pub account: AccountNameFlag,
}

impl MessageSendCommand {
    pub async fn execute(self, printer: &mut impl Printer, config: &TomlConfig) -> Result<()> {
        info!("executing send message command");

        let (toml_account_config, account_config) = config.clone().into_account_configs(
            self.account.name.as_ref().map(String::as_str),
            #[cfg(feature = "sync")]
            self.cache.disable,
        )?;

        let send_message_kind = toml_account_config.send_message_kind();

        let backend = Backend::new(
            &toml_account_config,
            &account_config,
            send_message_kind,
            |builder| {
                match send_message_kind {
                    #[cfg(feature = "smtp")]
                    Some(BackendKind::Smtp) => {
                        builder.set_send_message(|ctx| {
                            ctx.smtp.as_ref().and_then(SendMessageSmtp::new)
                        });
                    }
                    #[cfg(feature = "sendmail")]
                    Some(BackendKind::Sendmail) => {
                        builder.set_send_message(|ctx| {
                            ctx.sendmail.as_ref().and_then(SendMessageSendmail::new)
                        });
                    }
                    _ => (),
                };
            },
        )
        .await?;

        let msg = if io::stdin().is_terminal() {
            self.message.raw()
        } else {
            io::stdin()
                .lock()
                .lines()
                .filter_map(Result::ok)
                .collect::<Vec<_>>()
                .join("\r\n")
        };

        backend.send_message(msg.as_bytes()).await?;

        printer.print("Message successfully sent!")
    }
}
