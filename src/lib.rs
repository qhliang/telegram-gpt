use serde_json::json;
use tg_flows::{listen_to_update, Telegram, Update, UpdateKind};
use store_flows::{get, set};
use flowsnet_platform_sdk::logger;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() -> anyhow::Result<()> {
    logger::init();
    let telegram_token = std::env::var("telegram_token").unwrap();
    let placeholder_text = std::env::var("placeholder").unwrap_or("Typing ...".to_string());
    let system_prompt = std::env::var("system_prompt").unwrap_or("You are a helpful assistant answering questions on Telegram.".to_string());
    let help_mesg = std::env::var("help_mesg").unwrap_or("I am your assistant on Telegram. Ask me any question! To start a new conversation, type the /restart command.".to_string());

    listen_to_update(&telegram_token, |update| {
        let tele = Telegram::new(telegram_token.to_string());
        handler(tele, &placeholder_text, &system_prompt, &help_mesg, update)
    }).await;

    Ok(())
}

async fn handler(tele: Telegram, placeholder_text: &str, system_prompt: &str, help_mesg: &str, update: Update) {
    if let UpdateKind::Message(msg) = update.kind {
        let chat_id = msg.chat.id;
        log::info!("Received message from {}", chat_id);

        let text = msg.text().unwrap_or("");
        if text.eq_ignore_ascii_case("/help") {
            _ = tele.send_message(chat_id, help_mesg);

        } else if text.eq_ignore_ascii_case("/start") {
            _ = tele.send_message(chat_id, help_mesg);
            set(&chat_id.to_string(), json!(true), None);
            log::info!("Started converstion for {}", chat_id);

        } else if text.eq_ignore_ascii_case("/restart") {
            _ = tele.send_message(chat_id, "Ok, I am starting a new conversation.");
            set(&chat_id.to_string(), json!(true), None);
            log::info!("Restarted converstion for {}", chat_id);

        } else {
            let placeholder = tele
                .send_message(chat_id, placeholder_text)
                .expect("Error occurs when sending Message to Telegram");

            let restart = match get(&chat_id.to_string()) {
                Some(v) => v.as_bool().unwrap_or_default(),
                None => false,
            };
            if restart {
                log::info!("Detected restart = true");
                set(&chat_id.to_string(), json!(false), None);
            }

            tele.edit_message_text(chat_id, placeholder.id, "Sorry, an error has occured. Please try again later!");
        }
    }
}
