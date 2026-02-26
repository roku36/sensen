# Sensen 開発ガイド (Claude向け)

## プロジェクト概要

Bevy 0.18製のリアルタイムP2P対戦カードゲーム。GGRS + Matchbox WebRTCでロールバックネットコード実装。
Web版のみ配信（GitHub Pages）。P2PサーバーはFly.io。

## 重要なアーキテクチャ

### 画面遷移
- `Screen` state: Splash → Title → Lobby → Gameplay
- Lobby画面でMatchboxサーバーに接続、2人揃うとGGRSセッション開始

### ネットワーク
- Matchboxサーバー:
  - dev: `ws://localhost:3536/sensen?next=2` (`matchbox_server`コマンドで起動)
  - release: `wss://sensen-matchbox.fly.dev/sensen?next=2` (fly.io)
  - `src/network/lobby.rs` で `cfg(feature = "dev")` で切り替え
- matchbox_socket: **ローカルパッチ版** (`third_party/matchbox_socket`)
  - ICE gathering で空URLを正しく処理（`iceServers: []`）
- GGRS入力同期: `GameInput`のビットフラグ（u16）

### P2Pゲームの視点
各クライアントは自分の視点でゲームを見る:
- `LocalPlayer`: 自分（手札、デッキ、コストを持つ）
- `Opponent`: 相手（HPのみ）

**重要**: 相手がカードをプレイすると、自分の`LocalPlayer`のHPが減る（相手から攻撃を受けた）

### ゲームシステム
- コスト: 時間経過で蓄積（1.0/秒）
- カード: CardRegistry、CardDef、CardEffect (Damage/Heal/Draw)
- デッキ/手札/捨て札: Deck, Hand, DiscardPile コンポーネント

### キーボード操作（全操作キーボード完結）
- `D`: 1枚ドロー（コスト=手札枚数、0枚なら無料）
- `1-9`: 手札の1-9番目のカードをプレイ
- `0`: 10番目のカードをプレイ
- `P` / `Escape`: ポーズメニュー
- `Space`: 勝敗画面からタイトルに戻る

## GGRS同期の仕組み（重要）

### 入力フロー
1. `read_local_inputs` (ReadInputsスケジュール): キーボード入力 → `LocalInputs` リソース
2. GGRSが入力を同期 → `PlayerInputs` リソースに両プレイヤーの入力が格納
3. `process_ggrs_inputs` (GgrsSchedule): `PlayerInputs`から入力を読み取りゲーム状態更新

### プレイヤーハンドルの区別
```rust
let local_handle = local_players.0.first().copied().unwrap_or(0);
for (handle, (input, _status)) in inputs.iter().enumerate() {
    let is_local = handle == local_handle;
    if is_local {
        // 自分の入力 → 相手にダメージ
    } else {
        // 相手の入力 → 自分にダメージ
    }
}
```

### Rollback対象コンポーネント
`rollback_component_with_clone`で登録済み:
- Health, Cost, Hand, Deck, DiscardPile

## BRP (Bevy Remote Protocol) によるデバッグ

### セットアップ
Cargo.tomlで`bevy_remote`機能を有効化済み。main.rsでRemotePlugin追加済み（devフィーチャー時のみ）。

### ポート設定
```bash
# クライアント1（デフォルト15702）
cargo run --features dev

# クライアント2（ポート15703指定）
cargo run --features dev -- --brp-port=15703
```

### BRP操作例

```bash
# リソース一覧
curl -s -X POST http://127.0.0.1:15702/brp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.list_resources","params":{}}'

# 画面遷移（GotoScreenリソース挿入）
curl -s -X POST http://127.0.0.1:15702/brp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.insert_resources","params":{"resource":"sensen::screens::GotoScreen","value":"Lobby"}}'

# Opponent (相手) のHP確認
curl -s -X POST http://127.0.0.1:15702/brp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.query","params":{"data":{"components":["sensen::game::health::Health"]},"filter":{"with":["sensen::game::player::Opponent"]}}}'

# LocalPlayer (自分) のHP確認
curl -s -X POST http://127.0.0.1:15702/brp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.query","params":{"data":{"components":["sensen::game::health::Health"]},"filter":{"with":["sensen::game::player::LocalPlayer"]}}}'

# GGRS同期入力をシミュレート（P2P同期される）
curl -s -X POST http://127.0.0.1:15702/brp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.insert_resources","params":{"resource":"sensen::network::input::SimulatedGgrsInput","value":"D"}}'
# "D" = ドロー, "1"-"9" = カード1-9, "0" = カード10
```

### 重要なリソース

| リソース | 用途 |
|---------|------|
| `sensen::screens::GotoScreen` | BRP経由で画面遷移 |
| `sensen::network::input::SimulatedGgrsInput` | BRP経由でGGRS入力（P2P同期される） |
| `sensen::game::ui::SimulateInput` | ローカル入力のみ（P2P同期されない、非推奨） |

