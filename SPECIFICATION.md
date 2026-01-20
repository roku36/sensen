# Sensen - カードゲーム仕様書

## 概要

**Sensen**は、Bevy 0.18で開発されたリアルタイムカードゲームです。プレイヤーは時間経過で蓄積されるコストを使ってカードをプレイし、相手にダメージを与えたり、自身を回復したりします。

---

## ゲームシステム

### コストシステム

| 項目 | 値 |
|------|-----|
| 蓄積レート | 1.0 / 秒 |
| 初期値 | 0.0 |
| 上限 | なし |

コストは時間経過で自動的に蓄積され、カードのプレイやドローに消費されます。

### 状態効果

| 効果 | 内容 |
|------|------|
| Block | 被ダメージ時に先に消費される防御値 |
| Thorns | 攻撃を受けると攻撃者に反射ダメージ |
| Acceleration | 一定時間コスト蓄積レートが増加 |

### デッキ構成

初期デッキは12枚で構成されます：

| カード名 | CardId | 枚数 | コスト | 効果 |
|----------|--------|------|--------|------|
| Quick Strike | 1 | 2枚 | 1.0 | 5ダメージ |
| Heavy Blow | 2 | 1枚 | 3.0 | 15ダメージ |
| Heal | 3 | 1枚 | 2.0 | 10回復 |
| Draw | 4 | 1枚 | 1.5 | 2枚ドロー |
| Defend | 5 | 2枚 | 1.0 | 6ブロック |
| Shield Bash | 6 | 1枚 | 2.0 | 8ダメージ + 5ブロック |
| Bramble | 7 | 1枚 | 1.5 | 3棘 |
| Ironbark | 8 | 1枚 | 2.5 | 8ブロック + 2棘 |
| Adrenaline | 9 | 1枚 | 1.5 | 加速 +0.6/秒 (6秒) |
| Rush | 10 | 1枚 | 2.0 | 6ダメージ + 加速 +0.4/秒 (4秒) |

### カードプレイ

- カードをプレイするとコストが消費される
- プレイされたカードは即座にデッキの上に戻る
- コストが足りない場合はプレイできない（カードがグレーアウト表示）

### ドロー

| 項目 | 値 |
|------|-----|
| ドローコスト | 2.0 |
| ドロー枚数 | 3枚 |

- デッキが空の場合、捨て札をシャッフルしてデッキに戻す
- 初期手札は5枚

---

## 操作方法

### キーボード操作（推奨）

| キー | 機能 |
|------|------|
| 1-9 | 手札の1-9番目のカードをプレイ |
| 0 | 手札の10番目のカードをプレイ |
| D | 3枚ドロー（コスト2.0） |
| P / Escape | ポーズメニュー |
| F1 | Gameplay画面に直接ジャンプ（デバッグ用） |
| ` (バッククォート) | UIデバッグオーバーレイ切り替え |

### マウス操作

- カードをクリック: カードをプレイ
- 「Draw 3」ボタンをクリック: 3枚ドロー

---

## 画面遷移

```
Splash (1.8秒) → Title → Gameplay
                    ↓
               Settings / Credits
```

### Title画面

- Play: ゲーム開始
- Settings: 設定画面
- Credits: クレジット画面
- Exit: 終了（デスクトップ版のみ）

### Gameplay画面

- 上部左: コスト表示（リアルタイム更新、レート表示）
- 上部右: デッキ枚数、捨て札枚数
- HPバー付近: Block / Thorns の現在値
- 下部: 手札表示（キーラベル付き）、ドローボタン

---

## 技術仕様

### 依存関係

- **Bevy**: 0.18
- **bevy_brp_extras**: 0.18 (リモートデバッグ用)
- **rand**: 0.9 (デッキシャッフル用)

### BRP (Bevy Remote Protocol)

ゲームはBRPを通じてリモート制御が可能です。AIアシスタント（MCP）からの操作に対応しています。

**エンドポイント**: `http://localhost:15702`

#### 利用可能なメソッド

