use std::time::SystemTime;
use reqwest;
use serde_json;
use serde::Deserialize;

/// Represents the configuration for the GitHub repository and commit details.
#[derive(Deserialize)]
struct Config {
    github_token: String,
    github_username: String,
    github_file_path: String,
    github_repo_name: String,
    github_branch: String,
    github_commit_message: String,
}

/// Checks the internet connection by sending a request to Google.
/// Returns `Ok(())` if the request is successful, otherwise returns an error.
fn check_internet_connection() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();
    let res = client.get("http://www.google.com").send()?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No internet connection")))
    }
}

/// Retrieves the configuration from the `config.json` file.
/// Returns the parsed `Config` struct if successful, otherwise returns an error.
fn retrieve_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config = std::fs::read_to_string("config.json")?;
    let config: Config = serde_json::from_str(&config)?;
    Ok(config)
}

/// Retrieves the SHA of the last commit on the specified branch of the GitHub repository.
/// Returns the SHA as a string if successful, otherwise returns an error.
fn retrieve_sha(config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>> {
    let uri = format!("https://api.github.com/repos/{}/{}/branches/{}", &config.github_username, &config.github_repo_name, &config.github_branch);
    let request = client.get(&uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1"); 
    let res = request.send()?;
    let body = res.text()?; 
    let sha: serde_json::Value = serde_json::from_str(&body)?;
    let sha = sha["commit"]["sha"].to_string();
    Ok(sha)
}

/// Creates a blob with the content of the commit message.
/// Returns the SHA of the created blob if successful, otherwise returns an error.
fn create_blob(config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>> {
    let now = SystemTime::now();
    let content = "This is an auto commit from Kommitter at ".to_owned() + &now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs().to_string() + " seconds since the Unix Epoch.";
    let uri = format!("https://api.github.com/repos/{}/{}/git/blobs", &config.github_username, &config.github_repo_name);
    let request = client.post(&uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
        .header(reqwest::header::AUTHORIZATION, format!("token {}", &config.github_token))
        .body(format!(r#"{{"content": "{}", "encoding": "utf-8"}}"#, content));
    let res = request.send()?;
    let body = res.text()?; 
    let blob_sha: serde_json::Value = serde_json::from_str(&body)?;
    let blob_sha = blob_sha["sha"].to_string();
    Ok(blob_sha)
}

/// Creates a tree with the specified SHA and blob SHA.
/// Returns the SHA of the created tree if successful, otherwise returns an error.
fn create_tree(sha: String, blob_sha: String, config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>> {
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
    }}"#, sha, &config.github_file_path, blob_sha);    
    let uri = format!("https://api.github.com/repos/{}/{}/git/trees", &config.github_username, &config.github_repo_name);
    let tree = client.post(&uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
        .header(reqwest::header::AUTHORIZATION, format!("token {}", &config.github_token))
        .body(json_string);
    let res = tree.send()?;
    let body = res.text()?;
    let tree_sha: serde_json::Value = serde_json::from_str(&body)?;
    let tree_sha = tree_sha["sha"].to_string();
    Ok(tree_sha)
}

/// Creates a commit with the specified SHA, tree SHA, and commit message.
/// Returns the SHA of the created commit if successful, otherwise returns an error.
fn create_commit(sha: String, tree_sha: String, config: &Config, client: &reqwest::blocking::Client) -> Result<String, Box<dyn std::error::Error>> {
    let json_string = format!(r#"{{
        "message": "{}",
        "tree": {},
        "parents": [
            {}
        ]
    }}"#, &config.github_commit_message, tree_sha, sha);
    let uri = format!("https://api.github.com/repos/{}/{}/git/commits", &config.github_username, &config.github_repo_name);
    let commit = client.post(&uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
        .header(reqwest::header::AUTHORIZATION, format!("token {}", &config.github_token))
        .body(json_string);
    let res = commit.send()?;
    let body = res.text()?;
    let commit_sha: serde_json::Value = serde_json::from_str(&body)?;
    let commit_sha = commit_sha["sha"].to_string();
    Ok(commit_sha)
}

/// Updates the reference of the branch to the specified commit SHA.
/// Returns `Ok(())` if successful, otherwise returns an error.
fn update_ref(config: &Config, commit_sha: String, client: &reqwest::blocking::Client) -> Result<(), Box<dyn std::error::Error>> {
    let uri = format!("https://api.github.com/repos/{}/{}/git/refs/heads/{}", &config.github_username, &config.github_repo_name, &config.github_branch);
    let update_ref = client.patch(&uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
        .header(reqwest::header::AUTHORIZATION, format!("token {}", &config.github_token))
        .body(format!(r#"{{"sha": {}}}"#, commit_sha));
    update_ref.send()?;
    Ok(())
}

/// Patches the branch with the specified commit SHA.
/// Returns `Ok(())` if successful, otherwise returns an error.
fn patch(config: &Config, commit_sha: String, client: &reqwest::blocking::Client) -> Result<(),Box<dyn std::error::Error>> {
    let uri = format!("https://api.github.com/repos/{}/{}/branches/{}", &config.github_username, &config.github_repo_name, &config.github_branch);
    let res = client.patch(&uri)
        .header(reqwest::header::USER_AGENT, "my-app/0.0.1")
        .header(reqwest::header::AUTHORIZATION, format!("token {}", &config.github_token))
        .body(format!(r#"{{"sha": {}}}"#, commit_sha));
    res.send()?;
    Ok(())
}

/// The main function that executes the commit process.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    check_internet_connection()?;
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