# dataset-tag-editor-rust
![image](./img/screenshot.png)

## How to download
https://github.com/ya0201/dataset-tag-editor-rust/releases/latest

## How to release

### Github action
run `bump-version` action

### Manual release
```
git switch main && export tag=vX.Y.Z && git tag ${tag} && git push origin ${tag}
```