## テスト手順

### P2P対戦テスト
```bash
# 1. matchbox_serverが起動していることを確認
pgrep -f matchbox_server || matchbox_server &

# 2. クライアント起動
cargo run --features dev -- --brp-port=15702 > /tmp/client1.log 2>&1 &
sleep 2
cargo run --features dev -- --brp-port=15703 > /tmp/client2.log 2>&1 &
sleep 3

# 3. 両方をLobbyに遷移
curl -s -X POST http://127.0.0.1:15702/brp -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.insert_resources","params":{"resource":"sensen::screens::GotoScreen","value":"Lobby"}}'
curl -s -X POST http://127.0.0.1:15703/brp -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.insert_resources","params":{"resource":"sensen::screens::GotoScreen","value":"Lobby"}}'

# 4. GGRS接続待ち（自動でGameplayに遷移）
sleep 5

# 5. ドロー＆カードプレイ
curl -s -X POST http://127.0.0.1:15702/brp -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.insert_resources","params":{"resource":"sensen::network::input::SimulatedGgrsInput","value":"D"}}'
sleep 0.5
curl -s -X POST http://127.0.0.1:15702/brp -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.insert_resources","params":{"resource":"sensen::network::input::SimulatedGgrsInput","value":"1"}}'

# 6. HP同期確認
# Client 1の視点: Opponent HP減少（自分が相手を攻撃）
# Client 2の視点: Player HP減少（相手から攻撃を受けた）
```

### 期待される同期結果
Client 1がカードをプレイした場合:
| | Client 1 | Client 2 |
|---|---|---|
| Opponent HP | 95.0 (減少) | 100.0 (変化なし) |
| Player HP | 100.0 (変化なし) | 95.0 (減少) |

これは正しい動作。各クライアントは自分の視点でゲームを見ている。

## ファイル構成

- `src/main.rs`: BRPポート設定（--brp-port引数）
- `src/network/mod.rs`: GGRSプラグイン、ゲームロジック（GgrsSchedule）
- `src/network/lobby.rs`: Matchbox接続、GGRSセッション作成
- `src/network/input.rs`: 入力ビットフラグ、read_local_inputs、SimulatedGgrsInput
- `src/screens/mod.rs`: 画面State、GotoScreenリソース
- `src/game/ui.rs`: ゲームUI、キーボード操作
- `src/game/player.rs`: LocalPlayer、Opponent、PlayerBundle
- `src/game/health.rs`: Health、ダメージ/回復処理

## 確認済み動作

- [x] ビルド成功
- [x] BRP複数ポート対応（--brp-port=XXXX）
- [x] Matchboxサーバー接続
- [x] WebRTC P2P接続確立
- [x] GGRS同期完了
- [x] Gameplay画面への遷移
- [x] プレイヤーエンティティ生成
- [x] **P2Pゲームロジック同期（GgrsScheduleで実行）**
- [x] **BRP経由のGGRS入力シミュレーション（SimulatedGgrsInput）**

## デプロイ

### GitHub Pages (バージョン管理)
- URL: `roku36.github.io/sensen/` (バージョン一覧)
- `roku36.github.io/sensen/latest/` → 最新版
- `roku36.github.io/sensen/v0.1.2/` → 特定バージョン
- gh-pagesブランチに直接push方式

### Matchbox Server (fly.io)
- App: `sensen-matchbox` (Tokyo/nrt)
- URL: `wss://sensen-matchbox.fly.dev/`
- 設定: `matchbox-server/Dockerfile` + `matchbox-server/fly.toml`
- デプロイ: `cd matchbox-server && flyctl deploy`

### リリース手順
```bash
tools/release 0.2.0
# → Cargo.toml更新、commit、push
# → GitHub Actions: ビルド → gh-pagesブランチに v0.2.0/ と latest/ をデプロイ
```

## トラブルシューティング

### BRPに接続できない
- エンドポイントは `/brp` (例: `http://127.0.0.1:15702/brp`)
- devフィーチャーが有効か確認: `cargo run --features dev`
- BRPは`dev_native`フィーチャーのみ（wasmでは使用不可）

### Matchbox接続失敗
- ローカル: `pgrep -f matchbox_server || matchbox_server &`
- 本番: `flyctl status -a sensen-matchbox` で確認
- fly.ioのauto_stop有効のため、初回接続時に起動待ちが発生する場合あり

### Web版P2P接続が遅い
- ICE gatheringのSTUNタイムアウトが原因（~40秒）
- localhostでは `RtcIceServerConfig { urls: vec![] }` でSTUN無効化済み
- ブラウザコンソールの `[OFFER +Xms]` ログでボトルネック特定可能

### P2P同期が動作しない
- `SimulatedGgrsInput` を使っているか確認（`SimulateInput`はローカルのみ）
- GGRSセッションが確立しているか確認（ログで`Synchronized`を探す）
