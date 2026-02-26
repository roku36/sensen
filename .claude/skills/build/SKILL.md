---
name: build
description: Sensenプロジェクトのビルド・コンパイルチェック。cargo buildやcargo checkを実行し、エラーがあれば修正する。
user-invocable: true
allowed-tools: Bash, Read, Edit, Grep, Glob
argument-hint: [--check | --web | --release | --fix]
---

# Sensen Build

Sensenプロジェクト（Bevy 0.18カードゲーム）のビルドを実行する。

## 手順

1. 引数に応じてコマンドを選択:
   - `--check` または引数なし: `cargo check --features dev`
   - `--web`: `cargo check --target wasm32-unknown-unknown --no-default-features` (wasmビルド確認)
   - `--release`: `cargo build --release`
   - `--fix`: `cargo check --features dev` を実行し、エラーがあれば自動修正

2. ビルドを実行する

3. エラーがある場合:
   - エラーメッセージを解析
   - 該当ファイルを読んで修正
   - 再度ビルドして確認

## 注意点
- `dev` feature: dynamic_linking, dev_tools, ui_debug, track_location
- `dev_native` feature: dev + file_watcher, embedded_watcher, bevy_remote (BRP)
- wasm32ビルド: `--no-default-features` 必須（`dev_native`のBRPがwasmで動かない）
- `matchbox_socket`: ローカルパッチ版 (`third_party/matchbox_socket`, Cargo.toml [patch])
- Edition 2024を使用