| メソッド | 説明 |
|----------|------|
| `brp_extras/screenshot` | スクリーンショット撮影 |
| `brp_extras/send_keys` | キーボード入力送信 |
| `brp_extras/shutdown` | アプリケーション終了 |
| `brp_extras/set_window_title` | ウィンドウタイトル変更 |
| `world.query` | エンティティ・コンポーネントのクエリ |
| `world.list_components` | 登録コンポーネント一覧 |
| `rpc.discover` | 利用可能なメソッド一覧 |

#### MCP/AI操作例

```bash
# ゲーム起動後、Gameplay画面にジャンプ
curl -X POST http://localhost:15702 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "brp_extras/send_keys", "id": 1, "params": {"keys": ["F1"]}}'

# カードをプレイ（1番目のカード）
curl -X POST http://localhost:15702 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "brp_extras/send_keys", "id": 1, "params": {"keys": ["Digit1"]}}'

# ドロー
curl -X POST http://localhost:15702 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "brp_extras/send_keys", "id": 1, "params": {"keys": ["KeyD"]}}'

# ゲーム状態クエリ
curl -X POST http://localhost:15702 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "world.query", "id": 1, "params": {"data": {"components": ["sensen::game::cost::Cost", "sensen::game::deck::Hand", "sensen::game::deck::Deck", "sensen::game::deck::DiscardPile"]}}}'

# スクリーンショット
curl -X POST http://localhost:15702 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc": "2.0", "method": "brp_extras/screenshot", "id": 1, "params": {"path": "/tmp/screenshot.png"}}'
```

### リフレクション対応コンポーネント

BRP経由でクエリ可能なゲームコンポーネント：

| コンポーネント | 説明 |
|----------------|------|
| `sensen::game::cost::Cost` | コスト状態（current, rate） |
| `sensen::game::deck::Hand` | 手札（cards: Vec<CardId>） |
| `sensen::game::deck::Deck` | デッキ（cards: Vec<CardId>） |
| `sensen::game::deck::DiscardPile` | 捨て札（cards: Vec<CardId>） |
| `sensen::game::card::CardId` | カードID（u32） |

---

## プロジェクト構造

```
src/
├── main.rs              # エントリポイント、AppPlugin
├── asset_tracking.rs    # アセット読み込み管理
├── audio.rs             # オーディオシステム
├── dev_tools.rs         # 開発ツール（F1ショートカット等）
├── game/                # ゲームロジック
│   ├── mod.rs
│   ├── card.rs          # カード定義、CardRegistry
│   ├── cost.rs          # コストシステム
│   ├── deck.rs          # デッキ、手札、捨て札管理
│   ├── player.rs        # プレイヤーエンティティ
│   └── ui.rs            # ゲームUI、キーボード操作
├── menus/               # メニュー画面
│   ├── main.rs          # タイトルメニュー
│   ├── pause.rs         # ポーズメニュー
│   ├── settings.rs      # 設定画面
│   └── credits.rs       # クレジット画面
├── screens/             # 画面状態管理
│   ├── splash.rs        # スプラッシュ画面
│   ├── title.rs         # タイトル画面
│   ├── loading.rs       # ローディング画面
│   └── gameplay.rs      # ゲームプレイ画面
├── theme/               # UIテーマ
│   ├── interaction.rs   # ボタンインタラクション
│   ├── palette.rs       # カラーパレット
│   └── widget.rs        # UIウィジェット
└── demo/                # レベル設定
    └── level.rs         # ゲームレベル初期化
```

---

## ビルド・実行

### 開発ビルド

```bash
cargo run
```

### リリースビルド

```bash
cargo build --release
```

### Web版ビルド

```bash
cargo build --target wasm32-unknown-unknown --profile web-release
```

---

## 変更履歴

### v0.1.0
- 初期実装
- コストシステム、カードシステム、デッキ管理
- キーボード完全対応（1-9, 0でカードプレイ、Dでドロー）
- BRP（Bevy Remote Protocol）対応
- ホバー音の連続再生バグ修正

---

## 今後の拡張予定

- [ ] 対戦相手AI
- [ ] HP/ライフシステム
- [ ] 勝敗判定
- [ ] カード効果の実際の適用
- [ ] デッキ編集機能
- [ ] セーブ/ロード機能
- [ ] カード追加（バフ、デバフ、条件付き効果など）
