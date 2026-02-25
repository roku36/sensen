---
name: test-p2p
description: P2P対戦テスト。web版またはnative版でmatchbox_server + クライアントを起動しテストする。
user-invocable: true
allowed-tools: Bash, Read
argument-hint: [web|native] [action]
---

# Sensen P2P Test

`tools/debug-p2p` でP2P環境を起動し、テストする。

## モード選択

| モード | 用途 | BRP操作 |
|--------|------|---------|
| `web` | web版の動作確認・マッチング速度調査 | 不可（matchboxログで確認） |
| `native` | BRPでゲームロジック・同期のデバッグ | `tools/brp` で操作可能 |

**原則**: 挙動テスト・ゲームロジックの確認 → `native`、web固有の問題調査 → `web`

## テスト手順

### 1. 環境起動

```bash
# web版（ブラウザで2タブ開く）
tools/debug-p2p web &

# native版（BRP付きクライアント2つ）
tools/debug-p2p native &
```

matchbox_serverは自動起動され、ログは `/tmp/matchbox.log` に出力される。

### 2. matchboxログ確認

```bash
cat /tmp/matchbox.log
```

マッチング遅延の調査にはタイムスタンプを確認。

### 3. ゲーム操作（native のみ）

```bash
# Client 1でドロー
tools/brp draw --port 15702

# Client 1でカード1をプレイ
tools/brp play 1 --port 15702
```

### 4. 状態確認（native のみ）

```bash
tools/brp status --port 15702
tools/brp status --port 15703
```

### 5. クライアントログ確認

```bash
# native
grep -i "synchronized\|gameplay\|matchbox\|peer" /tmp/client1.log
grep -i "synchronized\|gameplay\|matchbox\|peer" /tmp/client2.log
```

### 6. 期待される同期結果
Client 1がカードをプレイした場合:
- Client 1: Opponent HP減少（自分が相手を攻撃）
- Client 2: LocalPlayer HP減少（相手から攻撃を受けた）

## クリーンアップ
`tools/debug-p2p` は Ctrl+C で自動クリーンアップ。手動の場合:
```bash
pkill -f "sensen.*brp-port"
pkill -f matchbox_server
```
