use anyhow::anyhow;
use shuttle_poise::ShuttlePoise;
use dotenv::dotenv;
use poise::serenity_prelude::{self as serenity, GuildContainer, PartialGuild, Guild};

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = anyhow::Error;
type Context<'a> = poise::Context<'a, Data, Error>;

const MINE_QUERY: &'static str = "?container=mine-mc-1";
const MINE_ROLE: &'static str = "testing";

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

async fn get_guild(ctx: Context<'_>) -> Result<serenity::Guild, Error> {
    let guild = match ctx.guild() {
        Some(guild) => guild,
        None => {
            ctx.say("You are not a member of this server").await?;
            return Err(anyhow!("You are not a member of this server"));
        },
    };

    Ok(guild)
}

async fn get_minecraft_role(ctx: Context<'_>) -> Result<serenity::Role, Error> {
    let guild = get_guild(ctx).await?;
    
    let minecraft_role = 
    {
        let mine_roles = guild.roles
        .into_iter()
        .find(|(_, role)| role.name == MINE_ROLE);

        match mine_roles {
            Some(role) => role,
            None => {
                ctx.say("The Minecraft role does not exist").await?;
                return Err(anyhow!("The Minecraft role does not exist"));
            }
        }
    };

    Ok(minecraft_role.1)
}

#[poise::command(slash_command, prefix_command)]
async fn mine_restart(ctx: Context<'_>) -> Result<(), Error> {
    let guild = get_guild(ctx).await?;
    
    let minecraft_role = get_minecraft_role(ctx).await?;

    let partial_guild: PartialGuild = Guild::get(&ctx, guild.id).await?;
    let enum_guild = GuildContainer::Guild(partial_guild);

    let member = ctx.author();
    let has_role = member
        .has_role(ctx, enum_guild, &minecraft_role)
        .await?;

    if !has_role {
        ctx.say("You do not have the Minecraft role").await?;
        return Err(anyhow!("You do not have the Minecraft role"));
    }

    let api_url = std::env::var("API_URL").expect("missing API_URL");
    let url = format!("{}/restart{}", api_url, MINE_QUERY);
    let body = reqwest::get(url).await?.text().await?;

    ctx.say(body).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn mine_healthcheck(ctx: Context<'_>) -> Result<(), Error> {
    let guild = get_guild(ctx).await?;
    
    let minecraft_role = get_minecraft_role(ctx).await?;

    let partial_guild: PartialGuild = Guild::get(&ctx, guild.id).await?;
    let enum_guild = GuildContainer::Guild(partial_guild);

    let member = ctx.author();
    let has_role = member
        .has_role(ctx, enum_guild, &minecraft_role)
        .await?;

    if !has_role {
        ctx.say("You do not have the Minecraft role").await?;
        return Err(anyhow!("You do not have the Minecraft role"));
    }

    let api_url = std::env::var("API_URL").expect("missing API_URL");
    let url = format!("{}/healthcheck{}", api_url, MINE_QUERY);
    let body = reqwest::get(url).await?.text().await?;

    ctx.say(body).await?;

    Ok(())
}

#[shuttle_runtime::main]
async fn poise() -> ShuttlePoise<Data, Error> {
    dotenv().ok();

    let commands = vec![age(), mine_healthcheck()];

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        }).build().await.map_err(shuttle_runtime::CustomError::new)?;

Ok(framework.into())
}
