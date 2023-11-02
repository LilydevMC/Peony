use std::env;
use anyhow::anyhow;
use chrono::Utc;
use serenity::model::channel::Embed;
use serenity::model::webhook::Webhook;
use crate::models::{DiscordConfig, GithubConfig};
use crate::models::modrinth::ModrinthUrl;
use crate::models::modrinth::project::ProjectResponse;

pub async fn send_discord_webhook(
    discord_config: &DiscordConfig,
    modrinth_url: &ModrinthUrl,
    modrinth_project_id: &String,
    github_config: &GithubConfig,
    version_name: &String,
    changelog: &String
) -> Result<(), anyhow::Error> {
    let modrinth_token = match env::var("MODRINTH_TOKEN") {
        Ok(token) => token,
        Err(err) => return Err(anyhow!(
            "Failed to get `MODRINTH_TOKEN` from environment: {}",
            err
        ))
    };

    let modrinth_project = match reqwest::Client::new()
        .get(format!(
            "{}/project/{}",
            modrinth_url.labrinth,
            modrinth_project_id
        ))
        .header("Authorization", modrinth_token)
        .send().await {
        Ok(res) => {
            match res.json::<ProjectResponse>().await {
                Ok(json) => json,
                Err(err) => return Err(anyhow!(
                    "Error parsing response from get project: {}\n\
                     Make sure your Modrinth token can read projects!",
                    err.to_string()
                ))
            }
        },
        Err(err) => return Err(anyhow!(
            "Error getting project from project id: {}",
            err
        ))
    };

    let description = format!("\
        **New release!**\n\n\
        {} [GitHub](https://github.com/{}/{}/releases/latest)\n\
        {} [Modrinth]({}/modpack/{})\n\n\
        {}
        ",
            discord_config.github_emoji_id,
            github_config.repo_owner,
            github_config.repo_name,
            discord_config.modrinth_emoji_id,
            modrinth_url.knossos,
            modrinth_project.slug,
            changelog
    );

    let embed_color = match discord_config.embed_color {
        Some(color) => color,
        None => match discord_config.embed_color {
            Some(color) => color,
            None => match modrinth_project.color {
                Some(color) => color as u32,
                None => 0x232634
            }
        }
    } as i32;

    let release_time = Utc::now().format("%b, %d %Y %r");

    let embed = Embed::fake(|mut e| {
        if let Some(url) = &discord_config.embed_image_url {
            e = e.image(url)
        }

        if let Some(url) = &discord_config.thumbnail_image_url {
            e = e.thumbnail(url)
        }

        e.title(format!("{} {}", discord_config.title_emoji, version_name))
            .color(embed_color)
            .description(description)
            .footer(|f| {
                f.text(format!(
                    "{} | {} UTC",
                    modrinth_project.project_type.formatted(),
                    release_time
                ))
            })
    });

    let http = serenity::http::Http::new("token");
    let url = match env::var("WEBHOOK_URL") {
        Ok(url) => url,
        Err(err) => return Err(anyhow!(
            "Failed to get webhook url: {}", err
        ))
    };

    let webhook = Webhook::from_url(&http, &*url).await?;

    match webhook.execute(&http, true, |w| {
        w
            .content(&discord_config.discord_ping_role)
            .embeds(vec![embed])
    }).await {
        Ok(_) => Ok(()),
        Err(err) => return Err(anyhow!(
            "Failed to send Discord webhook: {}", err
        ))
    }
}

