use serenity::{all::*, async_trait, prelude::*};
use tokio::sync::RwLockReadGuard;

mod parser;

struct IreneKey;
impl TypeMapKey for IreneKey {
    type Value = Irene;
}

struct Irene {
    admins: Vec<u64>,
}

struct Handler;

struct CommandDetails {
    name: &'static str,
}

#[async_trait]
trait IreneCommand: Send + Sync {
    fn details(&self) -> CommandDetails;
    async fn run(&self, ctx: Context, msg: Message);
}

struct PurgeCommand;

#[async_trait]
impl IreneCommand for PurgeCommand {
    fn details(&self) -> CommandDetails {
        CommandDetails { name: "!purge" }
    }
    async fn run(&self, ctx: Context, msg: Message) {
        let amount = msg
            .content
            .split_whitespace()
            .nth(1)
            .unwrap()
            .parse()
            .unwrap();
        let channel_id = msg.channel_id;
        let messages = channel_id
            .messages(&ctx.http, GetMessages::new().limit(amount))
            .await
            .unwrap();
        channel_id
            .delete_messages(&ctx.http, &messages)
            .await
            .unwrap();
    }
}

struct ListAdminsCommand;

#[async_trait]
impl IreneCommand for ListAdminsCommand {
    fn details(&self) -> CommandDetails {
        CommandDetails { name: "!admins" }
    }
    async fn run(&self, ctx: Context, msg: Message) {
        let read = ctx.data.read().await;
        let admins = read
            .get::<IreneKey>()
            .unwrap()
            .admins
            .iter()
            .map(|id| format!("<@{}>", id));
        msg.channel_id
            .say(
                &ctx.http,
                format!("Admins: {}", admins.collect::<Vec<String>>().join(", ")),
            )
            .await
            .unwrap();
    }
}

struct CommandRouter;

impl CommandRouter {
    async fn message(&self, ctx: Context, msg: Message) {
        let commands: Vec<Box<dyn IreneCommand>> =
            vec![Box::new(PurgeCommand), Box::new(ListAdminsCommand)];
        for command in commands.iter() {
            if msg.content.starts_with(command.details().name) {
                println!("Running command: {}", command.details().name);
                command.run(ctx.clone(), msg.clone()).await;
            }
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.data.name == "ping" {
                command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new().content("Pong!"),
                        ),
                    )
                    .await
                    .unwrap();
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        CommandRouter.message(ctx, msg).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        Command::create_global_command(
            &ctx.http,
            CreateCommand::new("ping").description("Replies with Pong!"),
        )
        .await
        .unwrap();
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let admins: Vec<u64> = vec![350109440000917507, 1160173710779617332, 822547520986808392];
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    client
        .data
        .write()
        .await
        .insert::<IreneKey>(Irene { admins });
    client.start().await.unwrap();
}
