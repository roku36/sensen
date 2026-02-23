---
name: brp
description: BRP（Bevy Remote Protocol）で起動中のSensenゲームを操作・デバッグする。HP確認、カードプレイ、画面遷移、スクリーンショットなど。
user-invocable: true
allowed-tools: Bash
argument-hint: [command] [--port PORT]
---

# Sensen BRP Debug

`tools/brp` スクリプトで起動中のSensenクライアントにBRP経由でコマンドを送る。
出力はjqで整形済みのコンパクトなテキスト。

## 使い方

引数をそのまま `tools/brp` に渡す。

```bash
tools/brp <command> [args] [--port PORT]
```

## コマンド一覧

| コマンド | 説明 |
|---------|------|
| `hp` / `health` | 両プレイヤーのHP・Block表示 |
| `cost` | コスト状態 |
| `hand` | 手札のカードID一覧 |
| `deck` | デッキ/手札/捨て札の枚数 |
| `block` | Block値 |
| `buffs` | ステータス効果（Str/Vuln/Weak/Thorns） |
| `status` | 上記すべて一括表示 |
| `draw` | ドロー（GGRS同期） |
| `play N` | N番目のカードをプレイ（GGRS同期） |
| `goto SCREEN` | 画面遷移（Title/Lobby/Gameplay） |
| `screenshot [path]` | スクリーンショット |
| `query COMPONENT` | 任意コンポーネントのraw query |

## 引数の解釈

ユーザーの自然言語入力を以下のようにマッピングする:
- 「HP見せて」「体力は？」→ `tools/brp hp`
- 「全部見せて」「状態」→ `tools/brp status`
- 「ドローして」→ `tools/brp draw`
- 「1番プレイ」→ `tools/brp play 1`
- 「Lobbyに行って」→ `tools/brp goto Lobby`
- ポート指定: `tools/brp hp --port 15703`

## 両クライアント同時操作

2クライアント同時に状態確認する場合:
```bash
tools/brp status --port 15702
tools/brp status --port 15703
```
