use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use blog_client::{BlogClient, Transport};
use clap::{Parser, Subcommand};

/// Path where the JWT token is persisted between CLI invocations
const TOKEN_FILE: &str = ".blog_token";

/// Default server addresses
const DEFAULT_HTTP_SERVER: &str = "http://localhost:8080";
const DEFAULT_GRPC_SERVER: &str = "http://localhost:50051";

#[derive(Parser)]
#[command(name = "blog-cli", about = "Blog API client", version)]
struct Cli {
    /// Use gRPC transport (default: HTTP)
    #[arg(long, global = true)]
    grpc: bool,

    /// Server address (overrides default)
    #[arg(long, global = true)]
    server: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Register a new user
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        password: String,
    },
    /// Log in as an existing user
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    /// Create a new post (requires login)
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        content: String,
    },
    /// Get a post by ID
    Get {
        #[arg(long)]
        id: i64,
    },
    /// Update a post (requires login)
    Update {
        #[arg(long)]
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    /// Delete a post (requires login)
    Delete {
        #[arg(long)]
        id: i64,
    },
    /// List posts with optional pagination
    List {
        #[arg(long, default_value = "10")]
        limit: i32,
        #[arg(long, default_value = "0")]
        offset: i32,
    },
}

fn token_path() -> PathBuf {
    PathBuf::from(TOKEN_FILE)
}

fn load_token() -> Option<String> {
    fs::read_to_string(token_path()).ok().map(|s| s.trim().to_owned())
}

fn save_token(token: &str) -> Result<()> {
    fs::write(token_path(), token).context("Failed to save token to .blog_token")
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let transport = if cli.grpc {
        let addr = cli
            .server
            .unwrap_or_else(|| DEFAULT_GRPC_SERVER.to_owned());
        Transport::Grpc(addr)
    } else {
        let addr = cli
            .server
            .unwrap_or_else(|| DEFAULT_HTTP_SERVER.to_owned());
        Transport::Http(addr)
    };

    let mut client = BlogClient::new(transport)
        .await
        .context("Failed to connect to server")?;

    if let Some(token) = load_token() {
        client.set_token(token);
    }

    match cli.command {
        Commands::Register {
            username,
            email,
            password,
        } => {
            let resp = client
                .register(&username, &email, &password)
                .await
                .context("Register failed")?;
            save_token(&resp.token)?;
            println!("Registered successfully!");
            println!("User: {} ({})", resp.user.username, resp.user.email);
            println!("Token saved to {TOKEN_FILE}");
        }

        Commands::Login { username, password } => {
            let resp = client
                .login(&username, &password)
                .await
                .context("Login failed")?;
            save_token(&resp.token)?;
            println!("Logged in as: {}", resp.user.username);
            println!("Token saved to {TOKEN_FILE}");
        }

        Commands::Create { title, content } => {
            let post = client
                .create_post(&title, &content)
                .await
                .context("Create post failed")?;
            println!("Post created (id={})", post.id);
            println!("Title:   {}", post.title);
            println!("Author:  {}", post.author_username);
            println!("Created: {}", post.created_at);
        }

        Commands::Get { id } => {
            let post = client
                .get_post(id)
                .await
                .context("Get post failed")?;
            println!("ID:      {}", post.id);
            println!("Title:   {}", post.title);
            println!("Author:  {}", post.author_username);
            println!("Content:\n{}", post.content);
            println!("Created: {}", post.created_at);
            println!("Updated: {}", post.updated_at);
        }

        Commands::Update { id, title, content } => {
            let post = client
                .update_post(id, title.as_deref(), content.as_deref())
                .await
                .context("Update post failed")?;
            println!("Post updated (id={})", post.id);
            println!("Title:   {}", post.title);
            println!("Updated: {}", post.updated_at);
        }

        Commands::Delete { id } => {
            client
                .delete_post(id)
                .await
                .context("Delete post failed")?;
            println!("Post {id} deleted.");
        }

        Commands::List { limit, offset } => {
            let result = client
                .list_posts(limit, offset)
                .await
                .context("List posts failed")?;
            println!(
                "Showing {}/{} posts (offset={}):",
                result.posts.len(),
                result.total,
                result.offset
            );
            println!("{:-<60}", "");
            for post in &result.posts {
                println!(
                    "[{}] {} — by {} ({})",
                    post.id, post.title, post.author_username, post.created_at
                );
            }
        }
    }

    Ok(())
}
