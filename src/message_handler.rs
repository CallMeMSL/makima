use crate::setup::get_user_store;
use crate::store::Entry;
use anyhow::{anyhow, Result};
use serenity::all::{Context, Message};

pub async fn message_handler(ctx: Context, msg: Message) -> Result<()> {
    let (op, arg) = split_at_fist_space(&msg.content);
    let ctx_fallback = ctx.clone();
    let msg_fallback = msg.clone();
    if let Err(e) = match (op.as_str(), arg.as_str()) {
        ("help", _) => help(ctx, msg).await,
        ("add", ident) => add(ctx, msg, ident).await,
        ("list", _) => list_patterns(ctx, msg).await,
        ("remove", "all") => remove_all(ctx, msg).await,
        ("remove", ident) => remove(ctx, msg, ident).await,
        _ => Err(anyhow!(
            "Unknown Command. Check available commands with `help`."
        )),
    } {
        msg_fallback
            .reply(ctx_fallback, format!("an error occurred: {e}"))
            .await?;
    };
    Ok(())
}

async fn help(ctx: Context, msg: Message) -> Result<()> {
    msg.reply(
        ctx,
        "```Usage:\n\
              add pat\t\tchecks new releases for pat and notifies you about them\n\
              list\t\tlists all your patterns with their corresponding index\n\
              remove index|all\t\tremoves the pattern at that index or all of them\n\
              help\t\tshows this message```",
    )
        .await?;
    Ok(())
}

async fn add(ctx: Context, msg: Message, pat: &str) -> Result<()> {
    let user_id = msg.author.id.get();
    let mut store = get_user_store().write().await;
    let new_entry = Entry::new(user_id,
                               pat.to_string()
                                   .split(';')
                                      .map(|s| s.to_string())
                                   .collect());
    store.add(new_entry)?;
    drop(store);
    msg.reply(ctx, "pattern added").await?;
    Ok(())
}

async fn list_patterns(ctx: Context, msg: Message) -> Result<()> {
    let user_id = msg.author.id.get();
    let store = get_user_store().read().await;
    let user_patterns = store.get_elements_for_user(user_id);
    drop(store);
    let msg_text = format!(
        "```{}\n```",
        user_patterns
            .into_iter()
            .enumerate()
            .map(|(i, p)| format!("{i}\t\t{}", p.join("\t")))
            .collect::<Vec<String>>()
            .join("\n")
    );
    msg.reply(ctx, msg_text).await?;
    Ok(())
}

async fn remove_all(ctx: Context, msg: Message) -> Result<()> {
    let user_id = msg.author.id.get();
    let mut store = get_user_store().write().await;
    store.remove_user(user_id)?;
    drop(store);
    msg.reply(ctx, "successfully removed all patterns").await?;
    Ok(())
}

async fn remove(ctx: Context, msg: Message, index: &str) -> Result<()> {
    let index = index.parse()?;
    let user_id = msg.author.id.get();
    let mut store = get_user_store().write().await;
    store.remove_by_index(user_id, index)?;
    drop(store);
    msg.reply(ctx, "successfully removed pattern").await?;
    Ok(())
}

fn split_at_fist_space(command: &str) -> (String, String) {
    let mut operand = Vec::new();
    let mut argument = Vec::new();
    let mut take_operand = true;
    for c in command.chars() {
        if c == ' ' && take_operand {
            take_operand = false;
            continue;
        }
        if take_operand {
            operand.push(c);
        } else {
            argument.push(c);
        }
    }
    (operand.iter().collect(), argument.iter().collect())
}
