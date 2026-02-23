---
name: test-p2p
description: P2P対戦テスト。matchbox_serverとクライアント2つを起動し、BRP経由でゲームプレイをテストする。
user-invocable: true
allowed-tools: Bash, Read
argument-hint: [action]
---

# Sensen P2P Test

`tools/brp` スクリプトを使ってP2P対戦をテストする。

## 環境確認

まず以下を確認:
- `matchbox_server` が起動しているか (`pgrep -f matchbox_server`)
- 起動していなければ `matchbox_server &` で起動

## テスト手順

### 1. クライアント起動
```bash
cargo run --features dev -- --brp-port=15702 --lobby > /tmp/client1.log 2>&1 &
sleep 2
cargo run --features dev -- --brp-port=15703 --lobby > /tmp/client2.log 2>&1 &
sleep 5
```

### 2. GGRS接続待ち
Lobby画面で自動接続。Gameplay画面に自動遷移するまで待つ。
ログで確認: `grep -i "synchronized\|gameplay" /tmp/client1.log`

### 3. ゲーム操作（tools/brp経由）

```bash
# Client 1でドロー
tools/brp draw --port 15702

# Client 1でカード1をプレイ
tools/brp play 1 --port 15702
```

### 4. 状態確認

```bash
# Client 1の全状態
tools/brp status --port 15702

# Client 2の全状態
tools/brp status --port 15703

# HPだけ確認
tools/brp hp --port 15702
tools/brp hp --port 15703
```

### 5. 期待される同期結果
Client 1がカードをプレイした場合:
- Client 1: Opponent HP減少（自分が相手を攻撃）
- Client 2: LocalPlayer HP減少（相手から攻撃を受けた）

## クリーンアップ
```bash
pkill -f "sensen.*brp-port"
```
