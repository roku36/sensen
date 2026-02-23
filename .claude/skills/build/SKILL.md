---
name: build
description: Sensenプロジェクトのビルド・コンパイルチェック。cargo buildやcargo checkを実行し、エラーがあれば修正する。
user-invocable: true
allowed-tools: Bash, Read, Edit, Grep, Glob
argument-hint: [--release | --check | --fix]
---

# Sensen Build

Sensenプロジェクト（Bevy 0.17カードゲーム）のビルドを実行する。

## 手順

1. 引数に応じてコマンドを選択:
   - `--check` または引数なし: `cargo check --features dev`
   - `--release`: `cargo build --release`
   - `--fix`: `cargo check --features dev` を実行し、エラーがあれば自動修正

2. ビルドを実行する

3. エラーがある場合:
   - エラーメッセージを解析
   - 該当ファイルを読んで修正
   - 再度ビルドして確認

## 注意点
- devフィーチャーにはBRP、dev_tools、dynamic_linkingが含まれる
- Edition 2024を使用
- `bevy_la_mesa`, `bevy_ggrs`, `bevy_matchbox`などのクレート依存あり
