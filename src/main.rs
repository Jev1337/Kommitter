use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::SystemTime;
use reqwest;
use serde_json;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config{
    github_token: String,
    github_file_path: String,
    github_repo_name: String,
    github_branch: String,
    github_commit_message: String,
    localfile_path: String,
}

fn retrieve_config() -> Result<Config, Box<dyn std::error::Error>>{
    let config = std::fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config)?;
    Ok(config)
}
    
fn retrieve_sha(config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>>{
    let uri = format!("https://api.github.com/repos/{}/branches/{}", config.github_repo_name.clone(), config.github_branch.clone());
    let request = client.get(uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1"); 
    let res = request.send()?;
    let body = res.text()?; 
    let sha: serde_json::Value = serde_json::from_str(&body)?;
    let sha = sha["commit"]["sha"].to_string();
    Ok(sha)
}

fn create_blob(config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>>{
    // avoid borrow of partially moved value
    let mut file = OpenOptions::new()
        .read(true)
        .open(config.localfile_path.clone())?;
    let now = SystemTime::now();
    writeln!(file, "Commit + {:?}", now)?;
    let content = std::fs::read_to_string(config.localfile_path.clone())?;
    let request = client.post("https://api.github.com/repos/Jev1337/NiceOpener/git/blobs")
    .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
    .header(reqwest::header::AUTHORIZATION, format!("token {}", config.github_token))
    .json(&serde_json::json!({
        "content": content,
        "encoding": "utf-8"
    }));
    let res = request.send()?;
    let body = res.text()?; 
    let blob_sha: serde_json::Value = serde_json::from_str(&body)?;
    let blob_sha = blob_sha["sha"].to_string();
    Ok(blob_sha)
}

fn create_tree(sha: String, blob_sha: String, config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>>{
    let json_string = format!(r#"{{
        "base_tree": {},
        "tree": [
            {{
                "path": "{}",
                "mode": "100644",
                "type": "blob",
                "sha": {}
            }}
        ]
    }}"#, sha, config.github_file_path.clone(), blob_sha);    
    let tree = client.post("https://api.github.com/repos/Jev1337/NiceOpener/git/trees")
    .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
    .header(reqwest::header::AUTHORIZATION, "token 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")
    .body(json_string);
    let res = tree.send()?;
    let body = res.text()?;
    let tree_sha: serde_json::Value = serde_json::from_str(&body)?;
    let tree_sha = tree_sha["sha"].to_string();
    Ok(tree_sha)
}

fn create_commit(sha: String, tree_sha: String, config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>>{
    let json_string = format!(r#"{{
        "message": "{}",
        "tree": {},
        "parents": [
            {}
        ]
    }}"#, config.github_commit_message.clone(), tree_sha, sha);
    let commit = client.post("https://api.github.com/repos/Jev1337/NiceOpener/git/commits")
    .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
    .header(reqwest::header::AUTHORIZATION, "token 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")
    .body(json_string);
    let res = commit.send()?;
    let body = res.text()?;
    let commit_sha: serde_json::Value = serde_json::from_str(&body)?;
    let commit_sha = commit_sha["sha"].to_string();
    Ok(commit_sha)
}

fn update_ref(commit_sha: String, client: &reqwest::blocking::Client) -> Result<(), Box<dyn std::error::Error>>{
    let update_ref = client.patch("https://api.github.com/repos/Jev1337/NiceOpener/git/refs/heads/main")
    .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
    .header(reqwest::header::AUTHORIZATION, "token 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")
    .body(format!(r#"{{"sha": {}}}"#, commit_sha));
    update_ref.send()?;
    Ok(())
}

fn patch(commit_sha: String, client: &reqwest::blocking::Client) -> Result<(), Box<dyn std::error::Error>>{
    let res = client.patch("https://api.github.com/repos/Jev1337/NiceOpener/branches/main")
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
        .header(reqwest::header::AUTHORIZATION, "token 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")
        .body(format!(r#"{{"sha": {}}}"#, commit_sha));
    let res = res.send()?;
    let branch_sha = res.json::<serde_json::Value>()?;
    let branch_sha = branch_sha["sha"].to_string();
    println!("sha: {}", branch_sha);
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let config = retrieve_config()?;
    let sha = retrieve_sha(&config, &client)?;
    println!("Getting last commit SHA...");
    println!("sha: {}", sha);
    println!("Creating blob...");
    let blob_sha = create_blob(&config, &client)?;
    println!("sha: {}", blob_sha);
    println!("Creating tree...");
    let tree_sha = create_tree(sha.clone(), blob_sha.clone(), &config, &client)?;
    println!("sha: {}", tree_sha);
    let commit_sha = create_commit(sha.clone(), tree_sha.clone(), &config, &client)?;
    println!("sha: {}", commit_sha);
    println!("Updating ref...");
    update_ref(commit_sha.clone(), &client)?;
    println!("updated");
    patch(commit_sha.clone(), &client)?;
    println!("patched");
    Ok(())
}
