# Kommitter
Kommitter is a simple CLI tool to commit to a github repo.
## Usage
1. Create a config.json file in the same directory as the binary.
2. Fill in the config.json file with the following fields:
    - github_token: Your github token.
    - github_username: Your github username.
    - github_file_path: The path to the file you want to commit.
    - github_repo_name: The name of the repo you want to commit to.
    - github_branch: The branch you want to commit to.
    - github_commit_message: The commit message.
3. Run the binary.

## Example config.json
```json
{
    "github_token": "1234567890",
    "github_username": "username",
    "github_file_path": "README.md",
    "github_repo_name": "kommitter",
    "github_branch": "main",
    "github_commit_message": "Update README.md"
}
```

## Example output
```bash
$ ./kommitter
Getting last commit SHA...
sha: "1234567890"
Creating blob...
sha: "1234567890"
Creating tree...
sha: "1234567890"
Creating commit...
sha: "1234567890"
Updating ref...
Patching...
Done!
```

## License
[Mozilla Public License 2.0](https://choosealicense.com/licenses/mpl-2.0/)