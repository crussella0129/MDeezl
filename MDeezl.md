We make a small program that takes a given path for a directory/repo and gives a .md schema (via the [Scaffolding symbols generator]) and also applies the following scripts to the repo: 

```Bash/Zsh: find . -type f -not -path '*/.*' -exec awk 'BEGINFILE {print "\n---\nFile: " FILENAME "\n---\n"} {print}' {} + > repo_context.txt```

```Pwrshell: Get-ChildItem -Recurse -File | Where-Object { $_.FullName -notlike '*\.git\*' } | ForEach-Object { "---`nFile: $($_.FullName)`n---" | Out-File -FilePath .\repo_context.txt -Append Get-Content $_.FullName | Out-File -FilePath .\repo_context.txt -Append }```

So we can get both a printout of the structure of the repo and a plain text printout of the contents of the repository in .md (ex.):
 
```Kinesin/```
```├── .github/                        # ex comment 1```
```│   └── workflows/              # ex comment 2```
```└── README.md                # ex comment 3```

---
File: .github/workflows/
---
[Code contents here...]

---
File: README.md
---
[Code contents here...]
