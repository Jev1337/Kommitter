use std::time::SystemTime;
use reqwest;
use serde_json;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config{
    github_token: String,
    github_username: String,
    github_file_path: String,
    github_repo_name: String,
    github_branch: String,
    github_commit_message: String,
}

fn retrieve_config() -> Result<Config, Box<dyn std::error::Error>>{
    let config = std::fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config)?;
    Ok(config)
}
    
fn retrieve_sha(config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>>{
    let uri = format!("https://api.github.com/repos/{}/{}/branches/{}", config.github_username.clone(), config.github_repo_name.clone(), config.github_branch.clone());
    let request = client.get(uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1"); 
    let res = request.send()?;
    let body = res.text()?; 
    let sha: serde_json::Value = serde_json::from_str(&body)?;
    let sha = sha["commit"]["sha"].to_string();
    Ok(sha)
}

fn create_blob(config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>>{
    let now = SystemTime::now();
    let content = now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs().to_string();
    let uri = format!("https://api.github.com/repos/{}/{}/git/blobs",config.github_username.clone(), config.github_repo_name.clone());
    let request = client.post(uri)
    .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
    .header(reqwest::header::AUTHORIZATION, format!("token {}", config.github_token.clone()))
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
    let uri = format!("https://api.github.com/repos/{}/{}/git/trees",config.github_username.clone(), config.github_repo_name.clone());
    let tree = client.post(uri)
    .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
    .header(reqwest::header::AUTHORIZATION, format!("token {}", config.github_token.clone()))
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
    let uri = format!("https://api.github.com/repos/{}/{}/git/commits", config.github_username.clone(), config.github_repo_name.clone());
    let commit = client.post(uri)
    .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
    .header(reqwest::header::AUTHORIZATION, format!("token {}", config.github_token.clone()))
    .body(json_string);
    let res = commit.send()?;
    let body = res.text()?;
    let commit_sha: serde_json::Value = serde_json::from_str(&body)?;
    let commit_sha = commit_sha["sha"].to_string();
    Ok(commit_sha)
}

fn update_ref(config: &Config, commit_sha: String, client: &reqwest::blocking::Client) -> Result<(), Box<dyn std::error::Error>>{
    let uri = format!("https://api.github.com/repos/{}/{}/git/refs/heads/{}", config.github_username.clone(),config.github_repo_name.clone(), config.github_branch.clone());
    let update_ref = client.patch(uri)
    .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
    .header(reqwest::header::AUTHORIZATION, format!("token {}", config.github_token.clone()))
    .body(format!(r#"{{"sha": {}}}"#, commit_sha));
    update_ref.send()?;
    Ok(())
}

fn patch(config: &Config, commit_sha: String, client: &reqwest::blocking::Client) -> Result<(),Box<dyn std::error::Error>>{
    let uri = format!("https://api.github.com/repos/{}/{}/branches/{}",  config.github_username.clone(), config.github_repo_name.clone(), config.github_branch.clone());
    let res = client.patch(uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
        .header(reqwest::header::AUTHORIZATION, format!("token {}", config.github_token.clone()))
        .body(format!(r#"{{"sha": {}}}"#, commit_sha));
    res.send()?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let config = retrieve_config()?;
    println!("Getting last commit SHA...");
    let sha = retrieve_sha(&config, &client)?;
    println!("sha: {}", sha);
    println!("Creating blob...");
    let blob_sha = create_blob(&config, &client)?;
    println!("sha: {}", blob_sha);
    println!("Creating tree...");
    let tree_sha = create_tree(sha.clone(), blob_sha.clone(), &config, &client)?;
    println!("sha: {}", tree_sha);
    println!("Creating commit...");
    let commit_sha = create_commit(sha.clone(), tree_sha.clone(), &config, &client)?;
    println!("sha: {}", commit_sha);
    println!("Updating ref...");
    update_ref(&config, commit_sha.clone(), &client)?;
    println!("Patching...");
    patch(&config, commit_sha.clone(), &client)?;
    println!("Done!");
    Ok(())
}
